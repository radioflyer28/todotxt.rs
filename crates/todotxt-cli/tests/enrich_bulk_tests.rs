mod helpers;

use chrono::{Datelike, Duration, Local, NaiveTime};
use filetime::{set_file_mtime, FileTime};
use helpers::TestFixture;
use predicates::str::contains;
use predicates::prelude::PredicateBooleanExt;
use std::fs;
use toml::Value;

// Task content used for enrichment tests.
// Line 1: "(A) Buy milk +groceries @home"  — already has priority
// Line 2: "(B) Send report +work @office"  — different priority
// Line 3: "x 2024-01-01 Done task +work"  — completed
// Line 4: "Call dentist @personal"         — no priority, no due date
const SAMPLE_TODO: &str = "(A) Buy milk +groceries @home\n\
(B) Send report +work @office\n\
x 2024-01-01 Done task +work\n\
Call dentist @personal\n";

/// Fixture with a `done_file` entry pointing to a sibling `done.txt`.
fn fixture_with_done_file() -> (TestFixture, std::path::PathBuf) {
    let fixture = TestFixture::with_content(SAMPLE_TODO);
    let done_path = fixture.dir.path().join("done.txt");
    let todo_path = fixture.todo.path().to_string_lossy().into_owned();
    let done_str = done_path.to_string_lossy().into_owned();
    let toml = format!(
        "todo_file = {}\ndone_file = {}\n",
        Value::String(todo_path),
        Value::String(done_str)
    );
    fs::write(fixture.config.path(), toml).expect("rewrite config with done_file");
    (fixture, done_path)
}

fn previous_month_date() -> chrono::NaiveDate {
    let today = Local::now().date_naive();
    let first_of_month = today.with_day(1).unwrap();
    first_of_month
        .checked_sub_signed(Duration::days(1))
        .unwrap()
}

// ── pri / depri tests ────────────────────────────────────────────────────────

#[test]
fn test_pri_sets_priority() {
    let fixture = TestFixture::with_content("Buy milk +groceries @home\nSend report\n");
    fixture.cmd().args(["pri", "B", "1"]).assert().success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(
        content.lines().next().unwrap().starts_with("(B) "),
        "expected first task to have priority (B), got: {}",
        content.lines().next().unwrap()
    );
}

#[test]
fn test_pri_replaces_existing() {
    // Task 1 starts with (A); after `pri A 1 B` it should have (B).
    // NOTE: CLI syntax: pri <priority> <ids...>
    let fixture = TestFixture::with_content("(A) Buy milk\nSend report\n");
    fixture.cmd().args(["pri", "B", "1"]).assert().success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    let first = content.lines().next().unwrap();
    assert!(
        first.starts_with("(B) "),
        "expected (B) prefix, got: {first}"
    );
    assert!(!first.contains("(A)"), "old priority (A) should be gone");
}

#[test]
fn test_pri_multi_id() {
    let fixture = TestFixture::with_content("Task one\nTask two\nTask three\n");
    fixture
        .cmd()
        .args(["pri", "A", "1", "2", "3"])
        .assert()
        .success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    for line in content.lines().take(3) {
        assert!(
            line.starts_with("(A) "),
            "expected (A) prefix on all tasks, got: {line}"
        );
    }
}

#[test]
fn test_pri_invalid_letter_exits_2() {
    let fixture = TestFixture::with_content("Task one\n");
    // '1' is not an ASCII alphabetic char → CliError::Other → exit 2
    fixture.cmd().args(["pri", "1", "1"]).assert().code(2);
}

#[test]
fn test_depri_removes_priority() {
    let fixture = TestFixture::with_content("(A) Buy milk\nSend report\n");
    fixture.cmd().args(["depri", "1"]).assert().success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    let first = content.lines().next().unwrap();
    assert!(
        !first.starts_with("(A)"),
        "priority should be removed, got: {first}"
    );
    assert!(first.contains("Buy milk"), "task text should remain");
}

#[test]
fn test_depri_multi_id() {
    let fixture = TestFixture::with_content("(A) Task one\n(B) Task two\nTask three\n");
    fixture.cmd().args(["depri", "1", "2"]).assert().success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    let mut lines = content.lines();
    let first = lines.next().unwrap();
    let second = lines.next().unwrap();
    assert!(!first.starts_with("(A)"), "task 1 priority should be gone");
    assert!(!second.starts_with("(B)"), "task 2 priority should be gone");
}

#[test]
fn test_depri_idempotent() {
    // depri on a task that already has no priority should succeed (skip with info message)
    let fixture = TestFixture::with_content("Task one\n");
    fixture.cmd().args(["depri", "1"]).assert().success();
}

// ── due / postpone tests ─────────────────────────────────────────────────────

#[test]
fn test_due_iso_date() {
    let fixture = TestFixture::with_content("Buy milk\n");
    fixture
        .cmd()
        .args(["due", "1", "2026-12-31"])
        .assert()
        .success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(
        content.contains("due:2026-12-31"),
        "expected due:2026-12-31, got: {content}"
    );
}

#[test]
fn test_due_today() {
    let fixture = TestFixture::with_content("Buy milk\n");
    fixture.cmd().args(["due", "1", "today"]).assert().success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    // Just verify the due: tag was added in YYYY-MM-DD format
    assert!(
        content.contains("due:20"),
        "expected due:YYYY-MM-DD after 'today', got: {content}"
    );
}

#[test]
fn test_due_tomorrow() {
    let fixture = TestFixture::with_content("Buy milk\n");
    fixture
        .cmd()
        .args(["due", "1", "tomorrow"])
        .assert()
        .success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(
        content.contains("due:20"),
        "expected due:YYYY-MM-DD after 'tomorrow', got: {content}"
    );
}

#[test]
fn test_due_weekday() {
    // Weekday resolution: next occurrence (never today itself)
    let fixture = TestFixture::with_content("Buy milk\n");
    fixture
        .cmd()
        .args(["due", "1", "monday"])
        .assert()
        .success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(
        content.contains("due:20"),
        "expected a due date set for monday, got: {content}"
    );
}

#[test]
fn test_due_invalid_format_exits_2() {
    let fixture = TestFixture::with_content("Buy milk\n");
    fixture
        .cmd()
        .args(["due", "1", "not-a-date"])
        .assert()
        .code(2)
        .stderr(contains("invalid date"));
}

#[test]
fn test_due_nonexistent_id_exits_1() {
    let fixture = TestFixture::with_content("Buy milk\n");
    fixture
        .cmd()
        .args(["due", "99", "2026-12-31"])
        .assert()
        .code(1);
}

#[test]
fn test_due_json_output() {
    let fixture = TestFixture::with_content("Buy milk\n");
    let output = fixture
        .cmd()
        .args(["--json", "due", "1", "2026-12-31"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(output).unwrap();
    let val: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
    assert_eq!(val["schema_version"], 1, "schema_version should be 1");
    assert!(
        val["data"].is_object() || val["data"].is_array(),
        "data field should be present"
    );
}

#[test]
fn test_postpone_adds_days() {
    // Start with a task that has due:2026-04-15, postpone by 7 → 2026-04-22
    let fixture = TestFixture::with_content("Buy milk due:2026-04-15\n");
    fixture
        .cmd()
        .args(["postpone", "1", "7"])
        .assert()
        .success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(
        content.contains("due:2026-04-22"),
        "expected due:2026-04-22 after +7 days, got: {content}"
    );
}

#[test]
fn test_postpone_no_due_date_exits_2() {
    // Task has no due date → CliError::Other → exit 2
    let fixture = TestFixture::with_content("Buy milk\n");
    fixture
        .cmd()
        .args(["postpone", "1", "7"])
        .assert()
        .code(2)
        .stderr(contains("no due date"));
}

#[test]
fn test_postpone_cross_month() {
    // 2026-04-15 + 20 days = 2026-05-05
    let fixture = TestFixture::with_content("Buy milk due:2026-04-15\n");
    fixture
        .cmd()
        .args(["postpone", "1", "20"])
        .assert()
        .success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(
        content.contains("due:2026-05-05"),
        "expected due:2026-05-05 after crossing month boundary, got: {content}"
    );
}

#[test]
fn test_postpone_json_output() {
    let fixture = TestFixture::with_content("Buy milk due:2026-04-15\n");
    let output = fixture
        .cmd()
        .args(["--json", "postpone", "1", "7"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(output).unwrap();
    let val: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
    assert_eq!(val["schema_version"], 1);
    assert!(val["data"].is_object(), "data field should be present");
}

// ── archive / del-done tests ─────────────────────────────────────────────────

#[test]
fn test_archive_moves_completed() {
    let (fixture, done_path) = fixture_with_done_file();

    fixture.cmd().args(["archive"]).assert().success();

    let todo_content = fs::read_to_string(fixture.todo.path()).unwrap();
    let done_content = fs::read_to_string(&done_path).unwrap();

    assert!(
        !todo_content.contains("x 2024-01-01 Done task"),
        "completed task should be removed from todo.txt"
    );
    assert!(
        done_content.contains("x 2024-01-01 Done task"),
        "completed task should appear in done.txt"
    );
}

#[test]
fn test_archive_creates_done_txt() {
    let (fixture, done_path) = fixture_with_done_file();
    assert!(!done_path.exists(), "done.txt should not exist yet");

    fixture.cmd().args(["archive"]).assert().success();

    assert!(done_path.exists(), "archive should create done.txt");
}

#[test]
fn test_archive_empty_list_exits_0() {
    let fixture = TestFixture::with_content("No done tasks here\nAnother incomplete\n");
    fixture
        .cmd()
        .args(["archive"])
        .assert()
        .success()
        .stderr(contains("0"));
}

#[test]
fn test_archive_idempotent() {
    let (fixture, done_path) = fixture_with_done_file();

    fixture.cmd().args(["archive"]).assert().success();
    fixture.cmd().args(["archive"]).assert().success();

    // Second run: 0 completed remain in todo.txt → done.txt unchanged
    let done_content = fs::read_to_string(&done_path).unwrap();
    let count = done_content.lines().filter(|l| l.starts_with("x ")).count();
    assert_eq!(
        count, 1,
        "done.txt should have exactly 1 archived task after two runs"
    );
}

#[test]
fn test_archive_atomicity() {
    let (fixture, done_path) = fixture_with_done_file();

    fixture.cmd().args(["archive"]).assert().success();

    let todo_content = fs::read_to_string(fixture.todo.path()).unwrap();
    let done_content = fs::read_to_string(&done_path).unwrap();

    let in_todo = todo_content.contains("x 2024-01-01 Done task");
    let in_done = done_content.contains("x 2024-01-01 Done task");

    assert!(!in_todo, "completed task must not remain in todo.txt");
    assert!(in_done, "completed task must appear in done.txt");
    // Verify incomplete tasks are NOT in done.txt
    assert!(
        !done_content.contains("Buy milk"),
        "incomplete task should not be in done.txt"
    );
}

#[test]
fn test_archive_json_output() {
    let (fixture, _) = fixture_with_done_file();
    let output = fixture
        .cmd()
        .args(["--json", "archive"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(output).unwrap();
    let val: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
    assert_eq!(val["schema_version"], 1);
    assert_eq!(val["data"]["count"], 1, "should report 1 archived task");
}

#[test]
fn test_archive_rotates_prior_period_done_txt_and_reports_it() {
    let (fixture, done_path) = fixture_with_done_file();
    fs::write(&done_path, "x 2026-01-01 old archived task\n").unwrap();

    let previous_month = previous_month_date();
    let previous_month_time = previous_month.and_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap());
    set_file_mtime(
        &done_path,
        FileTime::from_unix_time(previous_month_time.and_utc().timestamp(), 0),
    )
    .unwrap();

    let rotated_name = format!(
        "done-{:04}-{:02}.txt",
        previous_month.year(),
        previous_month.month()
    );
    let rotated_path = fixture.dir.path().join(&rotated_name);

    fixture
        .cmd()
        .args(["archive"])
        .assert()
        .success()
        .stderr(contains("Rotated previous done.txt to").and(contains(rotated_name.as_str())));

    let rotated_content = fs::read_to_string(&rotated_path).unwrap();
    assert!(
        rotated_content.contains("old archived task"),
        "prior done.txt content should move into the rotated period file"
    );

    let active_done_content = fs::read_to_string(&done_path).unwrap();
    assert!(
        active_done_content.contains("x 2024-01-01 Done task +work"),
        "newly archived task should be written into the fresh active done.txt"
    );
    assert!(
        !active_done_content.contains("old archived task"),
        "fresh active done.txt should not retain prior-period content after rotation"
    );
}

#[test]
fn test_del_done_removes_completed() {
    let fixture = TestFixture::with_content(SAMPLE_TODO);

    fixture.cmd().args(["del-done"]).assert().success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(
        !content.contains("x 2024-01-01 Done task"),
        "completed task should be removed"
    );
    assert!(content.contains("Buy milk"), "incomplete tasks must remain");
}

#[test]
fn test_del_done_empty_list_exits_0() {
    let fixture = TestFixture::with_content("Incomplete one\nIncomplete two\n");
    fixture
        .cmd()
        .args(["del-done"])
        .assert()
        .success()
        .stderr(contains("0"));
}

#[test]
fn test_del_done_idempotent() {
    let fixture = TestFixture::with_content(SAMPLE_TODO);

    fixture.cmd().args(["del-done"]).assert().success();
    fixture.cmd().args(["del-done"]).assert().success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(
        !content.contains("x 2024-01-01"),
        "no completed tasks after second run"
    );
}

#[test]
fn test_del_done_json_output() {
    let fixture = TestFixture::with_content(SAMPLE_TODO);
    let output = fixture
        .cmd()
        .args(["--json", "del-done"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(output).unwrap();
    let val: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
    assert_eq!(val["schema_version"], 1);
    assert_eq!(val["data"]["count"], 1, "should report 1 deleted task");
}

// ── exit code verification tests ─────────────────────────────────────────────

#[test]
fn test_invalid_id_exits_1() {
    let fixture = TestFixture::with_content("Task one\n");
    fixture
        .cmd()
        .args(["pri", "A", "99"])
        .assert()
        .code(1)
        .stderr(contains("not found"));
}

#[test]
fn test_validation_error_exits_2() {
    // Non-alphabetic char as priority → exit 2
    let fixture = TestFixture::with_content("Task one\n");
    fixture.cmd().args(["pri", "9", "1"]).assert().code(2);
}

#[test]
fn test_success_exits_0() {
    let fixture = TestFixture::with_content("Task one\n");
    fixture.cmd().args(["pri", "A", "1"]).assert().code(0);
}

#[test]
fn test_due_success_exits_0() {
    let fixture = TestFixture::with_content("Task one\n");
    fixture
        .cmd()
        .args(["due", "1", "2026-12-31"])
        .assert()
        .code(0);
}

#[test]
fn test_postpone_cross_year() {
    // 2026-12-25 + 10 days = 2027-01-04
    let fixture = TestFixture::with_content("Holiday task due:2026-12-25\n");
    fixture
        .cmd()
        .args(["postpone", "1", "10"])
        .assert()
        .success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(
        content.contains("due:2027-01-04"),
        "expected due:2027-01-04 after crossing year boundary, got: {content}"
    );
}

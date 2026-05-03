mod helpers;

use helpers::TestFixture;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use std::fs;
use toml::Value;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Create a TestFixture whose config.toml also points to a `done_file`.
/// The `done.txt` is written with `done_content` if non-empty, or is not
/// created at all when `done_content` is `None`.
fn fixture_with_done_file(
    todo_content: &str,
    done_content: Option<&str>,
) -> (TestFixture, std::path::PathBuf) {
    let fixture = TestFixture::with_content(todo_content);
    let done_path = fixture.dir.path().join("done.txt");
    if let Some(dc) = done_content {
        fs::write(&done_path, dc).expect("write done.txt");
    }
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

// ── Group 1: Alias tests ──────────────────────────────────────────────────────

#[test]
fn test_alias_a_adds_task() {
    let fixture = TestFixture::with_content("Buy milk\n");
    fixture
        .cmd()
        .args(["a", "Call dentist"])
        .assert()
        .success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(content.contains("Call dentist"), "task should be present, got: {content}");
}

#[test]
fn test_alias_rm_deletes_task() {
    let fixture = TestFixture::with_content("Buy milk\nSend report\n");
    fixture.cmd().args(["rm", "1"]).assert().success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(!content.contains("Buy milk"), "deleted task should be gone, got: {content}");
    assert!(content.contains("Send report"), "second task should remain");
}

#[test]
fn test_alias_done_completes_task() {
    let fixture = TestFixture::with_content("Buy milk\n");
    fixture.cmd().args(["done", "1"]).assert().success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(content.starts_with("x "), "completed task should start with 'x ', got: {content}");
}

#[test]
fn test_alias_dp_removes_priority() {
    let fixture = TestFixture::with_content("(A) Buy milk\n");
    fixture.cmd().args(["dp", "1"]).assert().success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(!content.starts_with("(A)"), "priority should be removed, got: {content}");
    assert!(content.contains("Buy milk"), "task text should remain");
}

#[test]
fn test_alias_p_sets_priority() {
    let fixture = TestFixture::with_content("Buy milk\n");
    fixture.cmd().args(["p", "A", "1"]).assert().success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(content.starts_with("(A)"), "priority should be set, got: {content}");
}

#[test]
fn test_alias_app_appends_text() {
    let fixture = TestFixture::with_content("Buy milk\n");
    fixture.cmd().args(["app", "1", "due:tomorrow"]).assert().success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(content.contains("due:tomorrow"), "appended text should be present, got: {content}");
}

#[test]
fn test_alias_prep_prepends_text() {
    let fixture = TestFixture::with_content("Buy milk\n");
    fixture.cmd().args(["prep", "1", "(A)"]).assert().success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(content.contains("(A)"), "prepended text should be present, got: {content}");
}

#[test]
fn test_alias_lsc_lists_contexts() {
    let fixture = TestFixture::with_content("Buy milk @home\nSend report @office\n");
    fixture
        .cmd()
        .args(["lsc"])
        .assert()
        .success()
        .stdout(contains("@home"))
        .stdout(contains("@office"));
}

#[test]
fn test_alias_lsprj_lists_projects() {
    let fixture = TestFixture::with_content("Buy milk +groceries\nSend report +work\n");
    fixture
        .cmd()
        .args(["lsprj"])
        .assert()
        .success()
        .stdout(contains("+groceries"))
        .stdout(contains("+work"));
}

// ── Group 2: --all flag tests ─────────────────────────────────────────────────

#[test]
fn test_list_default_hides_future_threshold() {
    let fixture = TestFixture::with_content("Deferred task t:2099-12-31\nNormal task\n");
    fixture
        .cmd()
        .args(["list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Deferred task").not());
}

#[test]
fn test_list_all_shows_future_threshold() {
    let fixture = TestFixture::with_content("Deferred task t:2099-12-31\nNormal task\n");
    fixture
        .cmd()
        .args(["list", "--all"])
        .assert()
        .success()
        .stdout(contains("Deferred task"));
}

#[test]
fn test_list_default_hides_hidden_task() {
    let fixture = TestFixture::with_content("Hidden task h:1\nVisible task\n");
    fixture
        .cmd()
        .args(["list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Hidden task").not());
}

#[test]
fn test_list_all_shows_hidden_task() {
    let fixture = TestFixture::with_content("Hidden task h:1\nVisible task\n");
    fixture
        .cmd()
        .args(["list", "--all"])
        .assert()
        .success()
        .stdout(contains("Hidden task"));
}

#[test]
fn test_list_threshold_past_is_shown_by_default() {
    let fixture = TestFixture::with_content("Old task t:2020-01-01\n");
    fixture
        .cmd()
        .args(["list"])
        .assert()
        .success()
        .stdout(contains("Old task"));
}

// ── Group 3: --compat flag tests ─────────────────────────────────────────────

#[test]
fn test_list_compat_format() {
    let fixture = TestFixture::with_content("(A) Buy milk\nSend report\n");
    let output = fixture
        .cmd()
        .args(["list", "--compat"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(output).unwrap();
    // Each output line should be "{N} {raw_task}"
    assert!(
        text.lines().any(|l| l.starts_with("1 (A) Buy milk") || l.starts_with("2 (A) Buy milk")),
        "expected '1 (A) Buy milk' or '2 (A) Buy milk' in compat output, got:\n{text}"
    );
    // No table border characters from comfy-table
    assert!(!text.contains('─'), "compat output should not contain table borders, got:\n{text}");
    assert!(!text.contains('│'), "compat output should not contain table borders, got:\n{text}");
}

#[test]
fn test_list_compat_numbering_matches_id() {
    // Task at line 1 should be numbered 1 in compat output
    let fixture = TestFixture::with_content("First task\nSecond task\n");
    let output = fixture
        .cmd()
        .args(["list", "--compat"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(output).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert!(
        lines.iter().any(|l| l.starts_with("1 First task")),
        "expected '1 First task' in output, got:\n{text}"
    );
    assert!(
        lines.iter().any(|l| l.starts_with("2 Second task")),
        "expected '2 Second task' in output, got:\n{text}"
    );
}

// ── Group 4: listpri tests ────────────────────────────────────────────────────

#[test]
fn test_listpri_default_shows_all_priorities() {
    let fixture = TestFixture::with_content(
        "(A) High priority\n(C) Low priority\nNo priority\n",
    );
    fixture
        .cmd()
        .args(["listpri"])
        .assert()
        .success()
        .stdout(contains("(A) High priority"))
        .stdout(contains("(C) Low priority"))
        .stdout(predicates::str::contains("No priority").not());
}

#[test]
fn test_listpri_single_letter() {
    let fixture = TestFixture::with_content(
        "(A) High priority\n(B) Medium\n(C) Low\nNo priority\n",
    );
    fixture
        .cmd()
        .args(["listpri", "A"])
        .assert()
        .success()
        .stdout(contains("(A) High priority"))
        .stdout(predicates::str::contains("(B) Medium").not())
        .stdout(predicates::str::contains("(C) Low").not())
        .stdout(predicates::str::contains("No priority").not());
}

#[test]
fn test_listpri_range() {
    let fixture = TestFixture::with_content(
        "(A) First\n(B) Second\n(C) Third\n(D) Fourth\nNone\n",
    );
    fixture
        .cmd()
        .args(["listpri", "A-C"])
        .assert()
        .success()
        .stdout(contains("(A) First"))
        .stdout(contains("(B) Second"))
        .stdout(contains("(C) Third"))
        .stdout(predicates::str::contains("(D) Fourth").not())
        .stdout(predicates::str::contains("None").not());
}

#[test]
fn test_listpri_alias_lsp() {
    let fixture = TestFixture::with_content("(A) Task one\n(B) Task two\nNo pri\n");
    fixture
        .cmd()
        .args(["lsp", "A"])
        .assert()
        .success()
        .stdout(contains("(A) Task one"))
        .stdout(predicates::str::contains("(B) Task two").not());
}

#[test]
fn test_listpri_no_matching_tasks_exits_0() {
    let fixture = TestFixture::with_content("No priority task\n");
    fixture.cmd().args(["listpri"]).assert().success();
}

#[test]
fn test_listpri_invalid_spec_exits_nonzero() {
    let fixture = TestFixture::with_content("Buy milk\n");
    fixture
        .cmd()
        .args(["listpri", "123"])
        .assert()
        .failure();
}

// ── Group 5: listall tests ────────────────────────────────────────────────────

#[test]
fn test_listall_merges_todo_and_done() {
    let (fixture, done_path) = fixture_with_done_file(
        "(A) Active task\n",
        Some("x 2024-01-01 Completed task\n"),
    );
    let _ = done_path; // keep alive
    fixture
        .cmd()
        .args(["listall"])
        .assert()
        .success()
        .stdout(contains("Active task"))
        .stdout(contains("Completed task"));
}

#[test]
fn test_listall_missing_done_txt_exits_0() {
    // done.txt not created — listall should handle gracefully
    let fixture = TestFixture::with_content("(A) Active task\n");
    fixture
        .cmd()
        .args(["listall"])
        .assert()
        .success()
        .stdout(contains("Active task"));
}

#[test]
fn test_listall_alias_lsa() {
    let (fixture, done_path) = fixture_with_done_file(
        "Active task\n",
        Some("x 2024-01-01 Done task\n"),
    );
    let _ = done_path;
    fixture
        .cmd()
        .args(["lsa"])
        .assert()
        .success()
        .stdout(contains("Active task"))
        .stdout(contains("Done task"));
}

#[test]
fn test_listall_shows_hidden_tasks() {
    // listall bypasses h:1 suppression
    let fixture = TestFixture::with_content("Visible task\nHidden task h:1\n");
    fixture
        .cmd()
        .args(["listall"])
        .assert()
        .success()
        .stdout(contains("Hidden task"));
}

// ── Group 6: deduplicate tests ────────────────────────────────────────────────

#[test]
fn test_deduplicate_removes_exact_duplicate() {
    let fixture = TestFixture::with_content("Buy milk\nBuy milk\nSend report\n");
    fixture.cmd().args(["deduplicate"]).assert().success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 2, "expected 2 lines after dedup, got: {content}");
    assert!(content.contains("Buy milk"), "original task should remain");
    assert!(content.contains("Send report"), "second task should remain");
}

#[test]
fn test_deduplicate_no_duplicates_exits_0() {
    let fixture = TestFixture::with_content("Buy milk\nSend report\n");
    fixture.cmd().args(["deduplicate"]).assert().success();

    // File should be unchanged
    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 2, "file should be unchanged, got: {content}");
}

#[test]
fn test_deduplicate_multiple_duplicates() {
    let fixture = TestFixture::with_content("Task A\nTask B\nTask A\nTask C\nTask B\n");
    fixture.cmd().args(["deduplicate"]).assert().success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 3, "expected 3 unique tasks remaining, got: {content}");
}

#[test]
fn test_deduplicate_case_sensitive() {
    // "Buy milk" and "buy milk" are different strings — both should remain
    let fixture = TestFixture::with_content("Buy milk\nbuy milk\n");
    fixture.cmd().args(["deduplicate"]).assert().success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "case-different tasks should both survive, got: {content}"
    );
}

#[test]
fn test_deduplicate_idempotent() {
    let fixture = TestFixture::with_content("Buy milk\nBuy milk\nSend report\n");
    // Run twice — second run should find no duplicates
    fixture.cmd().args(["deduplicate"]).assert().success();
    fixture.cmd().args(["deduplicate"]).assert().success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 2, "should have 2 tasks after two passes, got: {content}");
}

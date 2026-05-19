mod helpers;

use helpers::TestFixture;
use predicates::str::contains;
use std::fs;
use toml::Value;

// ── Helpers ────────────────────────────────────────────────────────────────

/// Create a fixture with both todo_file and done_file configured.
fn fixture_with_done_file() -> (TestFixture, std::path::PathBuf) {
    let content = "(A) Buy milk +groceries @home\n\
(B) Send report +work @office\n\
Call dentist @personal\n";
    let fixture = TestFixture::with_content(content);
    let done_path = fixture.dir.path().join("done.txt");
    let todo_path = fixture.todo.path().to_string_lossy().into_owned();
    let done_str = done_path.to_string_lossy().into_owned();
    let toml = format!(
        "todo_file = {}\ndone_file = {}\n",
        Value::String(todo_path),
        Value::String(done_str),
    );
    fs::write(fixture.config.path(), toml).expect("rewrite config with done_file");
    (fixture, done_path)
}

// ── Scenario 1: Full workflow smoke test ───────────────────────────────────
//
// add → list → do → stats → archive
// Verifies the core lifecycle end to end.

#[test]
fn test_scenario_full_workflow_smoke() {
    let (fixture, done_path) = fixture_with_done_file();

    // add
    fixture
        .cmd()
        .args(["add", "Buy groceries +shopping"])
        .assert()
        .success();

    // list — our new task should appear
    fixture
        .cmd()
        .args(["list"])
        .assert()
        .success()
        .stdout(contains("Buy groceries"));

    // The new task was added as the last task — read ID from file
    let content = fs::read_to_string(fixture.todo.path()).expect("read todo");
    let task_count = content.lines().count();
    let new_id = task_count.to_string();

    // do {id}
    fixture.cmd().args(["do", &new_id]).assert().success();

    // stats — should show at least 1 completed
    let stats_out = fixture
        .cmd()
        .args(["stats"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stats_str = String::from_utf8_lossy(&stats_out);
    assert!(
        stats_str.contains("complete") || stats_str.contains("1"),
        "stats should reflect completed task: {stats_str}"
    );

    // archive — completed tasks move to done.txt
    fixture.cmd().args(["archive"]).assert().success();

    assert!(
        done_path.exists(),
        "done.txt must exist after archive with completed tasks"
    );

    let done_content = fs::read_to_string(&done_path).expect("read done.txt");
    assert!(
        done_content.contains("Buy groceries"),
        "archived task must appear in done.txt"
    );
}

// ── Scenario 2: Filter by project and context ─────────────────────────────

#[test]
fn test_scenario_filter_by_project_and_context() {
    let content = "Alpha task +alpha_project\n\
Beta task @beta_context\n\
Gamma task +alpha_project @beta_context\n\
Unrelated task\n";
    let fixture = TestFixture::with_content(content);

    // filter by +alpha_project — should see tasks 1 and 3 only
    let out = fixture
        .cmd()
        .args(["list", "+alpha_project"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8_lossy(&out);
    assert!(s.contains("Alpha task"), "Alpha task must appear");
    assert!(s.contains("Gamma task"), "Gamma task must appear");
    assert!(
        !s.contains("Unrelated task"),
        "Unrelated task must not appear"
    );

    // filter by @beta_context — should see tasks 2 and 3 only
    let out2 = fixture
        .cmd()
        .args(["list", "@beta_context"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s2 = String::from_utf8_lossy(&out2);
    assert!(s2.contains("Beta task"), "Beta task must appear");
    assert!(s2.contains("Gamma task"), "Gamma task must appear");
    assert!(
        !s2.contains("Alpha task +alpha_project\n"),
        "Alpha-only task must not appear for @beta filter"
    );

    // AND filter: +alpha_project AND @beta_context — only task 3
    let out3 = fixture
        .cmd()
        .args(["list", "+alpha_project", "@beta_context"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s3 = String::from_utf8_lossy(&out3);
    assert!(s3.contains("Gamma task"), "AND-filtered task must appear");
}

// ── Scenario 3: JSON output schema_version contract ───────────────────────
//
// Every command with --json MUST produce {"schema_version":1,...}
// Per D-02 (locked): schema_version assertion in every --json test.

#[test]
fn test_scenario_json_schema_version_contract() {
    let fixture = TestFixture::new();

    // list --json
    let out = fixture
        .cmd()
        .args(["--json", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value =
        serde_json::from_slice(&out).expect("list --json must produce valid JSON");
    assert_eq!(v["schema_version"], 1, "list: schema_version must be 1");
    assert!(v["data"].is_array(), "list: data must be an array");

    // stats --json
    let out2 = fixture
        .cmd()
        .args(["--json", "stats"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v2: serde_json::Value =
        serde_json::from_slice(&out2).expect("stats --json must produce valid JSON");
    assert_eq!(v2["schema_version"], 1, "stats: schema_version must be 1");
    assert!(v2["data"].is_object(), "stats: data must be an object");

    // add --json
    let out3 = fixture
        .cmd()
        .args(["--json", "add", "Test task for JSON contract"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v3: serde_json::Value =
        serde_json::from_slice(&out3).expect("add --json must produce valid JSON");
    assert_eq!(v3["schema_version"], 1, "add: schema_version must be 1");

    // show --json (show task 1)
    let out4 = fixture
        .cmd()
        .args(["--json", "show", "1"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v4: serde_json::Value =
        serde_json::from_slice(&out4).expect("show --json must produce valid JSON");
    assert_eq!(v4["schema_version"], 1, "show: schema_version must be 1");

    // do --json
    let out5 = fixture
        .cmd()
        .args(["--json", "do", "1"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v5: serde_json::Value =
        serde_json::from_slice(&out5).expect("do --json must produce valid JSON");
    assert_eq!(v5["schema_version"], 1, "do: schema_version must be 1");
}

// ── Scenario 4: Exit code contract ────────────────────────────────────────
//
// exit 0 = success, exit 1 = not found, exit 2 = validation error

#[test]
fn test_scenario_exit_code_contract() {
    let fixture = TestFixture::new();

    // success → exit 0
    fixture.cmd().args(["add", "valid task"]).assert().code(0);

    // non-existent ID → exit 1 (NotFound)
    fixture.cmd().args(["do", "9999"]).assert().code(1);

    // show non-existent ID → exit 1
    fixture.cmd().args(["show", "9999"]).assert().code(1);

    // invalid subcommand → clap exits 2
    fixture.cmd().args(["not-a-real-command"]).assert().code(2);
}

// ── Scenario 5: Enrichment pipeline ──────────────────────────────────────
//
// add → due → postpone → do → archive
// Validates cross-command state: enrichment metadata survives the full pipeline.

#[test]
fn test_scenario_enrichment_pipeline() {
    let (fixture, done_path) = fixture_with_done_file();

    // add a task
    fixture
        .cmd()
        .args(["add", "Prepare quarterly report"])
        .assert()
        .success();

    // Determine the task ID (it's the last line in the file)
    let content = fs::read_to_string(fixture.todo.path()).expect("read todo");
    let task_id = content.lines().count().to_string();

    // due {id} 2026-12-01 — set a due date
    fixture
        .cmd()
        .args(["due", &task_id, "2026-12-01"])
        .assert()
        .success();

    // Verify due date was added to the file
    let content2 = fs::read_to_string(fixture.todo.path()).expect("read after due");
    assert!(
        content2.contains("due:2026-12-01"),
        "task must have due:2026-12-01 after `due` command"
    );

    // postpone {id} 7 — advance by 7 days (2026-12-01 + 7 = 2026-12-08)
    fixture
        .cmd()
        .args(["postpone", &task_id, "7"])
        .assert()
        .success();

    let content3 = fs::read_to_string(fixture.todo.path()).expect("read after postpone");
    assert!(
        content3.contains("due:2026-12-08"),
        "due date must advance 7 days after postpone: {content3}"
    );

    // do {id} — complete the task
    fixture.cmd().args(["do", &task_id]).assert().success();

    // archive — task moves to done.txt
    fixture.cmd().args(["archive"]).assert().success();

    assert!(done_path.exists(), "done.txt must exist after archive");
    let done = fs::read_to_string(&done_path).expect("read done.txt");
    assert!(
        done.contains("Prepare quarterly report"),
        "completed task must be in done.txt: {done}"
    );
}

#[test]
fn recurring_cli_single_creates_next_occurrence_without_prompt() {
    let fixture = TestFixture::with_content("Pay rent rec:+1m due:2026-01-31\n");

    fixture.cmd().args(["do", "1"]).assert().success();

    let content = fs::read_to_string(fixture.todo.path()).expect("read todo");
    let lines: Vec<_> = content.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "expected completed task plus next occurrence"
    );
    assert!(
        lines[0].starts_with("x "),
        "original task must be completed: {}",
        lines[0]
    );
    assert!(lines[0].contains("Pay rent"));
    assert!(lines[1].contains("Pay rent"));
    assert!(lines[1].contains("rec:+1m"));
    assert!(
        lines[1].contains("due:2026-02-28"),
        "next monthly due date should be appended: {}",
        lines[1]
    );
    assert!(
        !lines[1].starts_with("x "),
        "next occurrence must be incomplete"
    );
}

#[test]
fn recurring_cli_multi_creates_one_next_occurrence_per_recurring_task() {
    let fixture = TestFixture::with_content(
        "Pay rent rec:+1m due:2026-01-31\n\
Stretch rec:1w due:2026-01-10\n\
Normal task\n\
x 2026-01-01 Already done rec:1d due:2026-01-01\n",
    );

    fixture
        .cmd()
        .args(["do", "1", "2", "3", "4"])
        .assert()
        .success();

    let content = fs::read_to_string(fixture.todo.path()).expect("read todo");
    let lines: Vec<_> = content.lines().collect();
    assert_eq!(
        lines.len(),
        6,
        "two recurring follow-ups should be appended"
    );
    assert!(
        lines[0].starts_with("x "),
        "first recurring task must be completed"
    );
    assert!(
        lines[1].starts_with("x "),
        "second recurring task must be completed"
    );
    assert!(
        lines[2].starts_with("x "),
        "non-recurring task must be completed"
    );
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.contains("Already done") && line.starts_with("x "))
            .count(),
        1,
        "already-completed recurring task must not duplicate"
    );
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.contains("Pay rent"))
            .count(),
        2,
        "pay rent should appear once completed and once as next occurrence"
    );
    assert_eq!(
        lines.iter().filter(|line| line.contains("Stretch")).count(),
        2,
        "stretch should appear once completed and once as next occurrence"
    );
}

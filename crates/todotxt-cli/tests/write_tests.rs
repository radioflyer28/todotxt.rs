mod helpers;

use helpers::TestFixture;
use predicates::str::contains;
use std::fs;
use toml::Value;

fn set_auto_creation_date(fixture: &TestFixture, enabled: bool) {
    let todo_path = fixture.todo.path().to_string_lossy().into_owned();
    let toml = format!(
        "todo_file = {}\nauto_creation_date = {}\n",
        Value::String(todo_path), enabled
    );
    fs::write(fixture.config.path(), toml).expect("rewrite config.toml");
}

// -- add tests (WRITE-01) -----------------------------------------------------

#[test]
fn add_creates_task_in_file() {
    let fixture = TestFixture::new();
    fixture
        .cmd()
        .args(["add", "Buy carrots +groceries"])
        .assert()
        .success()
        .stdout(contains("Buy carrots"));

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(content.contains("Buy carrots +groceries"));
}

#[test]
fn add_prints_new_task_id_to_stderr() {
    let fixture = TestFixture::new();
    fixture
        .cmd()
        .args(["add", "New task"])
        .assert()
        .success()
        .stderr(contains("Added task #5."));
}

#[test]
fn add_with_date_flag_prepends_creation_date() {
    let fixture = TestFixture::new();
    fixture
        .cmd()
        .args(["add", "--date", "Dated task"])
        .assert()
        .success()
        .stdout(contains("Dated task"));

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    let last_line = content.lines().last().unwrap_or("");
    assert!(last_line.starts_with("20") && last_line.contains('-'));
}

#[test]
fn add_with_no_date_flag_suppresses_date() {
    let fixture = TestFixture::new();
    set_auto_creation_date(&fixture, true);

    fixture
        .cmd()
        .args(["add", "--no-date", "No date task"])
        .assert()
        .success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    let last_line = content.lines().last().unwrap_or("");
    assert!(!last_line.starts_with("20") || !last_line.contains('-'));
}

#[test]
fn add_empty_text_exits_2() {
    let fixture = TestFixture::new();
    fixture.cmd().args(["add", ""]).assert().code(2);
}

#[test]
fn add_json_returns_task_envelope() {
    let fixture = TestFixture::new();
    fixture
        .cmd()
        .args(["--json", "add", "JSON task"])
        .assert()
        .success()
        .stdout(contains("schema_version"))
        .stdout(contains("JSON task"));
}

// -- do tests (WRITE-02) ------------------------------------------------------

#[test]
fn do_marks_task_complete() {
    let fixture = TestFixture::new();
    fixture
        .cmd()
        .args(["do", "1"])
        .assert()
        .success()
        .stdout(contains("Buy milk"));

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    let first_line = content.lines().next().unwrap_or("");
    assert!(first_line.starts_with("x "));
}

#[test]
fn do_idempotent_already_completed() {
    let fixture = TestFixture::new();
    fixture
        .cmd()
        .args(["do", "3"])
        .assert()
        .success()
        .stderr(contains("already completed"));
}

#[test]
fn do_invalid_id_exits_1() {
    let fixture = TestFixture::new();
    fixture.cmd().args(["do", "99"]).assert().code(1);
}

#[test]
fn do_json_returns_completed_task_envelope() {
    let fixture = TestFixture::new();
    fixture
        .cmd()
        .args(["--json", "do", "1"])
        .assert()
        .success()
        .stdout(contains("schema_version"))
        .stdout(contains("\"completed\":true"));
}

// -- undo tests (WRITE-03) ----------------------------------------------------

#[test]
fn undo_removes_completion_prefix() {
    let fixture = TestFixture::new();
    fixture
        .cmd()
        .args(["undo", "3"])
        .assert()
        .success()
        .stdout(contains("Done task"));

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    let third = lines.get(2).unwrap_or(&"");
    assert!(!third.starts_with("x "));
}

#[test]
fn undo_idempotent_already_incomplete() {
    let fixture = TestFixture::new();
    fixture
        .cmd()
        .args(["undo", "1"])
        .assert()
        .success()
        .stderr(contains("already incomplete"));
}

#[test]
fn undo_invalid_id_exits_1() {
    let fixture = TestFixture::new();
    fixture.cmd().args(["undo", "99"]).assert().code(1);
}

#[test]
fn undo_json_returns_task_envelope() {
    let fixture = TestFixture::new();
    fixture
        .cmd()
        .args(["--json", "undo", "3"])
        .assert()
        .success()
        .stdout(contains("schema_version"))
        .stdout(contains("\"completed\":false"));
}

// -- del tests (WRITE-04) -----------------------------------------------------

#[test]
fn del_removes_task_from_file() {
    let fixture = TestFixture::new();
    fixture
        .cmd()
        .args(["del", "2"])
        .assert()
        .success()
        .stdout(contains("Send report"));

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(!content.contains("Send report"));
}

#[test]
fn del_invalid_id_exits_1() {
    let fixture = TestFixture::new();
    fixture.cmd().args(["del", "99"]).assert().code(1);
}

#[test]
fn del_json_returns_deleted_task_envelope() {
    let fixture = TestFixture::new();
    fixture
        .cmd()
        .args(["--json", "del", "2"])
        .assert()
        .success()
        .stdout(contains("schema_version"))
        .stdout(contains("Send report"));
}

#[test]
fn del_multi_id_removes_all_specified_tasks() {
    let fixture = TestFixture::new();
    fixture.cmd().args(["del", "1", "2"]).assert().success();

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(!content.contains("Buy milk"));
    assert!(!content.contains("Send report"));
}

#[test]
fn del_one_invalid_id_aborts_all_deletions() {
    let fixture = TestFixture::new();
    fixture.cmd().args(["del", "1", "99"]).assert().code(1);

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(content.contains("Buy milk"));
}

// -- edit tests (WRITE-05) ----------------------------------------------------

#[test]
fn edit_replaces_task_text() {
    let fixture = TestFixture::new();
    fixture
        .cmd()
        .args(["edit", "1", "New task text +project"])
        .assert()
        .success()
        .stdout(contains("New task text"));

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(content.contains("New task text +project"));
    assert!(!content.contains("Buy milk"));
}

#[test]
fn edit_empty_text_exits_2() {
    let fixture = TestFixture::new();
    fixture.cmd().args(["edit", "1", ""]).assert().code(2);
}

#[test]
fn edit_invalid_id_exits_1() {
    let fixture = TestFixture::new();
    fixture.cmd().args(["edit", "99", "New text"]).assert().code(1);
}

#[test]
fn edit_json_returns_task_envelope() {
    let fixture = TestFixture::new();
    fixture
        .cmd()
        .args(["--json", "edit", "1", "JSON edited text"])
        .assert()
        .success()
        .stdout(contains("schema_version"))
        .stdout(contains("JSON edited text"));
}

// -- append tests (WRITE-06) --------------------------------------------------

#[test]
fn append_adds_text_to_end_of_task() {
    let fixture = TestFixture::new();
    fixture
        .cmd()
        .args(["append", "2", "due:2026-05-01"])
        .assert()
        .success()
        .stdout(contains("due:2026-05-01"));

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    assert!(content.contains("Send report") && content.contains("due:2026-05-01"));
}

#[test]
fn append_invalid_id_exits_1() {
    let fixture = TestFixture::new();
    fixture
        .cmd()
        .args(["append", "99", "extra text"])
        .assert()
        .code(1);
}

#[test]
fn append_json_returns_task_envelope() {
    let fixture = TestFixture::new();
    fixture
        .cmd()
        .args(["--json", "append", "2", "+json"])
        .assert()
        .success()
        .stdout(contains("schema_version"))
        .stdout(contains("+json"));
}

// -- prepend tests (WRITE-07) -------------------------------------------------

#[test]
fn prepend_inserts_text_before_body() {
    let fixture = TestFixture::new();
    fixture
        .cmd()
        .args(["prepend", "4", "(A)"])
        .assert()
        .success()
        .stdout(contains("Call dentist"));

    let content = fs::read_to_string(fixture.todo.path()).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    let fourth = lines.get(3).unwrap_or(&"");
    assert!(fourth.starts_with("(A)"));
}

#[test]
fn prepend_invalid_id_exits_1() {
    let fixture = TestFixture::new();
    fixture
        .cmd()
        .args(["prepend", "99", "prefix"])
        .assert()
        .code(1);
}

#[test]
fn prepend_json_returns_task_envelope() {
    let fixture = TestFixture::new();
    fixture
        .cmd()
        .args(["--json", "prepend", "4", "NOTE:"])
        .assert()
        .success()
        .stdout(contains("schema_version"))
        .stdout(contains("NOTE:"));
}

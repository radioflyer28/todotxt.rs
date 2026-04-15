use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

/// Integration test: CLI auto-creates config and succeeds with --todo-file override.
#[test]
fn config_auto_creates_with_todo_file() {
    let dir = assert_fs::TempDir::new().unwrap();
    let config_path = dir.child("config.toml");
    let todo_file = dir.child("todo.txt");
    todo_file.write_str("").unwrap();

    Command::cargo_bin("todotxt")
        .unwrap()
        .arg("--config")
        .arg(config_path.path())
        .arg("--todo-file")
        .arg(todo_file.path())
        .arg("list")
        .assert()
        .success();

    config_path.assert(predicate::path::exists());
}

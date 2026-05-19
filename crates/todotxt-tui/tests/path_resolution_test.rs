use std::path::PathBuf;

use todotxt_tui::config::{resolve_startup_paths, CliPathOverrides, TuiConfig};

fn path(value: &str) -> PathBuf {
    PathBuf::from(value)
}

#[test]
fn path_resolution_test_no_cli_flags_uses_config_values() {
    let mut config = TuiConfig::default();
    config.todo_file = Some(path("C:/cfg/todo.txt"));
    config.done_file = Some(path("C:/cfg/done.txt"));

    let overrides = CliPathOverrides::default();
    let resolved =
        resolve_startup_paths(&config, &overrides).expect("expected resolved startup paths");

    assert_eq!(resolved.todo_path, path("C:/cfg/todo.txt"));
    assert_eq!(resolved.archive_path, path("C:/cfg/done.txt"));
}

#[test]
fn path_resolution_test_cli_todo_overrides_config_todo() {
    let mut config = TuiConfig::default();
    config.todo_file = Some(path("C:/cfg/todo.txt"));
    config.done_file = Some(path("C:/cfg/done.txt"));

    let overrides = CliPathOverrides {
        todo: Some(path("C:/cli/work.txt")),
        archive: None,
    };
    let resolved =
        resolve_startup_paths(&config, &overrides).expect("expected resolved startup paths");

    assert_eq!(resolved.todo_path, path("C:/cli/work.txt"));
}

#[test]
fn path_resolution_test_cli_todo_without_archive_defaults_archive_to_todo_sibling_done() {
    let mut config = TuiConfig::default();
    config.todo_file = Some(path("C:/cfg/todo.txt"));
    config.done_file = Some(path("C:/cfg/done.txt"));

    let overrides = CliPathOverrides {
        todo: Some(path("C:/alt/inbox.txt")),
        archive: None,
    };
    let resolved =
        resolve_startup_paths(&config, &overrides).expect("expected resolved startup paths");

    assert_eq!(resolved.todo_path, path("C:/alt/inbox.txt"));
    assert_eq!(resolved.archive_path, path("C:/alt/done.txt"));
}

#[test]
fn path_resolution_test_cli_archive_overrides_config_archive() {
    let mut config = TuiConfig::default();
    config.todo_file = Some(path("C:/cfg/todo.txt"));
    config.done_file = Some(path("C:/cfg/done.txt"));

    let overrides = CliPathOverrides {
        todo: None,
        archive: Some(path("C:/cli/archive.txt")),
    };
    let resolved =
        resolve_startup_paths(&config, &overrides).expect("expected resolved startup paths");

    assert_eq!(resolved.todo_path, path("C:/cfg/todo.txt"));
    assert_eq!(resolved.archive_path, path("C:/cli/archive.txt"));
}

#[test]
fn path_resolution_test_cli_todo_and_archive_are_used_exactly() {
    let mut config = TuiConfig::default();
    config.todo_file = Some(path("C:/cfg/todo.txt"));
    config.done_file = Some(path("C:/cfg/done.txt"));

    let overrides = CliPathOverrides {
        todo: Some(path("C:/cli/tasks.txt")),
        archive: Some(path("C:/cli/completed.txt")),
    };
    let resolved =
        resolve_startup_paths(&config, &overrides).expect("expected resolved startup paths");

    assert_eq!(resolved.todo_path, path("C:/cli/tasks.txt"));
    assert_eq!(resolved.archive_path, path("C:/cli/completed.txt"));
}

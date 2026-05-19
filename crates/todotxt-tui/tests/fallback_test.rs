use std::io::Write;

use tempfile::NamedTempFile;
use todotxt_core::TaskList;
use todotxt_tui::app::App;
use todotxt_tui::config::TuiConfig;
use todotxt_tui::state::{DisplayRow, Pane};
use todotxt_tui::theme::Theme;

fn make_app_with_lines(lines: &[&str]) -> App {
    let mut file = NamedTempFile::new().expect("failed to create temp file");
    for line in lines {
        writeln!(file, "{}", line).expect("failed to write todo line");
    }
    file.flush().expect("failed to flush temp file");
    let path = file.path().to_path_buf();
    let task_list = TaskList::load(&path).expect("failed to load task list");

    App::new(
        task_list,
        path,
        TuiConfig::default(),
        None,
        Theme::Default,
        true,
    )
}

#[test]
fn test_single_pane_mode_with_empty_panes() {
    let mut app = make_app_with_lines(&["task A"]);
    app.panes.clear();

    assert!(app.should_show_single_pane());
}

#[test]
fn test_single_pane_mode_with_one_pane() {
    let app = make_app_with_lines(&["task A"]);

    assert!(app.should_show_single_pane());
}

#[test]
fn test_single_pane_mode_with_all_empty() {
    let mut app = make_app_with_lines(&[]);
    app.panes.push(Pane::new(1, "Empty1".to_string()));
    app.panes.push(Pane::new(2, "Empty2".to_string()));

    assert!(app.should_show_single_pane());
}

#[test]
fn test_multi_pane_mode_with_populated_panes() {
    let mut app = make_app_with_lines(&["task A", "task B"]);
    app.panes.push(Pane::new(1, "Work".to_string()));

    assert!(!app.should_show_single_pane());
}

#[test]
fn test_reconcile_empty_panes() {
    let mut app = make_app_with_lines(&["task A"]);
    app.panes.clear();
    app.active_pane = 5;

    app.reconcile_active_pane();

    assert_eq!(app.panes.len(), 1);
    assert_eq!(app.active_pane, 0);
}

#[test]
fn test_reconcile_out_of_bounds_active_pane() {
    let mut app = make_app_with_lines(&["task A"]);
    app.panes.push(Pane::new(1, "Work".to_string()));
    app.active_pane = 10;

    app.reconcile_active_pane();

    assert_eq!(app.active_pane, 1);
}

#[test]
fn test_display_rows_fallback() {
    let mut app = make_app_with_lines(&["task A"]);
    app.panes[0].display_rows = vec![DisplayRow::Task(0)];

    let rows = app.display_rows();
    assert_eq!(rows.len(), 1);
}

#[test]
fn test_display_rows_multi_pane() {
    let mut app = make_app_with_lines(&["task A", "task B", "task C"]);
    app.panes.push(Pane::new(1, "Work".to_string()));
    app.panes[1].display_rows = vec![DisplayRow::Task(1), DisplayRow::Task(2)];
    app.active_pane = 1;

    let rows = app.display_rows();
    assert_eq!(rows.len(), 2);
}

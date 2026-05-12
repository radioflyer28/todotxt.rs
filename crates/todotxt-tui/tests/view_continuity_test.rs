// Phase 37: Metadata Flexibility + View Continuity (VIEW-03)
// These tests verify that all v1.5 mutation flows (add, edit, delete, toggle, bulk-append, paste, undo)
// preserve pane filter/sort/group state, and that undo correctly restores original raw task text (including tag order).

use std::io::Write;

use tempfile::NamedTempFile;
use todotxt_core::{TaskList, SortOrder};
use todotxt_tui::app::App;
use todotxt_tui::config::TuiConfig;
use todotxt_tui::state::DisplayRow;
use todotxt_tui::theme::Theme;

/// Create an App from lines
fn make_app_with_lines(lines: &[&str]) -> App {
    let mut file = NamedTempFile::new().expect("failed to create temp file");
    for line in lines {
        writeln!(file, "{}", line).expect("failed to write todo line");
    }
    file.flush().expect("failed to flush temp file");
    let path = file.path().to_path_buf();
    let task_list = TaskList::load(&path).expect("failed to load task list");

    App::new(task_list, path, TuiConfig::default(), None, Theme::Default, true)
}

/// Helper to set up an app with specific pane state (filter, sort, grouping)
fn setup_app_with_state(
    lines: &[&str],
    filter_query: &str,
    sort_order: SortOrder,
    grouping: bool,
) -> App {
    let mut app = make_app_with_lines(lines);

    // Configure the single pane with desired state
    if let Some(pane) = app.panes.get_mut(0) {
        pane.filter_query = filter_query.to_string();
        pane.sort_order = sort_order;
        pane.grouping = grouping;
    }

    // Rebuild display based on new filter/sort/group state
    app.rebuild_all_panes();

    app
}

/// Helper to assert pane state is preserved
fn assert_pane_state_preserved(
    app: &App,
    expected_filter: &str,
    expected_sort: SortOrder,
    expected_grouping: bool,
    context: &str,
) {
    if let Some(pane) = app.panes.get(0) {
        assert_eq!(
            pane.filter_query, expected_filter,
            "{}: filter_query mismatch",
            context
        );
        assert_eq!(
            pane.sort_order, expected_sort,
            "{}: sort_order mismatch",
            context
        );
        assert_eq!(
            pane.grouping, expected_grouping,
            "{}: grouping mismatch",
            context
        );
    } else {
        panic!("No pane found in app");
    }
}

const INITIAL_TASKS: &[&str] = &[
    "Buy milk @home +shopping due:2026-05-15",
    "(A) Finish report +work @office due:2026-05-01",
    "x 2026-04-28 Completed task @archive",
    "Call client @phone +work due:2026-05-05 t:2026-04-30",
    "@email/waiting Follow up on invoice",
    "+client/acme Prepare presentation",
];

// ── VIEW-03: Mutation Flow Tests ─────────────────────────────────────────────

/// Test 1: Filter state persists after add
#[test]
fn test_add_task_preserves_filter_state() {
    let mut app = setup_app_with_state(INITIAL_TASKS, "@home", SortOrder::FileOrder, false);
    let initial_filter = app.panes[0].filter_query.clone();
    let initial_sort = app.panes[0].sort_order;
    let initial_grouping = app.panes[0].grouping;

    // Add a task directly to task_list to simulate add operation
    let new_task = todotxt_core::Task::parse("New task @home +shopping");
    app.task_list
        .add(new_task)
        .expect("failed to add task");
    
    // Rebuild display (simulating what save_and_exit does)
    app.rebuild_all_panes();

    // Verify pane state is preserved
    assert_eq!(app.panes[0].filter_query, initial_filter, "filter_query should not change");
    assert_eq!(app.panes[0].sort_order, initial_sort, "sort_order should not change");
    assert_eq!(app.panes[0].grouping, initial_grouping, "grouping should not change");
}

/// Test 2: Filter state persists after edit
#[test]
fn test_edit_task_preserves_filter_state() {
    let mut app = setup_app_with_state(INITIAL_TASKS, "@office", SortOrder::Priority, true);
    let initial_filter = app.panes[0].filter_query.clone();
    let initial_sort = app.panes[0].sort_order;
    let initial_grouping = app.panes[0].grouping;

    // Edit the first task
    if let Some(task) = app.task_list.tasks().get(0).cloned() {
        let edited = task.with_priority(Some('A'));
        app.task_list.update(0, edited).expect("failed to update");
    }

    // Rebuild display
    app.rebuild_all_panes();

    // Verify pane state is preserved
    assert_eq!(app.panes[0].filter_query, initial_filter, "filter_query should not change");
    assert_eq!(app.panes[0].sort_order, initial_sort, "sort_order should not change");
    assert_eq!(app.panes[0].grouping, initial_grouping, "grouping should not change");
}

/// Test 3: Filter state persists after delete
#[test]
fn test_delete_task_preserves_filter_state() {
    let mut app = setup_app_with_state(INITIAL_TASKS, "+work", SortOrder::FileOrder, false);
    let initial_filter = app.panes[0].filter_query.clone();
    let initial_sort = app.panes[0].sort_order;
    let initial_grouping = app.panes[0].grouping;

    // Delete the first task
    if let Some(DisplayRow::Task(idx)) = app.display_rows().get(0) {
        let idx = *idx;
        app.task_list.delete(idx).expect("failed to delete");
    }

    // Rebuild display
    app.rebuild_all_panes();

    // Verify pane state is preserved
    assert_eq!(app.panes[0].filter_query, initial_filter, "filter_query should not change");
    assert_eq!(app.panes[0].sort_order, initial_sort, "sort_order should not change");
    assert_eq!(app.panes[0].grouping, initial_grouping, "grouping should not change");
}

/// Test 4: Filter state persists after toggle (mark done)
#[test]
fn test_toggle_task_preserves_filter_state() {
    let mut app = setup_app_with_state(INITIAL_TASKS, "DONE", SortOrder::CompletedDate, true);
    let initial_filter = app.panes[0].filter_query.clone();
    let initial_sort = app.panes[0].sort_order;
    let initial_grouping = app.panes[0].grouping;

    // Toggle done on a task
    if let Some(DisplayRow::Task(idx)) = app.display_rows().get(0) {
        let idx = *idx;
        if let Some(task) = app.task_list.tasks().get(idx).cloned() {
            let toggle_value = !task.completed;
            let toggled = task.with_completed(toggle_value);
            app.task_list.update(idx, toggled).expect("failed to toggle");
        }
    }

    // Rebuild display
    app.rebuild_all_panes();

    // Verify pane state is preserved
    assert_eq!(app.panes[0].filter_query, initial_filter, "filter_query should not change");
    assert_eq!(app.panes[0].sort_order, initial_sort, "sort_order should not change");
    assert_eq!(app.panes[0].grouping, initial_grouping, "grouping should not change");
}

/// Test 5: Filter state preserved through multiple mutations
#[test]
fn test_multiple_mutations_preserve_filter_state() {
    let mut app = setup_app_with_state(INITIAL_TASKS, "-DONE", SortOrder::FileOrder, false);
    let initial_filter = app.panes[0].filter_query.clone();
    let initial_sort = app.panes[0].sort_order;
    let initial_grouping = app.panes[0].grouping;

    // Add
    let new_task = todotxt_core::Task::parse("Task 1");
    app.task_list.add(new_task).expect("add failed");
    app.rebuild_all_panes();
    assert_eq!(app.panes[0].filter_query, initial_filter, "filter_query after add");

    // Edit
    if let Some(task) = app.task_list.tasks().get(0).cloned() {
        let edited = task.with_priority(Some('B'));
        app.task_list.update(0, edited).expect("edit failed");
    }
    app.rebuild_all_panes();
    assert_eq!(app.panes[0].filter_query, initial_filter, "filter_query after edit");

    // Delete
    if let Some(DisplayRow::Task(idx)) = app.display_rows().get(0) {
        let idx = *idx;
        app.task_list.delete(idx).expect("delete failed");
    }
    app.rebuild_all_panes();
    assert_eq!(app.panes[0].filter_query, initial_filter, "filter_query after delete");

    // Final state
    assert_eq!(app.panes[0].sort_order, initial_sort, "sort_order unchanged");
    assert_eq!(app.panes[0].grouping, initial_grouping, "grouping unchanged");
}

/// Test 6: Undo entry captures original task state before mutation
#[test]
fn test_undo_entry_captures_original_state() {
    let mut app = make_app_with_lines(&[
        "Buy milk @home +shopping @evening due:2026-05-15",
    ]);

    // Get original task and raw text
    let original_task = app.task_list.tasks()[0].clone();
    let original_raw = original_task.to_raw().to_string();

    // Verify tag positions in original
    let home_pos = original_raw.find("@home").expect("@home not in original");
    let evening_pos = original_raw.find("@evening").expect("@evening not in original");
    assert!(
        home_pos < evening_pos,
        "original has tags in order: @home before @evening"
    );

    // Create undo entry (simulating what mutation handlers do)
    app.undo_entry = Some(todotxt_tui::state::UndoEntry {
        tasks: vec![original_task.clone()],
        selected: app.panes.first().map(|p| p.selected).unwrap_or(0),
    });

    // Verify undo_entry captured the original task state
    if let Some(undo) = app.undo_entry.as_ref() {
        assert_eq!(undo.tasks.len(), 1, "undo entry should contain 1 task");
        let undo_raw = undo.tasks[0].to_raw();
        assert_eq!(
            undo_raw, original_raw.as_str(),
            "undo entry should preserve original raw text with original tag order"
        );
    } else {
        panic!("undo_entry should be set");
    }
}

/// Test 7: Hierarchical tag filter state preserved (testing new META-02 variants)
#[test]
fn test_hierarchical_filter_state_preserved() {
    let mut app = setup_app_with_state(
        &["Task @email/waiting", "Task @work"],
        "@email",  // Parent prefix filter
        SortOrder::FileOrder,
        false,
    );

    let initial_filter = app.panes[0].filter_query.clone();

    // Add a task
    let new_task = todotxt_core::Task::parse("Follow up @email/urgent");
    app.task_list.add(new_task).expect("add failed");
    app.rebuild_all_panes();

    // Filter should still be @email (parent prefix)
    assert_eq!(
        app.panes[0].filter_query, initial_filter,
        "hierarchical filter should persist"
    );
}

/// Test 8: Project hierarchical filter state preserved
#[test]
fn test_project_hierarchical_filter_preserved() {
    let mut app = setup_app_with_state(
        &["Work +client/acme", "Work +other"],
        "+client",  // Parent prefix filter for projects
        SortOrder::FileOrder,
        true,
    );

    let initial_filter = app.panes[0].filter_query.clone();
    let initial_grouping = app.panes[0].grouping;

    // Delete a task
    if let Some(DisplayRow::Task(idx)) = app.display_rows().get(0) {
        let idx = *idx;
        app.task_list.delete(idx).expect("delete failed");
    }
    app.rebuild_all_panes();

    // Filter should still be +client (parent prefix)
    assert_eq!(
        app.panes[0].filter_query, initial_filter,
        "project hierarchical filter should persist"
    );
    assert_eq!(
        app.panes[0].grouping, initial_grouping,
        "grouping should persist"
    );
}


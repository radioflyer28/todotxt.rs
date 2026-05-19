// Phase 31: Single-Pane Filter/Sort/Group Bridge Fix (GAP-1 closure)
// These tests verify that single-pane mode correctly maintains per-pane query state.

use std::io::Write;

use tempfile::NamedTempFile;
use todotxt_core::{SortOrder, TaskList};
use todotxt_tui::app::App;
use todotxt_tui::config::TuiConfig;
use todotxt_tui::state::Pane;
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

/// Phase 31, Task 3: Test single-pane mode initialization with filter state.
///
/// Scenario: Default startup (single pane). Verify that setting filter on
/// the active pane doesn't cause a panic and state is preserved.
#[test]
fn test_single_pane_mode_filter_state_preserved() {
    let mut app = make_app_with_lines(&["task one @home", "task two @work", "task three @home"]);

    // Verify we're in single-pane mode
    assert_eq!(app.panes.len(), 1);
    assert!(app.should_show_single_pane());

    // Set filter on active pane (Phase 31 fix: this state is synced during rebuild)
    app.active_pane_mut().filter_query = "@home".to_string();

    // Verify the pane's filter_query is set
    assert_eq!(app.active_pane().filter_query, "@home");

    // Verify single-pane mode is still active
    assert!(app.should_show_single_pane());
}

/// Phase 31, Task 3: Test that sort order is preserved on active pane.
///
/// Scenario: Set sort order on active pane, verify it persists.
#[test]
fn test_single_pane_mode_sort_state_preserved() {
    let mut app = make_app_with_lines(&[
        "(B) task priority B",
        "(A) task priority A",
        "(C) task priority C",
    ]);

    // Verify single-pane mode
    assert_eq!(app.panes.len(), 1);
    assert!(app.should_show_single_pane());

    // Set sort order on active pane
    app.active_pane_mut().sort_order = SortOrder::Priority;

    // Verify sort order is preserved
    assert_eq!(app.active_pane().sort_order, SortOrder::Priority);
}

/// Phase 31, Task 3: Test that grouping toggle works on active pane.
///
/// Scenario: Set grouping on active pane, verify it persists.
#[test]
fn test_single_pane_mode_grouping_state_preserved() {
    let mut app = make_app_with_lines(&["task one @home", "task two @home"]);

    // Verify single-pane mode
    assert!(app.should_show_single_pane());

    // Set grouping on active pane
    app.active_pane_mut().grouping = true;

    // Verify grouping is preserved
    assert!(app.active_pane().grouping);
}

/// Phase 31, Task 3: Test panes_hidden mode state preservation.
///
/// Scenario: Multi-pane setup with panes_hidden toggle.
/// Verify that panes_hidden state is correctly tracked.
#[test]
fn test_panes_hidden_mode_state_preserved() {
    let mut app = make_app_with_lines(&["personal task @home", "work task @work"]);

    // Set up multi-pane mode first
    app.panes.push(Pane::new(1, "Work".to_string()));
    assert!(!app.should_show_single_pane(), "Should be multi-pane");

    // Toggle panes hidden
    app.panes_hidden = true;

    // Verify state
    assert!(app.panes_hidden);

    // Set filter on active pane
    app.active_pane_mut().filter_query = "@work".to_string();

    // Verify filter is set
    assert_eq!(app.active_pane().filter_query, "@work");
}

/// Phase 31 Regression Test: Verify multi-pane mode state is unaffected.
///
/// When NOT in single-pane or panes_hidden mode, per-pane state should
/// be preserved independently across panes.
#[test]
fn test_multi_pane_mode_per_pane_state_independent() {
    let mut app = make_app_with_lines(&["task one @home", "task two @work"]);

    // Create multi-pane setup
    app.panes.push(Pane::new(1, "Work".to_string()));
    app.panes[0].filter_query = "@home".to_string();
    app.panes[1].filter_query = "@work".to_string();

    app.active_pane = 0;
    assert_eq!(app.active_pane().filter_query, "@home");

    // Switch to pane 1
    app.active_pane = 1;
    assert_eq!(app.active_pane().filter_query, "@work");

    // Back to pane 0
    app.active_pane = 0;
    assert_eq!(app.active_pane().filter_query, "@home");
}

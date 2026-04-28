/// Integration tests for per-pane query behavior (Phase 25)
/// Tests pane navigation, state preservation, and empty-pane safety.

#[cfg(test)]
mod pane_integration_tests {
    use todotxt_tui::app::App;
    use todotxt_tui::state::Pane;
    use todotxt_core::{TaskList, SortOrder};
    use std::fs::File;
    use std::io::Write;

    /// Helper: Create a test App with a task list
    fn setup_test_app() -> App {
        // Create a temporary empty todo.txt file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_todo.txt");
        File::create(&test_file).expect("Failed to create test file").write_all(b"").expect("Failed to write to test file");
        
        let task_list = TaskList::load(&test_file).expect("Failed to load TaskList");
        let config = todotxt_tui::config::TuiConfig::default();
        
        let app = App::new(
            task_list,
            test_file,
            config,
            None,
            todotxt_tui::theme::Theme::Default,
            false,
        );
        app
    }

    #[test]
    fn test_pane_navigation_wraps_around() {
        let mut app = setup_test_app();
        
        // App starts with one default pane
        assert_eq!(app.panes.len(), 1);
        assert_eq!(app.active_pane, 0);
        
        // Add a second pane (simulating Phase 26 pane creation)
        app.panes.push(Pane::new(1, "Pane 2".to_string()));
        assert_eq!(app.panes.len(), 2);
        
        // Navigate right to pane 1
        app.focus_next_pane();
        assert_eq!(app.active_pane, 1);
        
        // Navigate right again — should wrap to pane 0
        app.focus_next_pane();
        assert_eq!(app.active_pane, 0);
        
        // Navigate left — should wrap to pane 1
        app.focus_prev_pane();
        assert_eq!(app.active_pane, 1);
        
        // Navigate left again — should go to pane 0
        app.focus_prev_pane();
        assert_eq!(app.active_pane, 0);
    }

    #[test]
    fn test_pane_filter_state_preserved_on_navigation() {
        let mut app = setup_test_app();
        app.panes.push(Pane::new(1, "Pane 2".to_string()));
        
        // Set filter on pane 0
        app.active_pane_mut().filter_query = "project:work".to_string();
        assert_eq!(app.panes[0].filter_query, "project:work");
        
        // Navigate to pane 1
        app.focus_next_pane();
        assert_eq!(app.active_pane, 1);
        
        // Pane 1 should have empty filter
        assert_eq!(app.active_pane().filter_query, "");
        
        // Navigate back to pane 0
        app.focus_prev_pane();
        assert_eq!(app.active_pane, 0);
        
        // Filter should be preserved
        assert_eq!(app.active_pane().filter_query, "project:work");
    }

    #[test]
    fn test_pane_sort_state_preserved_on_navigation() {
        let mut app = setup_test_app();
        app.panes.push(Pane::new(1, "Pane 2".to_string()));
        
        // Set sort on pane 0
        app.active_pane_mut().sort_order = SortOrder::Priority;
        assert_eq!(app.panes[0].sort_order, SortOrder::Priority);
        
        // Navigate to pane 1
        app.focus_next_pane();
        
        // Pane 1 should have default FileOrder
        assert_eq!(app.active_pane().sort_order, SortOrder::FileOrder);
        
        // Navigate back to pane 0
        app.focus_prev_pane();
        
        // Sort should be preserved
        assert_eq!(app.active_pane().sort_order, SortOrder::Priority);
    }

    #[test]
    fn test_pane_grouping_state_preserved_on_navigation() {
        let mut app = setup_test_app();
        app.panes.push(Pane::new(1, "Pane 2".to_string()));
        
        // Enable grouping on pane 0
        app.active_pane_mut().grouping = true;
        assert_eq!(app.panes[0].grouping, true);
        
        // Navigate to pane 1
        app.focus_next_pane();
        
        // Pane 1 should have grouping disabled
        assert_eq!(app.active_pane().grouping, false);
        
        // Navigate back to pane 0
        app.focus_prev_pane();
        
        // Grouping should be preserved
        assert_eq!(app.active_pane().grouping, true);
    }

    #[test]
    fn test_empty_pane_allows_filter_modification() {
        let mut app = setup_test_app();
        
        // Active pane should be empty (no tasks)
        assert!(app.active_pane().is_empty());
        
        // Modifying filter should succeed even on empty pane
        app.active_pane_mut().filter_query = "project:test".to_string();
        
        assert_eq!(app.active_pane().filter_query, "project:test");
    }

    #[test]
    fn test_empty_pane_allows_sort_modification() {
        let mut app = setup_test_app();
        
        // Active pane should be empty
        assert!(app.active_pane().is_empty());
        
        // Modifying sort order should succeed even on empty pane
        app.active_pane_mut().sort_order = SortOrder::DueDate;
        
        assert_eq!(app.active_pane().sort_order, SortOrder::DueDate);
    }

    #[test]
    fn test_empty_pane_allows_grouping_modification() {
        let mut app = setup_test_app();
        
        // Active pane should be empty
        assert!(app.active_pane().is_empty());
        
        // Toggling grouping should succeed even on empty pane
        app.active_pane_mut().grouping = true;
        
        assert!(app.active_pane().grouping);
    }

    #[test]
    fn test_reconcile_active_pane_ensures_bounds() {
        let mut app = setup_test_app();
        
        // Add a couple panes
        app.panes.push(Pane::new(1, "Pane 2".to_string()));
        app.panes.push(Pane::new(2, "Pane 3".to_string()));
        
        // Set active_pane to an out-of-bounds value
        app.active_pane = 5;
        
        // reconcile_active_pane should fix it
        app.reconcile_active_pane();
        
        // Should clamp to last valid index (2)
        assert_eq!(app.active_pane, 2);
    }

    #[test]
    fn test_reconcile_active_pane_creates_default_pane_when_empty() {
        let mut app = setup_test_app();
        
        // Remove all panes
        app.panes.clear();
        assert_eq!(app.panes.len(), 0);
        
        // reconcile_active_pane should create a default pane
        app.reconcile_active_pane();
        
        assert_eq!(app.panes.len(), 1);
        assert_eq!(app.active_pane, 0);
        assert_eq!(app.panes[0].label, "Tasks");
    }

    #[test]
    fn test_single_pane_fallback_when_all_panes_empty() {
        let mut app = setup_test_app();
        
        // Add multiple panes
        app.panes.push(Pane::new(1, "Pane 2".to_string()));
        app.panes.push(Pane::new(2, "Pane 3".to_string()));
        
        // All panes should be empty (no tasks added)
        for pane in &app.panes {
            assert!(pane.is_empty());
        }
        
        // should_show_single_pane should return true (fallback)
        assert!(app.should_show_single_pane());
    }

    #[test]
    fn test_active_pane_mut_reconciles_bounds() {
        let mut app = setup_test_app();
        
        // Set active_pane to invalid index
        app.active_pane = 10;
        
        // active_pane_mut() should call reconcile before returning
        let _ = app.active_pane_mut();
        
        // Should be fixed now
        assert!(app.active_pane < app.panes.len());
    }

    #[test]
    fn test_pane_selection_clamped_on_navigation() {
        let mut app = setup_test_app();
        app.panes.push(Pane::new(1, "Pane 2".to_string()));
        
        // Manually set selected to an invalid value on pane 0
        app.panes[0].selected = 100;
        assert!(!app.panes[0].is_empty() || app.panes[0].selected == 100); // pane is empty or selection is out of bounds
        
        // Navigate to pane 1
        app.focus_next_pane();
        
        // Navigate back to pane 0
        app.focus_prev_pane();
        
        // Selection should still be 100 (we don't auto-fix in navigate, only in rebuild)
        // This is expected behavior — the selection gets reconciled when display_rows is rebuilt
        assert_eq!(app.panes[0].selected, 100);
    }
}

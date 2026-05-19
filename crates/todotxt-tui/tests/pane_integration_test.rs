/// Integration tests for per-pane query behavior (Phase 25)
/// Tests pane navigation, state preservation, and empty-pane safety.

#[cfg(test)]
mod pane_integration_tests {
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    use todotxt_core::{SortOrder, TaskList};
    use todotxt_tui::app::App;
    use todotxt_tui::config::{PaneConfig, PaneSort, TuiConfig, TuiStateFile};
    use todotxt_tui::state::Pane;

    /// Helper: Create a test App with a task list
    fn setup_test_app() -> App {
        // Create a temporary empty todo.txt file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_todo.txt");
        File::create(&test_file)
            .expect("Failed to create test file")
            .write_all(b"")
            .expect("Failed to write to test file");

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

    fn unique_temp_file(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        std::env::temp_dir().join(format!("{}_{}_{}.toml", name, std::process::id(), nanos))
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
        assert_eq!(app.panes[0].label, "");
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

    #[test]
    fn test_startup_bootstrap_uses_configured_panes() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_todo_bootstrap.txt");
        File::create(&test_file)
            .expect("Failed to create test file")
            .write_all(b"")
            .expect("Failed to write to test file");
        let task_list = TaskList::load(&test_file).expect("Failed to load TaskList");

        let mut config = TuiConfig::default();
        config.panes = vec![
            PaneConfig {
                label: "Work".to_string(),
                filter: "project:work".to_string(),
                sort: PaneSort::Priority,
                group: true,
                group_by: None,
            },
            PaneConfig {
                label: "Today".to_string(),
                filter: "due:today".to_string(),
                sort: PaneSort::DueDate,
                group: false,
                group_by: None,
            },
        ];

        let app = App::new(
            task_list,
            test_file,
            config,
            None,
            todotxt_tui::theme::Theme::Default,
            false,
        );

        assert_eq!(app.panes.len(), 2);
        assert_eq!(app.panes[0].label, "Work");
        assert_eq!(app.panes[0].filter_query, "project:work");
        assert_eq!(app.panes[0].sort_order, SortOrder::Priority);
        assert!(app.panes[0].grouping);

        assert_eq!(app.panes[1].label, "Today");
        assert_eq!(app.panes[1].filter_query, "due:today");
        assert_eq!(app.panes[1].sort_order, SortOrder::DueDate);
        assert!(!app.panes[1].grouping);
    }

    #[test]
    fn test_invalid_pane_entries_are_skipped_and_valid_ones_load() {
        let config_path = unique_temp_file("pane_config_invalid_skip");
        let config_toml = r#"
[[panes]]
label = "Valid"
sort = "priority"
group = true

[[panes]]
label = "Invalid"
sort = "nope"
group = false
"#;
        fs::write(&config_path, config_toml).expect("should write config");

        let config =
            TuiConfig::load(&config_path).expect("load should tolerate invalid pane entries");
        assert_eq!(config.panes.len(), 1);
        assert_eq!(config.panes[0].label, "Valid");
        assert_eq!(config.panes[0].sort, PaneSort::Priority);

        let _ = fs::remove_file(&config_path);
    }

    #[test]
    fn test_invalid_only_config_still_keeps_runtime_safe_default_pane() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_todo_invalid_only.txt");
        File::create(&test_file)
            .expect("Failed to create test file")
            .write_all(b"")
            .expect("Failed to write to test file");

        let mut config = TuiConfig::default();
        config.panes = vec![];
        let app = App::new(
            TaskList::load(&test_file).expect("Failed to load TaskList"),
            test_file,
            config,
            None,
            todotxt_tui::theme::Theme::Default,
            false,
        );

        assert_eq!(app.panes.len(), 1);
        assert_eq!(app.active_pane, 0);
        assert_eq!(app.panes[0].label, "");
    }

    #[test]
    fn test_quit_persists_runtime_panes_into_config() {
        let todo_path = unique_temp_file("pane_persist_todo");
        File::create(&todo_path)
            .expect("Failed to create todo file")
            .write_all(b"")
            .expect("Failed to write todo file");

        let config_path = unique_temp_file("pane_persist_config");
        let initial_config = r#"
todo_file = "tasks.txt"
normalize_append = false
"#;
        fs::write(&config_path, initial_config).expect("should write initial config");

        let mut config = TuiConfig::load(&config_path).expect("should load config");
        config.todo_file = Some(todo_path.clone());

        let mut app = App::new(
            TaskList::load(&todo_path).expect("Failed to load TaskList"),
            todo_path,
            config,
            Some(config_path.clone()),
            todotxt_tui::theme::Theme::Default,
            false,
        );

        app.panes = vec![
            Pane::new(0, "Work".to_string()),
            Pane::new(1, "Today".to_string()),
        ];
        app.panes[0].filter_query = "project:work".to_string();
        app.panes[0].sort_order = SortOrder::Priority;
        app.panes[0].grouping = true;

        app.panes[1].filter_query = "due:today".to_string();
        app.panes[1].sort_order = SortOrder::DueDate;
        app.panes[1].grouping = false;

        app.save_view_state()
            .expect("quit persistence should succeed");

        let state_path = todotxt_tui::config::state_file_path(&config_path);
        let reloaded =
            TuiStateFile::load(&state_path).expect("state file should be written and parseable");
        assert_eq!(reloaded.panes.len(), 2);
        assert_eq!(reloaded.panes[0].label, "Work");
        assert_eq!(reloaded.panes[0].filter, "project:work");
        assert_eq!(reloaded.panes[0].sort, PaneSort::Priority);
        assert!(reloaded.panes[0].group);

        assert_eq!(reloaded.panes[1].label, "Today");
        assert_eq!(reloaded.panes[1].filter, "due:today");
        assert_eq!(reloaded.panes[1].sort, PaneSort::DueDate);
        assert!(!reloaded.panes[1].group);

        // config.toml must NOT be rewritten at runtime (PRSV-03).
        let config_contents =
            fs::read_to_string(&config_path).expect("config.toml should still exist");
        assert!(
            config_contents.contains("normalize_append"),
            "config.toml should be preserved unchanged"
        );

        let _ = fs::remove_file(&config_path);
        let _ = fs::remove_file(&state_path);
    }

    #[test]
    fn test_persisted_pane_data_contains_only_config_fields() {
        let dir = tempfile::tempdir().expect("temp dir must be creatable");
        let todo_path = dir.path().join("todo.txt");
        let config_path = dir.path().join("config.toml");
        File::create(&todo_path)
            .expect("Failed to create todo file")
            .write_all(b"")
            .expect("Failed to write todo file");

        fs::write(&config_path, "").expect("should create config file");

        let mut app = App::new(
            TaskList::load(&todo_path).expect("Failed to load TaskList"),
            todo_path,
            TuiConfig::default(),
            Some(config_path.clone()),
            todotxt_tui::theme::Theme::Default,
            false,
        );

        app.panes = vec![Pane::new(42, "Persist Me".to_string())];
        app.panes[0].selected = 99;
        app.panes[0].filter_query = "@home".to_string();
        app.panes[0].sort_order = SortOrder::Alphabetical;
        app.panes[0].grouping = true;

        app.save_view_state()
            .expect("quit persistence should succeed");

        let state_path = todotxt_tui::config::state_file_path(&config_path);
        let persisted = fs::read_to_string(&state_path).expect("should read persisted state file");
        assert!(persisted.contains("label = \"Persist Me\""));
        assert!(persisted.contains("filter = \"@home\""));
        assert!(persisted.contains("sort = \"alphabetical\""));
        assert!(persisted.contains("group = true"));
        assert!(!persisted.contains("id ="));
        assert!(!persisted.contains("selected ="));
        assert!(!persisted.contains("display_rows"));
        // dir drops here, cleaning up all temp files automatically
    }

    #[test]
    fn test_no_pane_write_occurs_until_quit_persist_path() {
        let dir = tempfile::tempdir().expect("temp dir must be creatable");
        let todo_path = dir.path().join("todo.txt");
        let config_path = dir.path().join("config.toml");
        File::create(&todo_path)
            .expect("Failed to create todo file")
            .write_all(b"")
            .expect("Failed to write todo file");

        let initial_config = r#"
[[panes]]
label = "Initial"
filter = "project:one"
sort = "file_order"
group = false
"#;
        fs::write(&config_path, initial_config).expect("should write initial config");

        let mut app = App::new(
            TaskList::load(&todo_path).expect("Failed to load TaskList"),
            todo_path,
            TuiConfig::load(&config_path).expect("should load config"),
            Some(config_path.clone()),
            todotxt_tui::theme::Theme::Default,
            false,
        );

        app.panes[0].filter_query = "project:changed".to_string();
        let state_path = todotxt_tui::config::state_file_path(&config_path);
        // state file must not exist before quit save
        assert!(
            !state_path.exists(),
            "state file must not be written until quit"
        );

        app.save_view_state()
            .expect("quit persistence should succeed");
        let after_quit_write =
            fs::read_to_string(&state_path).expect("should read state file after quit save");
        assert!(after_quit_write.contains("project:changed"));
        // dir drops here, cleaning up all temp files automatically
    }

    // PRSV-02 startup override: tui-state.toml panes fully replace config.toml panes
    #[test]
    fn test_startup_state_file_overrides_config_panes() {
        let dir = tempfile::tempdir().expect("temp dir must be creatable");
        let todo_path = dir.path().join("todo.txt");
        let config_path = dir.path().join("config.toml");
        let state_path = todotxt_tui::config::state_file_path(&config_path);

        File::create(&todo_path).unwrap().write_all(b"").unwrap();

        // config.toml defines "OldWork" pane
        fs::write(
            &config_path,
            "[[panes]]\nlabel = \"OldWork\"\nfilter = \"+old\"\n",
        )
        .expect("should write config");

        // tui-state.toml defines "StateWork" pane — should win at startup
        let state = todotxt_tui::config::TuiStateFile {
            panes: vec![PaneConfig {
                label: "StateWork".to_string(),
                filter: "+state".to_string(),
                sort: PaneSort::FileOrder,
                group: false,
                group_by: None,
            }],
        };
        state.save(&state_path).expect("should write state file");

        // Replicate main.rs startup logic: load config, then load state, override panes
        let mut config = TuiConfig::load(&config_path).expect("should load config");
        if let Some(loaded_state) = todotxt_tui::config::TuiStateFile::load(&state_path) {
            if !loaded_state.panes.is_empty() {
                config.panes = loaded_state.panes;
            }
        }

        let app = App::new(
            TaskList::load(&todo_path).expect("Failed to load TaskList"),
            todo_path,
            config,
            Some(config_path),
            todotxt_tui::theme::Theme::Default,
            false,
        );

        // Panes come from state file, not config.toml
        assert_eq!(app.panes.len(), 1, "panes should come from state file");
        assert_eq!(
            app.panes[0].label, "StateWork",
            "label must match state file, not config.toml"
        );
        assert_eq!(app.panes[0].filter_query, "+state");
    }

    // PRSV-02 startup fallback: absent state file → config.toml panes used, no error
    #[test]
    fn test_startup_absent_state_file_uses_config_panes() {
        let dir = tempfile::tempdir().expect("temp dir must be creatable");
        let todo_path = dir.path().join("todo.txt");
        let config_path = dir.path().join("config.toml");
        let state_path = todotxt_tui::config::state_file_path(&config_path);

        File::create(&todo_path).unwrap().write_all(b"").unwrap();

        // config.toml defines "ConfigPane" — no state file present
        fs::write(
            &config_path,
            "[[panes]]\nlabel = \"ConfigPane\"\nfilter = \"+config\"\n",
        )
        .expect("should write config");

        // No state file created — must not exist
        assert!(
            !state_path.exists(),
            "state file must not exist for this test"
        );

        // Replicate main.rs startup logic: load state returns None → config unchanged
        let mut config = TuiConfig::load(&config_path).expect("should load config");
        if let Some(loaded_state) = todotxt_tui::config::TuiStateFile::load(&state_path) {
            if !loaded_state.panes.is_empty() {
                config.panes = loaded_state.panes;
            }
        }

        let app = App::new(
            TaskList::load(&todo_path).expect("Failed to load TaskList"),
            todo_path,
            config,
            Some(config_path),
            todotxt_tui::theme::Theme::Default,
            false,
        );

        // Panes come from config.toml
        assert_eq!(app.panes.len(), 1, "panes should come from config.toml");
        assert_eq!(
            app.panes[0].label, "ConfigPane",
            "label must match config.toml"
        );
        assert_eq!(app.panes[0].filter_query, "+config");
    }

    // PRSV-03 / D-06 skip-write: unchanged pane state → save_view_state does NOT write state file
    #[test]
    fn test_save_view_state_no_write_when_unchanged() {
        let dir = tempfile::tempdir().expect("temp dir must be creatable");
        let todo_path = dir.path().join("todo.txt");
        let config_path = dir.path().join("config.toml");
        let state_path = todotxt_tui::config::state_file_path(&config_path);

        File::create(&todo_path).unwrap().write_all(b"").unwrap();

        // Config with one pane — app starts with this, nothing changes
        let config_toml = "[[panes]]\nlabel = \"Work\"\nfilter = \"+work\"\ngroup = false\n";
        fs::write(&config_path, config_toml).expect("should write config");

        let app = App::new(
            TaskList::load(&todo_path).expect("Failed to load TaskList"),
            todo_path,
            TuiConfig::load(&config_path).expect("should load config"),
            Some(config_path.clone()),
            todotxt_tui::theme::Theme::Default,
            false,
        );

        // No pane state mutation — startup_pane_snapshot matches current state
        app.save_view_state()
            .expect("save_view_state must not fail");

        // State file must NOT be written (compare-on-quit optimization)
        assert!(
            !state_path.exists(),
            "state file must NOT be written when pane state is unchanged from startup"
        );
    }
}

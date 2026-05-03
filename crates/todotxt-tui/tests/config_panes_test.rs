use std::fs;
use std::path::PathBuf;

use todotxt_tui::config::{PaneSort, TuiConfig};

fn unique_temp_file(prefix: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!("{}_{}_{}.toml", prefix, std::process::id(), nanos))
}

#[test]
fn config_panes_test_valid_entries_deserialize_and_map_fields() {
    let config_toml = r#"
[[panes]]
label = "Work"
filter = "project:work"
sort = "priority"
group = true

[[panes]]
label = "Today"
filter = "due:today"
sort = "due_date"
group = false
"#;

    let config: TuiConfig = toml::from_str(config_toml).expect("expected config to deserialize");

    assert_eq!(config.panes.len(), 2);
    assert_eq!(config.panes[0].label, "Work");
    assert_eq!(config.panes[0].filter, "project:work");
    assert_eq!(config.panes[0].sort, PaneSort::Priority);
    assert!(config.panes[0].group);

    assert_eq!(config.panes[1].label, "Today");
    assert_eq!(config.panes[1].filter, "due:today");
    assert_eq!(config.panes[1].sort, PaneSort::DueDate);
    assert!(!config.panes[1].group);
}

#[test]
fn config_panes_test_invalid_sort_is_skipped_while_other_entries_survive() {
    let path = unique_temp_file("config_panes_invalid_sort");
    let config_toml = r#"
[[panes]]
label = "Keep"
sort = "priority"

[[panes]]
label = "Drop"
sort = "not_a_real_sort"

[[panes]]
label = "Also Keep"
sort = "alphabetical"
"#;

    fs::write(&path, config_toml).expect("should write test config");

    let config = TuiConfig::load(&path).expect("invalid pane entry should be skipped during load");

    assert_eq!(config.panes.len(), 2);
    assert_eq!(config.panes[0].label, "Keep");
    assert_eq!(config.panes[0].sort, PaneSort::Priority);
    assert_eq!(config.panes[1].label, "Also Keep");
    assert_eq!(config.panes[1].sort, PaneSort::Alphabetical);

    let _ = fs::remove_file(path);
}

#[test]
fn config_panes_test_missing_and_empty_sections_deserialize_safely() {
    let missing_panes_toml = r#"
todo_file = "tasks.txt"
"#;
    let missing: TuiConfig = toml::from_str(missing_panes_toml)
        .expect("missing [[panes]] should deserialize with defaults");
    assert!(missing.panes.is_empty());

    let path = unique_temp_file("config_panes_empty_array");
    fs::write(&path, "panes = []\n").expect("should write empty panes config");

    let empty = TuiConfig::load(&path).expect("empty panes array should load");
    assert!(empty.panes.is_empty());

    let _ = fs::remove_file(path);
}

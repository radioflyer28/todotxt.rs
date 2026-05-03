use std::fs;
use tempfile::tempdir;
use todotxt_core::{resolve_config_path, TaskList};

// === Line ending round-trip tests ===

#[test]
fn test_crlf_file_round_trips_without_corruption() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("todo.txt");

    // Write a file with explicit CRLF line endings
    let crlf_content = b"(A) Buy groceries +shopping\r\n(B) Call dentist @phone\r\ntask three\r\n";
    fs::write(&path, crlf_content).expect("write CRLF file");

    // Load via TaskList
    let list = TaskList::load(&path).expect("load CRLF file");
    assert_eq!(list.tasks().len(), 3, "should have 3 tasks");

    // Save back to the same file
    list.save().expect("save CRLF file");

    // Read raw bytes and verify CRLF preserved
    let saved = fs::read(&path).expect("read saved file");
    let has_crlf = saved.windows(2).any(|w| w == b"\r\n");
    assert!(has_crlf, "CRLF line endings must be preserved after round-trip");

    // Verify no double-CR was introduced
    let has_double_cr = saved.windows(3).any(|w| w == b"\r\r\n");
    assert!(!has_double_cr, "double CR must not be introduced");

    // Task count must be preserved
    let reloaded = TaskList::load(&path).expect("reload after save");
    assert_eq!(reloaded.tasks().len(), 3, "task count must survive round-trip");
}

#[test]
fn test_lf_file_round_trips_without_corruption() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("todo.txt");

    // Write a file with LF only
    let lf_content = b"task alpha\ntask beta\ntask gamma\n";
    fs::write(&path, lf_content).expect("write LF file");

    let list = TaskList::load(&path).expect("load LF file");
    assert_eq!(list.tasks().len(), 3);

    list.save().expect("save LF file");

    let saved = fs::read(&path).expect("read saved file");

    // No CRLF should be introduced into an LF file
    let has_crlf = saved.windows(2).any(|w| w == b"\r\n");
    assert!(!has_crlf, "CRLF must not be introduced into an LF file");

    let reloaded = TaskList::load(&path).expect("reload after save");
    assert_eq!(reloaded.tasks().len(), 3, "task count must survive round-trip");
}

#[test]
fn test_task_count_consistent_across_line_endings() {
    let dir = tempdir().expect("temp dir");

    let crlf_path = dir.path().join("crlf.txt");
    let lf_path = dir.path().join("lf.txt");

    // Same 3 tasks, different line endings
    fs::write(&crlf_path, b"task one\r\ntask two\r\ntask three\r\n").expect("write CRLF");
    fs::write(&lf_path, b"task one\ntask two\ntask three\n").expect("write LF");

    let crlf_list = TaskList::load(&crlf_path).expect("load CRLF");
    let lf_list = TaskList::load(&lf_path).expect("load LF");

    assert_eq!(crlf_list.tasks().len(), 3, "CRLF file should have 3 tasks");
    assert_eq!(lf_list.tasks().len(), 3, "LF file should have 3 tasks");
    assert_eq!(
        crlf_list.tasks().len(),
        lf_list.tasks().len(),
        "task count must be identical regardless of line endings"
    );
}

// === Portable mode tests ===

#[test]
fn test_portable_mode_prefers_binary_adjacent_config() {
    let binary_dir = tempdir().expect("binary temp dir");
    let platform_dir = tempdir().expect("platform temp dir");

    // Write config.toml beside the "binary"
    let config_path = binary_dir.path().join("config.toml");
    fs::write(&config_path, "[paths]\n").expect("write portable config");

    let result = resolve_config_path(binary_dir.path(), platform_dir.path());

    assert_eq!(
        result,
        binary_dir.path(),
        "portable mode: binary-adjacent config.toml must take precedence"
    );
}

#[test]
fn test_portable_mode_falls_back_to_platform_path() {
    let binary_dir = tempdir().expect("binary temp dir without config");
    let platform_dir = tempdir().expect("platform temp dir");

    // No config.toml in binary_dir — should fall back
    let result = resolve_config_path(binary_dir.path(), platform_dir.path());

    assert_eq!(
        result,
        platform_dir.path(),
        "without sidecar config.toml, must fall back to platform directory"
    );
}

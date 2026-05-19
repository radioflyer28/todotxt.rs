use std::fs;
use tempfile::TempDir;
use todotxt_core::{LineEnding, Task, TaskList, TodoError};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Write bytes to a file in a TempDir and return the file path.
fn write_file(dir: &TempDir, name: &str, content: &[u8]) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).unwrap();
    path
}

/// Write a UTF-8 string to a temp file and return the path.
fn write_str(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    write_file(dir, name, content.as_bytes())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn load_and_parse() {
    let dir = TempDir::new().unwrap();
    let path = write_str(
        &dir,
        "todo.txt",
        "(A) 2024-01-15 Call dentist +Health @phone\n\
         x 2024-01-10 2024-01-05 Pay bills\n\
         Buy milk\n",
    );

    let list = TaskList::load(&path).unwrap();

    assert_eq!(list.len(), 3);
    assert_eq!(list.tasks()[0].priority, Some('A'));
    assert!(list.tasks()[1].completed);
    assert_eq!(list.tasks()[2].body, "Buy milk");
}

#[test]
fn load_bom_stripping() {
    let dir = TempDir::new().unwrap();

    // Write file with UTF-8 BOM prefix.
    let mut content: Vec<u8> = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
    content.extend_from_slice(b"(A) First task with BOM\n(B) Second task\n");
    let path = write_file(&dir, "bom.txt", &content);

    let list = TaskList::load(&path).unwrap();

    // BOM must be stripped — priority A must parse correctly.
    assert_eq!(list.len(), 2);
    assert_eq!(
        list.tasks()[0].priority,
        Some('A'),
        "BOM was not stripped: priority did not parse"
    );
    assert_eq!(list.tasks()[0].body, "First task with BOM");

    // Save and verify no BOM in output.
    let out_path = dir.path().join("out.txt");
    fs::copy(&path, &out_path).unwrap();
    let list2 = TaskList::load(&out_path).unwrap();
    list2.save().unwrap();
    let raw = fs::read(&out_path).unwrap();
    assert!(
        !raw.starts_with(&[0xEF, 0xBB, 0xBF]),
        "BOM must not be written back on save"
    );
}

#[test]
fn crlf_round_trip() {
    let dir = TempDir::new().unwrap();
    let path = write_str(
        &dir,
        "crlf.txt",
        "(A) First task\r\n(B) Second task\r\nBuy milk\r\n",
    );

    let list = TaskList::load(&path).unwrap();

    assert_eq!(list.line_ending(), LineEnding::CrLf);
    assert_eq!(list.len(), 3);

    // Save to a new path and verify raw bytes contain CRLF.
    let out_path = dir.path().join("out_crlf.txt");
    fs::copy(&path, &out_path).unwrap();
    let list2 = TaskList::load(&out_path).unwrap();
    list2.save().unwrap();

    let raw = fs::read(&out_path).unwrap();
    let content = String::from_utf8(raw).unwrap();
    assert!(content.contains("\r\n"), "CRLF was not preserved on save");
    // Should NOT contain bare \n that isn't preceded by \r.
    let bare_lf = content
        .as_bytes()
        .windows(2)
        .any(|w| w[0] != b'\r' && w[1] == b'\n');
    assert!(!bare_lf, "Bare LF found in CRLF file after save");
}

#[test]
fn lf_round_trip() {
    let dir = TempDir::new().unwrap();
    let path = write_str(
        &dir,
        "lf.txt",
        "(A) First task\n(B) Second task\nBuy milk\n",
    );

    let list = TaskList::load(&path).unwrap();
    assert_eq!(list.line_ending(), LineEnding::Lf);

    list.save().unwrap();

    let raw = fs::read(&path).unwrap();
    let content = String::from_utf8(raw).unwrap();
    assert!(
        !content.contains("\r\n"),
        "CRLF should not appear in LF file after save"
    );
}

#[test]
fn add_task() {
    let dir = TempDir::new().unwrap();
    let path = write_str(&dir, "todo.txt", "(A) First\n(B) Second\n(C) Third\n");

    let mut list = TaskList::load(&path).unwrap();
    let new_task = Task::parse("(D) Fourth");
    list.add(new_task).unwrap();

    assert_eq!(list.len(), 4);
    assert_eq!(list.tasks()[3].body, "Fourth");

    // Verify persisted to disk.
    let reloaded = TaskList::load(&path).unwrap();
    assert_eq!(reloaded.len(), 4);
    assert_eq!(reloaded.tasks()[3].body, "Fourth");
}

#[test]
fn delete_by_index() {
    let dir = TempDir::new().unwrap();
    let path = write_str(&dir, "todo.txt", "Alpha\nBeta\nGamma\n");

    let mut list = TaskList::load(&path).unwrap();
    list.delete(1).unwrap(); // Remove "Beta"

    assert_eq!(list.len(), 2);
    assert_eq!(list.tasks()[0].body, "Alpha");
    assert_eq!(list.tasks()[1].body, "Gamma");

    let reloaded = TaskList::load(&path).unwrap();
    assert_eq!(reloaded.len(), 2);
}

#[test]
fn update_by_index() {
    let dir = TempDir::new().unwrap();
    let path = write_str(&dir, "todo.txt", "Alpha\nBeta\nGamma\n");

    let mut list = TaskList::load(&path).unwrap();
    let replacement = Task::parse("Delta");
    list.update(1, replacement).unwrap();

    assert_eq!(list.tasks()[0].body, "Alpha");
    assert_eq!(list.tasks()[1].body, "Delta");
    assert_eq!(list.tasks()[2].body, "Gamma");

    let reloaded = TaskList::load(&path).unwrap();
    assert_eq!(reloaded.tasks()[1].body, "Delta");
}

/// C-1 fix: delete must use index-based identity, not raw-string comparison.
/// Two identical tasks exist; delete(0) must remove only the first.
#[test]
fn duplicate_task_deletion() {
    let dir = TempDir::new().unwrap();
    let path = write_str(&dir, "todo.txt", "Buy milk\nBuy milk\n");

    let mut list = TaskList::load(&path).unwrap();
    assert_eq!(list.len(), 2);

    list.delete(0).unwrap();

    assert_eq!(list.len(), 1, "should have exactly 1 task after delete(0)");
    assert_eq!(
        list.tasks()[0].to_string(),
        "Buy milk",
        "remaining task should be the second 'Buy milk'"
    );

    let reloaded = TaskList::load(&path).unwrap();
    assert_eq!(reloaded.len(), 1);
}

#[test]
fn delete_out_of_bounds() {
    let dir = TempDir::new().unwrap();
    let path = write_str(&dir, "todo.txt", "Alpha\nBeta\nGamma\n");

    let mut list = TaskList::load(&path).unwrap();
    let result = list.delete(5);

    match result {
        Err(TodoError::IndexOutOfBounds { index, count }) => {
            assert_eq!(index, 5);
            assert_eq!(count, 3);
        }
        other => panic!("expected IndexOutOfBounds, got {:?}", other),
    }
}

#[test]
fn update_out_of_bounds() {
    let dir = TempDir::new().unwrap();
    let path = write_str(&dir, "todo.txt", "Alpha\nBeta\n");

    let mut list = TaskList::load(&path).unwrap();
    let result = list.update(10, Task::parse("Replacement"));

    match result {
        Err(TodoError::IndexOutOfBounds { index, count }) => {
            assert_eq!(index, 10);
            assert_eq!(count, 2);
        }
        other => panic!("expected IndexOutOfBounds, got {:?}", other),
    }
}

#[test]
fn atomic_write_creates_file() {
    let dir = TempDir::new().unwrap();
    let path = write_str(&dir, "todo.txt", "Alpha\nBeta\n");

    let list = TaskList::load(&path).unwrap();
    list.save().unwrap();

    // File must exist and contain the tasks.
    assert!(path.exists(), "file must exist after save");
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("Alpha"));
    assert!(content.contains("Beta"));
}

#[test]
fn preserve_whitespace() {
    let dir = TempDir::new().unwrap();
    let content = "Alpha\n\nBeta\n\nGamma\n";
    let path = write_str(&dir, "todo.txt", content);

    // preserve_whitespace: false (default) drops blank lines.
    let list_no_ws = TaskList::load(&path).unwrap();
    assert_eq!(
        list_no_ws.len(),
        3,
        "blank lines should be dropped by default"
    );

    // preserve_whitespace: true keeps blank lines.
    let list_with_ws = TaskList::load_with_options(&path, true).unwrap();
    assert_eq!(
        list_with_ws.len(),
        5,
        "blank lines should be preserved when preserve_whitespace=true"
    );
    assert_eq!(list_with_ws.tasks()[1].to_string(), "");
}

// ── Mixed line-ending regression tests (UAT gap 03-04) ──────────────────────

/// Loading a file with mixed LF/CRLF line endings must produce tasks whose
/// to_raw() values contain no trailing carriage-return characters.
#[test]
fn load_mixed_line_endings_no_cr_in_raw() {
    let dir = TempDir::new().unwrap();
    // Two LF tasks and one CRLF task (third line).
    let content = b"Buy milk\nx 2024-01-10 Completed task\r\n(A) Important task\n";
    let path = write_file(&dir, "mixed.txt", content);

    let list = TaskList::load(&path).unwrap();

    assert_eq!(list.len(), 3);
    for task in list.tasks() {
        assert!(
            !task.to_raw().contains('\r'),
            "to_raw() must not contain carriage return; got: {:?}",
            task.to_raw()
        );
    }
    // The CRLF row must still parse correctly.
    assert!(list.tasks()[1].completed);
    assert_eq!(list.tasks()[1].body, "Completed task");
}

/// When the first line is LF but later lines are CRLF, line_ending is Lf
/// and save must produce an LF-only file.
#[test]
fn mixed_file_detected_as_lf_saves_as_lf() {
    let dir = TempDir::new().unwrap();
    // First newline is bare LF → detected as Lf.
    let content = b"Buy milk\nx 2024-01-10 Completed task\r\n";
    let path = write_file(&dir, "mixed.txt", content);

    let list = TaskList::load(&path).unwrap();
    assert_eq!(
        list.line_ending(),
        LineEnding::Lf,
        "first LF line should set Lf ending"
    );

    list.save().unwrap();
    let saved = fs::read(&path).unwrap();
    assert!(
        !saved.windows(2).any(|w| w == b"\r\n"),
        "saved file must use LF line endings because first line was LF"
    );
}

#[test]
fn reload_picks_up_external_changes() {
    let dir = TempDir::new().unwrap();
    let path = write_str(&dir, "todo.txt", "Alpha\nBeta\n");

    let mut list = TaskList::load(&path).unwrap();
    assert_eq!(list.len(), 2);

    // Externally overwrite the file with different content.
    fs::write(&path, "Alpha\nBeta\nGamma\nDelta\n").unwrap();

    list.reload().unwrap();
    assert_eq!(list.len(), 4);
    assert_eq!(list.tasks()[2].body, "Gamma");
    assert_eq!(list.tasks()[3].body, "Delta");
}

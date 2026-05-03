use todotxt_core::{Task, TaskList, TodoError};
use std::fs;
use tempfile::TempDir;

fn task_list_from(lines: &[&str]) -> (TaskList, TempDir) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("todo.txt");
    let content = lines.join("\n") + "\n";
    fs::write(&path, content.as_bytes()).unwrap();
    let tl = TaskList::load(&path).unwrap();
    (tl, dir)
}

// ── Valid batch ───────────────────────────────────────────────────────────────

#[test]
fn batch_update_applies_all_replacements() {
    let (mut tl, _tmp) = task_list_from(&["Task one", "Task two", "Task three"]);
    let new_one = Task::parse("Updated one");
    let new_three = Task::parse("Updated three");
    tl.batch_update(vec![(0, new_one), (2, new_three)]).unwrap();
    assert_eq!(tl.tasks()[0].to_raw(), "Updated one");
    assert_eq!(tl.tasks()[1].to_raw(), "Task two"); // unchanged
    assert_eq!(tl.tasks()[2].to_raw(), "Updated three");
    // File must reflect both changes
    let on_disk = std::fs::read_to_string(tl.path()).unwrap();
    assert!(on_disk.contains("Updated one"));
    assert!(on_disk.contains("Updated three"));
}

#[test]
fn batch_update_single_replacement_works() {
    let (mut tl, _tmp) = task_list_from(&["Original task"]);
    let updated = Task::parse("(A) Updated task");
    tl.batch_update(vec![(0, updated)]).unwrap();
    assert_eq!(tl.tasks()[0].priority, Some('A'));
}

// ── Fail-fast: out-of-bounds ──────────────────────────────────────────────────

#[test]
fn batch_update_out_of_bounds_returns_error_no_mutation() {
    let (mut tl, _tmp) = task_list_from(&["Task zero", "Task one"]);
    let replacement = Task::parse("Replacement");
    // Index 5 is out of bounds (count is 2)
    let result = tl.batch_update(vec![(0, replacement.clone()), (5, replacement)]);
    assert!(
        matches!(result, Err(TodoError::IndexOutOfBounds { index: 5, count: 2 })),
        "expected IndexOutOfBounds(5, 2), got {:?}", result
    );
    // Task at index 0 must NOT have been mutated (fail-fast — validate before apply)
    assert_eq!(tl.tasks()[0].to_raw(), "Task zero");
    assert_eq!(tl.tasks()[1].to_raw(), "Task one");
}

// ── Empty batch ───────────────────────────────────────────────────────────────

#[test]
fn batch_update_empty_replacements_saves_unchanged() {
    let (mut tl, _tmp) = task_list_from(&["Task zero"]);
    let original_content = std::fs::read_to_string(tl.path()).unwrap();
    tl.batch_update(vec![]).unwrap();
    // Tasks unchanged in memory
    assert_eq!(tl.tasks()[0].to_raw(), "Task zero");
    // File content round-trips correctly (empty batch still calls save)
    let after_content = std::fs::read_to_string(tl.path()).unwrap();
    assert_eq!(original_content, after_content);
}

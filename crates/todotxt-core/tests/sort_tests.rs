use std::fs;
use tempfile::TempDir;
use todotxt_core::{SortOrder, TaskList};

fn task_list_from(lines: &[&str]) -> (TaskList, TempDir) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("todo.txt");
    let content = lines.join("\n") + "\n";
    fs::write(&path, content.as_bytes()).unwrap();
    let tl = TaskList::load(&path).unwrap();
    (tl, dir)
}

// ── Priority sort ─────────────────────────────────────────────────────────────

#[test]
fn sort_priority_a_before_b_before_none() {
    let (mut tl, _tmp) = task_list_from(&["No priority", "(B) Second", "(A) First"]);
    tl.sort(SortOrder::Priority);
    let tasks = tl.tasks();
    assert_eq!(tasks[0].priority, Some('A'));
    assert_eq!(tasks[1].priority, Some('B'));
    assert_eq!(tasks[2].priority, None);
}

#[test]
fn sort_priority_stable_for_equal_priorities() {
    let (mut tl, _tmp) =
        task_list_from(&["(A) Alpha first", "(A) Alpha second", "(A) Alpha third"]);
    tl.sort(SortOrder::Priority);
    let tasks = tl.tasks();
    // Original order preserved (stable sort)
    assert!(tasks[0].to_raw().contains("Alpha first"));
    assert!(tasks[1].to_raw().contains("Alpha second"));
    assert!(tasks[2].to_raw().contains("Alpha third"));
}

// ── DueDate sort ──────────────────────────────────────────────────────────────

#[test]
fn sort_due_date_earliest_first_none_last() {
    let (mut tl, _tmp) = task_list_from(&[
        "No due date",
        "Late task due:2099-12-31",
        "Early task due:2000-01-01",
    ]);
    tl.sort(SortOrder::DueDate);
    let tasks = tl.tasks();
    assert!(tasks[0].to_raw().contains("2000-01-01"));
    assert!(tasks[1].to_raw().contains("2099-12-31"));
    assert_eq!(tasks[2].due_date, None);
}

// ── Alphabetical sort ─────────────────────────────────────────────────────────

#[test]
fn sort_alphabetical_case_insensitive() {
    let (mut tl, _tmp) = task_list_from(&["Zebra task", "apple task", "Mango task"]);
    tl.sort(SortOrder::Alphabetical);
    let tasks = tl.tasks();
    assert!(tasks[0].to_raw().to_ascii_lowercase().starts_with("apple"));
    assert!(tasks[1].to_raw().to_ascii_lowercase().starts_with("mango"));
    assert!(tasks[2].to_raw().to_ascii_lowercase().starts_with("zebra"));
}

// ── Project sort ──────────────────────────────────────────────────────────────

#[test]
fn sort_project_first_tag_alpha_none_last() {
    let (mut tl, _tmp) = task_list_from(&["No project", "Task +zebra", "Task +alpha"]);
    tl.sort(SortOrder::Project);
    let tasks = tl.tasks();
    assert_eq!(tasks[0].projects.first().map(|s| s.as_str()), Some("alpha"));
    assert_eq!(tasks[1].projects.first().map(|s| s.as_str()), Some("zebra"));
    assert!(tasks[2].projects.is_empty());
}

// ── Context sort ──────────────────────────────────────────────────────────────

#[test]
fn sort_context_first_tag_alpha_none_last() {
    let (mut tl, _tmp) = task_list_from(&["No context", "Task @work", "Task @home"]);
    tl.sort(SortOrder::Context);
    let tasks = tl.tasks();
    assert_eq!(tasks[0].contexts.first().map(|s| s.as_str()), Some("home"));
    assert_eq!(tasks[1].contexts.first().map(|s| s.as_str()), Some("work"));
    assert!(tasks[2].contexts.is_empty());
}

// ── sort() must NOT save to disk ──────────────────────────────────────────────

#[test]
fn sort_does_not_save_to_disk() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("todo.txt");
    let original = "(B) Second\n(A) First\n";
    fs::write(&path, original).unwrap();
    let mut tl = TaskList::load(&path).unwrap();
    tl.sort(SortOrder::Priority);
    // Re-read file — should be in original order
    let on_disk = std::fs::read_to_string(&path).unwrap();
    assert_eq!(on_disk, original, "sort() must not write to disk");
}

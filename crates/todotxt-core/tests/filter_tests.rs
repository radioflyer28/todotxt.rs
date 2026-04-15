use todotxt_core::{Filter, TaskList};
use chrono::Local;
use std::fs;
use tempfile::TempDir;

fn today_str() -> String { Local::now().date_naive().format("%Y-%m-%d").to_string() }

fn task_list_from(lines: &[&str]) -> (TaskList, TempDir) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("todo.txt");
    let content = lines.join("\n") + "\n";
    fs::write(&path, content.as_bytes()).unwrap();
    let tl = TaskList::load(&path).unwrap();
    (tl, dir)
}

// ── DONE / -DONE ─────────────────────────────────────────────────────────────

#[test]
fn filter_done_returns_only_completed() {
    let (tl, _tmp) = task_list_from(&[
        "x 2024-01-01 Completed task",
        "Incomplete task",
    ]);
    let f = Filter::from_query("DONE");
    let results = tl.filter(&f);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 0);
    assert!(results[0].1.completed);
}

#[test]
fn filter_not_done_returns_only_incomplete() {
    let (tl, _tmp) = task_list_from(&[
        "x 2024-01-01 Completed task",
        "Incomplete task",
    ]);
    let f = Filter::from_query("-DONE");
    let results = tl.filter(&f);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 1);
    assert!(!results[0].1.completed);
}

// ── due: tokens ───────────────────────────────────────────────────────────────

#[test]
fn filter_due_today_matches_today_only() {
    let today = today_str();
    let (tl, _tmp) = task_list_from(&[
        &format!("Task due today due:{}", today),
        "Overdue task due:2000-01-01",
        "No due date",
    ]);
    let f = Filter::from_query("due:today");
    let results = tl.filter(&f);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 0);
}

#[test]
fn filter_due_past_matches_overdue() {
    let (tl, _tmp) = task_list_from(&[
        "Overdue due:2000-01-01",
        "Future due:2099-12-31",
    ]);
    let f = Filter::from_query("due:past");
    let results = tl.filter(&f);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 0);
}

#[test]
fn filter_due_future_matches_future_only() {
    let today = today_str();
    let (tl, _tmp) = task_list_from(&[
        "Past due:2000-01-01",
        &format!("Today due:{}", today),
        "Future due:2099-12-31",
    ]);
    let f = Filter::from_query("due:future");
    let results = tl.filter(&f);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 2);
}

#[test]
fn filter_due_active_matches_today_and_past() {
    let today = today_str();
    let (tl, _tmp) = task_list_from(&[
        "Past due:2000-01-01",
        &format!("Today due:{}", today),
        "Future due:2099-12-31",
        "No due",
    ]);
    let f = Filter::from_query("due:active");
    let results = tl.filter(&f);
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|(i, _)| *i == 0));
    assert!(results.iter().any(|(i, _)| *i == 1));
}

// ── substring include / exclude ───────────────────────────────────────────────

#[test]
fn filter_substring_include_case_insensitive() {
    let (tl, _tmp) = task_list_from(&["Buy milk @home", "Send email @work"]);
    let f = Filter::from_query("@Home");
    let results = tl.filter(&f);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 0);
}

#[test]
fn filter_substring_exclude_removes_matches() {
    let (tl, _tmp) = task_list_from(&["Buy milk @home", "Send email @work"]);
    let f = Filter::from_query("-@work");
    let results = tl.filter(&f);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 0);
}

// ── AND logic ────────────────────────────────────────────────────────────────

#[test]
fn filter_multi_token_and_logic() {
    let (tl, _tmp) = task_list_from(&[
        "Buy milk @home",
        "x 2024-01-01 Done task @home",
        "Send email @work",
    ]);
    let f = Filter::from_query("@home -DONE");
    let results = tl.filter(&f);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 0);
}

// ── suppression pre-filters ───────────────────────────────────────────────────

#[test]
fn filter_suppresses_hidden_h1_by_default() {
    let (tl, _tmp) = task_list_from(&["Normal task", "Secret task h:1"]);
    let f = Filter::new();
    let results = tl.filter(&f);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 0);
}

#[test]
fn filter_shows_hidden_when_suppression_off() {
    let (tl, _tmp) = task_list_from(&["Normal task", "Secret task h:1"]);
    let f = Filter { suppress_hidden: false, ..Filter::new() };
    let results = tl.filter(&f);
    assert_eq!(results.len(), 2);
}

#[test]
fn filter_suppresses_future_threshold_by_default() {
    let (tl, _tmp) = task_list_from(&[
        "Future task t:2099-12-31",
        "Past threshold t:2000-01-01",
        "No threshold",
    ]);
    let f = Filter::new();
    let results = tl.filter(&f);
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|(i, _)| *i != 0));
}

// ── index accuracy ────────────────────────────────────────────────────────────

#[test]
fn filter_returns_correct_indices() {
    let (tl, _tmp) = task_list_from(&[
        "Task zero",
        "Task one @keep",
        "Task two",
        "Task three @keep",
    ]);
    let f = Filter::from_query("@keep");
    let results = tl.filter(&f);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].0, 1);
    assert_eq!(results[1].0, 3);
}

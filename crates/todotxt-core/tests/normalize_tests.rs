//! Integration tests for normalize_append() and normalize_line() — Phase 21
//!
//! Run: cargo test -p todotxt-core normalize

use chrono::NaiveDate;
use todotxt_core::{normalize_append, normalize_line, Task};

// ── normalize_append: priority (NORM-01) ─────────────────────────────────────

#[test]
fn priority_appended_wins_over_original() {
    // D-03: appended priority replaces original
    let task = Task::parse("(A) fix bug");
    let result = normalize_append(&task, "(B)");
    assert_eq!(result.priority, Some('B'), "appended (B) should replace original (A)");
    assert_eq!(result.body, "fix bug");
    assert!(result.to_raw().starts_with("(B) "), "raw must start with (B)");
}

#[test]
fn priority_appended_to_task_without_priority() {
    // D-04: appended priority becomes task's priority when original has none
    let task = Task::parse("fix bug");
    let result = normalize_append(&task, "(A)");
    assert_eq!(result.priority, Some('A'));
    assert_eq!(result.body, "fix bug");
}

#[test]
fn priority_unchanged_when_not_in_append_text() {
    let task = Task::parse("(A) fix bug");
    let result = normalize_append(&task, "more text");
    assert_eq!(result.priority, Some('A'), "original priority preserved");
    assert!(result.body.contains("more text"));
}

// ── normalize_append: projects (NORM-02) ─────────────────────────────────────

#[test]
fn projects_from_append_added_to_existing() {
    let task = Task::parse("fix bug +work");
    let result = normalize_append(&task, "+home");
    assert!(result.projects.contains(&"work".to_string()));
    assert!(result.projects.contains(&"home".to_string()));
    // rebuild_raw places projects after body: check raw form
    let raw = result.to_raw();
    assert!(raw.contains("+work"), "raw must contain +work");
    assert!(raw.contains("+home"), "raw must contain +home");
}

#[test]
fn projects_deduplicated_when_appended_already_present() {
    let task = Task::parse("fix bug +work");
    let result = normalize_append(&task, "+work");
    let work_count = result.projects.iter().filter(|p| p.as_str() == "work").count();
    assert_eq!(work_count, 1, "+work must appear exactly once");
}

// ── normalize_append: contexts (NORM-03) ─────────────────────────────────────

#[test]
fn contexts_from_append_added_to_existing() {
    let task = Task::parse("fix bug @office");
    let result = normalize_append(&task, "@home");
    assert!(result.contexts.contains(&"office".to_string()));
    assert!(result.contexts.contains(&"home".to_string()));
}

// ── normalize_append: dates (NORM-04) ────────────────────────────────────────

#[test]
fn due_date_from_append_wins_over_original() {
    let task = Task::parse("fix bug due:2026-01-01");
    let result = normalize_append(&task, "due:2026-06-15");
    assert_eq!(
        result.due_date,
        Some(NaiveDate::from_ymd_opt(2026, 6, 15).unwrap())
    );
}

#[test]
fn due_date_preserved_when_not_in_append_text() {
    let task = Task::parse("fix bug due:2026-01-01");
    let result = normalize_append(&task, "plain text");
    assert_eq!(
        result.due_date,
        Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    );
}

#[test]
fn threshold_date_from_append_wins_over_original() {
    let task = Task::parse("fix bug t:2026-01-01");
    let result = normalize_append(&task, "t:2026-06-15");
    assert_eq!(
        result.threshold_date,
        Some(NaiveDate::from_ymd_opt(2026, 6, 15).unwrap())
    );
}

// ── normalize_append: unknown tokens / plain text (NORM-05) ──────────────────

#[test]
fn unknown_tokens_preserved_in_body() {
    // D-02: rec:+1w is unrecognized — must land in body, not be discarded
    let task = Task::parse("fix bug");
    let result = normalize_append(&task, "rec:+1w");
    assert!(
        result.body.contains("rec:+1w"),
        "unrecognized token must be preserved in body; got body: {:?}",
        result.body
    );
}

#[test]
fn plain_text_preserved_in_body() {
    let task = Task::parse("fix bug");
    let result = normalize_append(&task, "extra detail");
    assert!(result.body.contains("extra detail"));
    assert!(result.body.contains("fix bug"));
}

#[test]
fn empty_append_text_returns_original_unchanged() {
    let task = Task::parse("(A) fix bug +work due:2026-01-01");
    let result = normalize_append(&task, "");
    assert_eq!(result.to_raw(), task.to_raw());
}

// ── normalize_append: completed task fields preserved ────────────────────────

#[test]
fn completed_flag_preserved_after_normalize_append() {
    let task = Task::parse("x 2026-01-01 2025-12-01 fix bug");
    let result = normalize_append(&task, "+work");
    assert!(result.completed, "completed flag must be preserved");
    assert_eq!(result.completion_date, task.completion_date);
    assert_eq!(result.creation_date, task.creation_date);
}

// ── normalize_line: inline priority lifting (NORM-01 edit mode) ──────────────

#[test]
fn inline_priority_lifted_from_body() {
    // User typed "fix bug (A) +work" — (A) is stray, must be lifted to priority field
    let result = normalize_line("fix bug (A) +work");
    assert_eq!(result.priority, Some('A'), "inline (A) must be lifted to priority");
    assert!(
        !result.body.contains("(A)"),
        "body must not contain (A) after lifting; body: {:?}",
        result.body
    );
    assert_eq!(result.projects, vec!["work"]);
    assert!(result.to_raw().starts_with("(A) "), "raw must start with (A)");
}

#[test]
fn normalize_line_standard_prefix_priority_unchanged() {
    // Standard prefix priority — must not be doubled or altered
    let result = normalize_line("(B) fix bug +work");
    assert_eq!(result.priority, Some('B'));
    assert_eq!(result.projects, vec!["work"]);
    assert!(result.to_raw().starts_with("(B) "));
}

#[test]
fn normalize_line_malformed_priority_stays_in_body() {
    // T-21-01: ((A)) and (AB) are NOT valid priority tokens — must stay in body
    let result = normalize_line("fix bug ((A)) more");
    assert_eq!(result.priority, None, "((A)) must not be parsed as priority");
    assert!(result.body.contains("((A))"));

    let result2 = normalize_line("fix bug (AB) more");
    assert_eq!(result2.priority, None, "(AB) must not be parsed as priority");
    assert!(result2.body.contains("(AB)"));
}

#[test]
fn normalize_line_completed_task_preserved() {
    // Completed flag and dates must round-trip through normalize_line
    let result = normalize_line("x 2026-01-10 2026-01-05 fix bug +work");
    assert!(result.completed);
    assert_eq!(
        result.completion_date,
        Some(NaiveDate::from_ymd_opt(2026, 1, 10).unwrap())
    );
    assert_eq!(
        result.creation_date,
        Some(NaiveDate::from_ymd_opt(2026, 1, 5).unwrap())
    );
    assert_eq!(result.projects, vec!["work"]);
}

use chrono::NaiveDate;
use rstest::rstest;
use todotxt_core::{DueStatus, Task};

// ── Round-trip tests ──────────────────────────────────────────────────────────

/// Every line in the fixture file must survive parse → to_string unchanged.
#[test]
fn round_trip_all_fixture_lines() {
    let fixture = include_str!("fixtures/todo.txt");
    for line in fixture.lines() {
        let task = Task::parse(line);
        assert_eq!(
            task.to_string(),
            line,
            "round-trip failed for: {:?}",
            line
        );
    }
}

// ── Field extraction ──────────────────────────────────────────────────────────

#[test]
fn parse_full_line_all_fields() {
    let task = Task::parse("(A) 2024-01-15 Call dentist +Health @phone due:2024-01-31");
    assert_eq!(task.priority, Some('A'));
    assert_eq!(task.creation_date, Some(date(2024, 1, 15)));
    assert_eq!(task.body, "Call dentist");
    assert_eq!(task.projects, vec!["Health"]);
    assert_eq!(task.contexts, vec!["phone"]);
    assert_eq!(task.due_date, Some(date(2024, 1, 31)));
    assert!(!task.completed);
    assert_eq!(task.completion_date, None);
}

#[test]
fn parse_completed_with_dates() {
    let task = Task::parse("x 2024-01-10 2024-01-05 Pay bills +Finance @home");
    assert!(task.completed);
    assert_eq!(task.completion_date, Some(date(2024, 1, 10)));
    assert_eq!(task.creation_date, Some(date(2024, 1, 5)));
    assert_eq!(task.body, "Pay bills");
    assert_eq!(task.projects, vec!["Finance"]);
    assert_eq!(task.contexts, vec!["home"]);
}

#[test]
fn parse_threshold_date() {
    let task = Task::parse("(B) Write report +Work @office t:2024-02-01");
    assert_eq!(task.priority, Some('B'));
    assert_eq!(task.threshold_date, Some(date(2024, 2, 1)));
    assert_eq!(task.projects, vec!["Work"]);
    assert_eq!(task.contexts, vec!["office"]);
}

#[test]
fn parse_plain_task_no_structure() {
    let task = Task::parse("Buy groceries +Personal @errands");
    assert!(!task.completed);
    assert_eq!(task.priority, None);
    assert_eq!(task.creation_date, None);
    assert_eq!(task.body, "Buy groceries");
    assert_eq!(task.projects, vec!["Personal"]);
    assert_eq!(task.contexts, vec!["errands"]);
}

#[test]
fn parse_completed_no_date() {
    let task = Task::parse("x completed without date");
    assert!(task.completed);
    assert_eq!(task.completion_date, None);
    assert_eq!(task.body, "completed without date");
}

#[test]
fn parse_empty_string() {
    let task = Task::parse("");
    assert!(!task.completed);
    assert_eq!(task.priority, None);
    assert_eq!(task.creation_date, None);
    assert_eq!(task.body, "");
    assert_eq!(task.to_string(), "");
}

#[test]
fn parse_creation_date_only() {
    let task = Task::parse("2024-06-01 Task with creation date only");
    assert_eq!(task.creation_date, Some(date(2024, 6, 1)));
    assert_eq!(task.body, "Task with creation date only");
    assert_eq!(task.priority, None);
}

#[test]
fn parse_multiple_projects_and_contexts_sorted() {
    let task = Task::parse("(C) Task with multiple +proj1 +proj2 @ctx1 @ctx2 due:2024-03-15");
    // BTreeSet ensures sorted order
    assert_eq!(task.projects, vec!["proj1", "proj2"]);
    assert_eq!(task.contexts, vec!["ctx1", "ctx2"]);
    assert_eq!(task.due_date, Some(date(2024, 3, 15)));
}

// ── Priority case-sensitivity ─────────────────────────────────────────────────

#[rstest]
#[case("(A) uppercase priority", Some('A'))]
#[case("(Z) uppercase Z priority", Some('Z'))]
#[case("(a) lowercase is NOT priority", None)]
#[case("(1) digit is NOT priority", None)]
#[case("(AB) two chars is NOT priority", None)]
fn priority_case_sensitivity(#[case] line: &str, #[case] expected: Option<char>) {
    let task = Task::parse(line);
    assert_eq!(
        task.priority, expected,
        "priority mismatch for: {:?}",
        line
    );
}

// ── Completed marker case-sensitivity ────────────────────────────────────────

#[rstest]
#[case("x task", true)]
#[case("X task", false)] // uppercase X is NOT completed per todo.txt standard
#[case("xnotcompleted", false)]
fn completed_case_sensitivity(#[case] line: &str, #[case] expected: bool) {
    let task = Task::parse(line);
    assert_eq!(task.completed, expected, "for: {:?}", line);
}

// ── Builder methods ───────────────────────────────────────────────────────────

#[test]
fn with_priority_adds_priority() {
    let task = Task::parse("Buy milk");
    let updated = task.with_priority(Some('A'));
    assert_eq!(updated.priority, Some('A'));
    assert_eq!(updated.body, "Buy milk");
    assert!(updated.to_string().starts_with("(A) "));
}

#[test]
fn with_priority_removes_priority() {
    let task = Task::parse("(B) Buy milk");
    let updated = task.with_priority(None);
    assert_eq!(updated.priority, None);
    assert_eq!(updated.body, "Buy milk");
    assert!(!updated.to_string().contains("(B)"));
}

#[test]
fn with_priority_changes_priority() {
    let task = Task::parse("(A) Buy milk");
    let updated = task.with_priority(Some('C'));
    assert_eq!(updated.priority, Some('C'));
    assert!(updated.to_string().starts_with("(C) "));
    assert!(!updated.to_string().contains("(A)"));
}

#[test]
fn with_completed_sets_completion_date() {
    let task = Task::parse("(A) Buy milk");
    let completed = task.with_completed(true);
    assert!(completed.completed);
    assert!(completed.completion_date.is_some());
    // Priority should be stripped on completion
    assert_eq!(completed.priority, None);
    assert!(!completed.to_string().contains("(A)"));
    assert!(completed.to_string().starts_with("x "));
}

#[test]
fn with_completed_false_clears_completed() {
    let task = Task::parse("x 2024-01-10 Buy milk");
    let active = task.with_completed(false);
    assert!(!active.completed);
    assert_eq!(active.completion_date, None);
}

#[test]
fn with_due_date_adds_due_date() {
    let task = Task::parse("Buy milk");
    let d = date(2025, 3, 15);
    let updated = task.with_due_date(Some(d));
    assert_eq!(updated.due_date, Some(d));
    assert!(updated.to_string().contains("due:2025-03-15"));
}

#[test]
fn with_due_date_removes_due_date() {
    let task = Task::parse("Buy milk due:2025-03-15");
    let updated = task.with_due_date(None);
    assert_eq!(updated.due_date, None);
    assert!(!updated.to_string().contains("due:"));
}

#[test]
fn with_due_date_updates_existing() {
    let task = Task::parse("Buy milk due:2025-01-01");
    let new_date = date(2025, 12, 31);
    let updated = task.with_due_date(Some(new_date));
    assert_eq!(updated.due_date, Some(new_date));
    assert!(updated.to_string().contains("due:2025-12-31"));
    assert!(!updated.to_string().contains("due:2025-01-01"));
}

#[test]
fn with_creation_date_sets_date() {
    let task = Task::parse("Buy milk");
    let d = date(2024, 6, 1);
    let updated = task.with_creation_date(Some(d));
    assert_eq!(updated.creation_date, Some(d));
    // Creation date should appear before body
    assert!(updated.to_string().contains("2024-06-01 Buy milk"));
}

// ── DueStatus ─────────────────────────────────────────────────────────────────

#[test]
fn due_status_not_due_when_no_due_date() {
    let task = Task::parse("Buy milk");
    assert_eq!(task.due_status(), DueStatus::NotDue);
}

#[test]
fn due_status_not_due_when_completed() {
    let task = Task::parse("x 2024-01-01 Buy milk due:2020-01-01");
    assert_eq!(task.due_status(), DueStatus::NotDue);
}

#[test]
fn due_status_overdue_for_past_date() {
    let task = Task::parse("Buy milk due:2000-01-01");
    assert_eq!(task.due_status(), DueStatus::Overdue);
}

#[test]
fn due_status_not_due_for_far_future() {
    let task = Task::parse("Buy milk due:2999-12-31");
    assert_eq!(task.due_status(), DueStatus::NotDue);
}

// ── Display and to_raw ────────────────────────────────────────────────────────

#[test]
fn display_equals_to_raw_equals_original() {
    let line = "(A) 2024-01-15 Call dentist +Health @phone due:2024-01-31";
    let task = Task::parse(line);
    assert_eq!(task.to_raw(), line);
    assert_eq!(task.to_string(), line);
    assert_eq!(format!("{task}"), line);
}

// ── Insta snapshot tests ──────────────────────────────────────────────────────

#[test]
fn snapshot_full_task_display() {
    let line = "(A) 2024-01-15 Call dentist +Health @phone due:2024-01-31";
    let task = Task::parse(line);
    insta::assert_snapshot!(task.to_string());
}

#[test]
fn snapshot_full_task_json() {
    let line = "(A) 2024-01-15 Call dentist +Health @phone due:2024-01-31";
    let task = Task::parse(line);
    insta::assert_json_snapshot!(task);
}

// ── CR normalization regression tests (UAT gap 03-04) ───────────────────────

/// to_raw() must never include a trailing '\r' even when the input line has CRLF ending.
#[test]
fn parse_crlf_line_raw_has_no_trailing_cr() {
    let task = Task::parse("Buy milk\r");
    assert_eq!(
        task.to_raw(),
        "Buy milk",
        "to_raw() must not contain trailing \\r"
    );
    assert_eq!(task.to_string(), "Buy milk");
    assert!(!task.to_raw().contains('\r'));
}

/// Completed task parsed from CRLF line must store CR-free raw.
#[test]
fn parse_completed_crlf_line_raw_has_no_trailing_cr() {
    let task = Task::parse("x 2024-01-10 2024-01-05 Pay bills +Finance @home\r");
    assert_eq!(
        task.to_raw(),
        "x 2024-01-10 2024-01-05 Pay bills +Finance @home"
    );
    assert!(task.completed);
    assert_eq!(task.body, "Pay bills");
    assert!(!task.to_raw().contains('\r'));
}

/// Line with only '\r' produces an empty task with no CR in raw.
#[test]
fn parse_bare_cr_produces_empty_raw() {
    let task = Task::parse("\r");
    assert_eq!(task.to_raw(), "");
    assert!(!task.to_raw().contains('\r'));
}

// ── Helper ────────────────────────────────────────────────────────────────────

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

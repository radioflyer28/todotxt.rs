use std::collections::BTreeSet;
use std::fmt;

use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use winnow::{combinator::opt, error::ContextError, prelude::*, error::ModalResult};

/// Whether a task is overdue, due today, or not due.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DueStatus {
    NotDue,
    Today,
    Overdue,
}

/// A single todo.txt task.
///
/// `raw` stores the original line verbatim for perfect round-trip fidelity.
/// All public fields are parsed from `raw` and kept in sync via `with_*` builders.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Task {
    /// Original source line — the canonical serialization form.
    #[serde(skip)]
    raw: String,
    pub completed: bool,
    pub priority: Option<char>,
    pub creation_date: Option<NaiveDate>,
    pub completion_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub threshold_date: Option<NaiveDate>,
    /// Sorted, deduplicated project tags (without the leading `+`).
    pub projects: Vec<String>,
    /// Sorted, deduplicated context tags (without the leading `@`).
    pub contexts: Vec<String>,
    /// Body text with tags (`+proj`, `@ctx`, `due:`, `t:`) removed.
    pub body: String,
}

impl Task {
    /// Parse a todo.txt line into a `Task`. **Infallible** — any input is accepted.
    ///
    /// Lines that do not follow the todo.txt format produce a Task with all
    /// structured fields set to their defaults and `body` containing the full line.
    pub fn parse(line: &str) -> Self {
        // Strip trailing CR before storing raw so to_raw() never returns '\r'-terminated strings.
        let normalized = line.trim_end_matches('\r');
        let raw = normalized.to_string();
        // Parse using the CR-stripped view so CRLF lines parse identically to LF lines.
        let mut rest: &str = normalized;

        // 1. Completed marker: lowercase "x " prefix only (standard todo.txt).
        let completed = if rest.starts_with("x ") {
            rest = &rest[2..];
            true
        } else {
            false
        };

        // 2. Completion date — only present in completed tasks.
        let completion_date = if completed {
            parse_date_prefix(&mut rest)
        } else {
            None
        };

        // 3. Priority: "(A) " where A ∈ [A-Z] only (case-sensitive per CONTEXT.md).
        let priority = parse_priority_prefix(&mut rest);

        // 4. Creation date.
        let creation_date = parse_date_prefix(&mut rest);

        // 5. Remaining text — extract structured tags and body.
        let (body, projects, contexts, due_date, threshold_date) = extract_tags(rest);

        Task {
            raw,
            completed,
            priority,
            creation_date,
            completion_date,
            due_date,
            threshold_date,
            projects,
            contexts,
            body,
        }
    }

    /// Returns the original raw line (canonical serialization form).
    pub fn to_raw(&self) -> &str {
        &self.raw
    }

    /// Returns whether this task is overdue, due today, or not due.
    pub fn due_status(&self) -> DueStatus {
        if self.completed {
            return DueStatus::NotDue;
        }
        match self.due_date {
            None => DueStatus::NotDue,
            Some(due) => {
                let today = Local::now().date_naive();
                if due < today {
                    DueStatus::Overdue
                } else if due == today {
                    DueStatus::Today
                } else {
                    DueStatus::NotDue
                }
            }
        }
    }

    // ── Builder methods (value-consuming, per CONTEXT.md Decision 4) ─────────

    /// Mark or unmark this task as completed.
    ///
    /// When marking complete, today's date is set as the completion date and the
    /// priority is stripped (per todo.txt spec).
    /// When unmarking, the completion date is cleared.
    pub fn with_completed(self, completed: bool) -> Self {
        if self.completed == completed {
            return self;
        }
        if completed {
            let today = Local::now().date_naive();
            let new_task = Task {
                completed: true,
                priority: None, // stripped on completion per spec
                completion_date: Some(today),
                ..self
            };
            let new_raw = rebuild_raw(&new_task);
            Task::parse(&new_raw)
        } else {
            let new_task = Task {
                completed: false,
                completion_date: None,
                ..self
            };
            let new_raw = rebuild_raw(&new_task);
            Task::parse(&new_raw)
        }
    }

    /// Set or clear the priority, updating the raw string.
    pub fn with_priority(self, priority: Option<char>) -> Self {
        let new_task = Task { priority, ..self };
        let new_raw = rebuild_raw(&new_task);
        Task::parse(&new_raw)
    }

    /// Set or clear the creation date.
    pub fn with_creation_date(self, date: Option<NaiveDate>) -> Self {
        let new_task = Task { creation_date: date, ..self };
        let new_raw = rebuild_raw(&new_task);
        Task::parse(&new_raw)
    }

    /// Set or clear the due date (`due:YYYY-MM-DD` tag in body).
    pub fn with_due_date(self, date: Option<NaiveDate>) -> Self {
        let new_task = Task { due_date: date, ..self };
        let new_raw = rebuild_raw(&new_task);
        Task::parse(&new_raw)
    }

    /// Set or clear the threshold date (`t:YYYY-MM-DD` tag in body).
    pub fn with_threshold_date(self, date: Option<NaiveDate>) -> Self {
        let new_task = Task { threshold_date: date, ..self };
        let new_raw = rebuild_raw(&new_task);
        Task::parse(&new_raw)
    }

    /// Prepend `text` before the task body, after completion marker, priority, and date prefixes.
    ///
    /// The inserted text is separated from the existing body by a single space.
    pub fn with_text_prepended(self, text: &str) -> Self {
        let new_body = format!("{} {}", text, self.body);
        let new_task = Task {
            body: new_body,
            ..self
        };
        let new_raw = rebuild_raw(&new_task);
        Task::parse(&new_raw)
    }
}

// ── Normalization helpers for append and edit flows ──────────────────────────

/// Parse `append_text` for recognized todo.txt tokens, merge into `task`'s fields,
/// and return a new Task rebuilt via `rebuild_raw()`.
///
/// Merge rules (from 21-CONTEXT.md):
/// - **Priority (D-03/D-04):** appended priority wins; if none in append_text, original kept.
/// - **Projects/Contexts (NORM-02/03):** union — both original and appended; deduped via BTreeSet.
/// - **due_date / threshold_date (NORM-04):** appended wins if Some; otherwise original kept.
/// - **body (NORM-05):** appended body words concatenated after original body.
///   Unrecognized tokens (`rec:+1w`, `foo:bar`) land in `appended.body` by default —
///   they are preserved verbatim here.
/// - **completed / creation_date / completion_date:** always taken from `task` (not changed by append).
pub fn normalize_append(task: &Task, append_text: &str) -> Task {
    // Parse append_text — extract tokens into structured fields using existing parse logic.
    // Unknown tokens that extract_tags doesn't recognize land in appended.body (NORM-05/D-02).
    // Note: If append_text is just "(A)" without trailing space, the parser won't recognize it as priority.
    // We handle this by appending a space for parsing purposes.
    let parse_text = if append_text.len() == 3 
        && append_text.as_bytes()[0] == b'('
        && append_text.as_bytes()[1].is_ascii_uppercase()
        && append_text.as_bytes()[2] == b')'
    {
        // append_text is exactly "(X)" — add a space so the parser recognizes it as priority
        format!("{} ", append_text)
    } else {
        append_text.to_string()
    };
    let appended = Task::parse(&parse_text);

    // D-03/D-04: appended priority wins when present; else keep original.
    let priority = if appended.priority.is_some() {
        appended.priority
    } else {
        task.priority
    };

    // NORM-02: union of projects — BTreeSet for stable sort + dedup (mirrors parse behavior).
    let mut projects: BTreeSet<String> = task.projects.iter().cloned().collect();
    for p in &appended.projects {
        projects.insert(p.clone());
    }

    // NORM-03: union of contexts — same pattern.
    let mut contexts: BTreeSet<String> = task.contexts.iter().cloned().collect();
    for c in &appended.contexts {
        contexts.insert(c.clone());
    }

    // NORM-04: appended date wins if present; else keep original.
    let due_date = appended.due_date.or(task.due_date);
    let threshold_date = appended.threshold_date.or(task.threshold_date);

    // NORM-05: body = original body + appended body words (plain text + unrecognized tokens).
    let body = match (task.body.is_empty(), appended.body.is_empty()) {
        (true, _)  => appended.body.clone(),
        (_, true)  => task.body.clone(),
        _           => format!("{} {}", task.body, appended.body),
    };

    // Build the merged Task. `raw` is a private field — we are in the same module (task.rs).
    // rebuild_raw will set the canonical raw string; Task::parse re-syncs all fields.
    let merged = Task {
        raw: String::new(),
        completed: task.completed,
        priority,
        creation_date: task.creation_date,
        completion_date: task.completion_date,
        due_date,
        threshold_date,
        projects: projects.into_iter().collect(),
        contexts: contexts.into_iter().collect(),
        body,
    };

    let raw = rebuild_raw(&merged);
    Task::parse(&raw)
}

/// Normalize a complete task line: standard `Task::parse` plus inline priority detection.
///
/// `Task::parse` only recognizes priority when it appears as `(X) ` at the VERY START
/// of the line (after optional completion marker and dates). When a user edits a task
/// and types `"fix bug (A) +work"`, parse puts `(A)` in body.
///
/// `normalize_line` additionally scans `body` for the first `(X)` word (exactly 3 bytes:
/// open paren, single ASCII uppercase letter, close paren) and lifts it to the `priority`
/// field when no priority was found in the standard prefix position.
///
/// T-21-01: `((A))` is 5 bytes, `(AB)` is 4 bytes — neither matches the 3-byte check;
/// both stay in body (no panic).
pub fn normalize_line(text: &str) -> Task {
    let task = Task::parse(text);
    // If parse already found a priority in the standard position, nothing to lift.
    if task.priority.is_some() {
        return task;
    }
    // Scan body words for a stray "(X)" priority token.
    let words: Vec<&str> = task.body.split_whitespace().collect();
    let mut found_priority: Option<char> = None;
    let mut remaining: Vec<&str> = Vec::new();
    for word in &words {
        if found_priority.is_none() {
            let b = word.as_bytes();
            // Exactly 3 bytes: '(' + single ASCII uppercase + ')' — T-21-01 safety.
            if b.len() == 3 && b[0] == b'(' && b[2] == b')' && b[1].is_ascii_uppercase() {
                found_priority = Some(b[1] as char);
                continue; // skip — this word becomes the priority field
            }
        }
        remaining.push(word);
    }
    if let Some(p) = found_priority {
        // Re-build task with lifted priority and body without the priority token.
        let updated = Task {
            raw: String::new(),
            completed: task.completed,
            priority: Some(p),
            creation_date: task.creation_date,
            completion_date: task.completion_date,
            due_date: task.due_date,
            threshold_date: task.threshold_date,
            projects: task.projects.clone(),
            contexts: task.contexts.clone(),
            body: remaining.join(" "),
        };
        let raw = rebuild_raw(&updated);
        Task::parse(&raw)
    } else {
        task
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

// ── Winnow-backed prefix parser ───────────────────────────────────────────────

/// Winnow inner parser: matches "YYYY-MM-DD " and returns a `NaiveDate`.
///
/// Uses `PResult` so it integrates cleanly with `opt()` for backtracking.
fn winnow_date_inner(input: &mut &str) -> ModalResult<NaiveDate> {
    let s = *input;
    if s.len() < 11 {
        return Err(winnow::error::ErrMode::Backtrack(ContextError::new()));
    }
    let bytes = s.as_bytes();
    // Verify YYYY-MM-DD + space pattern
    if bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b' '
        || !bytes[0..4].iter().all(|&b| b.is_ascii_digit())
        || !bytes[5..7].iter().all(|&b| b.is_ascii_digit())
        || !bytes[8..10].iter().all(|&b| b.is_ascii_digit())
    {
        return Err(winnow::error::ErrMode::Backtrack(ContextError::new()));
    }
    match NaiveDate::parse_from_str(&s[..10], "%Y-%m-%d") {
        Ok(date) => {
            *input = &s[11..];
            Ok(date)
        }
        Err(_) => Err(winnow::error::ErrMode::Backtrack(ContextError::new())),
    }
}

/// Parse "YYYY-MM-DD " at the start of `s`, advance `s`, return the date.
/// Returns `None` (and leaves `s` unchanged) if no valid date prefix is present.
fn parse_date_prefix(s: &mut &str) -> Option<NaiveDate> {
    opt(winnow_date_inner).parse_next(s).unwrap_or_default()
}

// ── Manual prefix parsers ─────────────────────────────────────────────────────

/// Parse "(X) " priority prefix where X ∈ [A-Z] (uppercase only).
/// Returns `None` (and leaves `s` unchanged) for any other prefix.
fn parse_priority_prefix(s: &mut &str) -> Option<char> {
    if s.len() >= 4 {
        let b = s.as_bytes();
        if b[0] == b'(' && b[1].is_ascii_uppercase() && b[2] == b')' && b[3] == b' ' {
            let c = b[1] as char;
            *s = &s[4..];
            return Some(c);
        }
    }
    None
}

// ── Body tag extractor ────────────────────────────────────────────────────────

/// Split the body text into structured fields.
///
/// Tags (`+proj`, `@ctx`, `due:YYYY-MM-DD`, `t:YYYY-MM-DD`) are extracted into
/// their typed fields. All non-tag words form the returned body string.
/// Projects and contexts are sorted and deduplicated (matching C# `SortedSet`).
fn extract_tags(
    body: &str,
) -> (
    String,
    Vec<String>,
    Vec<String>,
    Option<NaiveDate>,
    Option<NaiveDate>,
) {
    let mut projects = BTreeSet::new();
    let mut contexts = BTreeSet::new();
    let mut due_date: Option<NaiveDate> = None;
    let mut threshold_date: Option<NaiveDate> = None;
    let mut body_words: Vec<&str> = Vec::new();

    for word in body.split_whitespace() {
        if let Some(proj) = word.strip_prefix('+') {
            if !proj.is_empty() {
                projects.insert(proj.to_string());
                continue;
            }
        } else if let Some(ctx) = word.strip_prefix('@') {
            if !ctx.is_empty() {
                contexts.insert(ctx.to_string());
                continue;
            }
        } else if let Some(date_str) = word.strip_prefix("due:") {
            if let Ok(d) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                due_date = Some(d);
                continue;
            }
        } else if let Some(date_str) = word.strip_prefix("t:") {
            if let Ok(d) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                threshold_date = Some(d);
                continue;
            }
        }
        body_words.push(word);
    }

    (
        body_words.join(" "),
        projects.into_iter().collect(),
        contexts.into_iter().collect(),
        due_date,
        threshold_date,
    )
}

// ── Builder helpers ───────────────────────────────────────────────────────────

/// Rebuild the raw todo.txt line from all parsed fields.
///
/// Used by `with_*` builder methods. The returned string is then re-parsed
/// via `Task::parse` to ensure all fields stay in sync.
fn rebuild_raw(task: &Task) -> String {
    let mut result = String::new();

    if task.completed {
        if let Some(d) = task.completion_date {
            result.push_str(&format!("x {} ", d.format("%Y-%m-%d")));
        } else {
            result.push_str("x ");
        }
    }
    if let Some(p) = task.priority {
        result.push_str(&format!("({p}) "));
    }
    if let Some(d) = task.creation_date {
        result.push_str(&format!("{} ", d.format("%Y-%m-%d")));
    }
    if !task.body.is_empty() {
        result.push_str(&task.body);
    }
    for proj in &task.projects {
        if !result.is_empty() && !result.ends_with(' ') {
            result.push(' ');
        }
        result.push_str(&format!("+{proj}"));
    }
    for ctx in &task.contexts {
        if !result.is_empty() && !result.ends_with(' ') {
            result.push(' ');
        }
        result.push_str(&format!("@{ctx}"));
    }
    if let Some(d) = task.due_date {
        if !result.is_empty() && !result.ends_with(' ') {
            result.push(' ');
        }
        result.push_str(&format!("due:{}", d.format("%Y-%m-%d")));
    }
    if let Some(d) = task.threshold_date {
        if !result.is_empty() && !result.ends_with(' ') {
            result.push(' ');
        }
        result.push_str(&format!("t:{}", d.format("%Y-%m-%d")));
    }

    // Remove trailing space when body is empty and there are no tags
    result.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    /// with_priority sets new priority and preserves all other metadata fields.
    #[test]
    fn test_with_priority_preserves_metadata() {
        let task = Task::parse("(B) 2025-12-01 fix login bug @work @home +proj1 +proj2 due:2026-03-01 t:2026-02-01");
        let result = task.with_priority(Some('A'));

        assert_eq!(result.priority, Some('A'));
        assert_eq!(result.creation_date, Some(date(2025, 12, 1)));
        assert_eq!(result.due_date, Some(date(2026, 3, 1)));
        assert_eq!(result.threshold_date, Some(date(2026, 2, 1)));
        assert_eq!(result.projects, vec!["proj1".to_string(), "proj2".to_string()]);
        assert!(result.contexts.contains(&"work".to_string()));
        assert!(result.contexts.contains(&"home".to_string()));
        assert_eq!(result.completed, false);
        assert!(result.to_raw().starts_with("(A) "), "raw should start with (A): {}", result.to_raw());
        assert!(result.to_raw().contains("due:2026-03-01"), "should contain due date: {}", result.to_raw());
        assert!(!result.to_raw().contains("(B)"), "should not contain old priority: {}", result.to_raw());
        // Exactly one due: token
        assert_eq!(result.to_raw().matches("due:").count(), 1, "exactly one due: token: {}", result.to_raw());
    }

    /// with_priority(None) clears priority — no '(' token in raw output.
    #[test]
    fn test_with_priority_clears_priority() {
        let task = Task::parse("(A) 2025-12-01 fix login bug @work +proj1 due:2026-03-01");
        let result = task.with_priority(None);

        assert_eq!(result.priority, None);
        assert!(!result.to_raw().contains('('), "raw should not contain '(': {}", result.to_raw());
        assert_eq!(result.creation_date, Some(date(2025, 12, 1)));
        assert_eq!(result.due_date, Some(date(2026, 3, 1)));
        assert_eq!(result.projects, vec!["proj1".to_string()]);
        assert!(result.contexts.contains(&"work".to_string()));
    }

    /// with_priority on completed task preserves x prefix and completion_date.
    #[test]
    fn test_with_priority_on_completed_task() {
        let task = Task::parse("x 2026-01-15 2025-12-01 fix login bug @work +proj1 due:2026-03-01");
        let result = task.with_priority(Some('C'));

        assert_eq!(result.completed, true);
        assert_eq!(result.completion_date, Some(date(2026, 1, 15)));
        assert_eq!(result.creation_date, Some(date(2025, 12, 1)));
        assert_eq!(result.priority, Some('C'));
        assert!(result.to_raw().starts_with("x "), "raw should start with 'x ': {}", result.to_raw());
        assert_eq!(result.due_date, Some(date(2026, 3, 1)));
        assert_eq!(result.projects, vec!["proj1".to_string()]);
        assert!(result.contexts.contains(&"work".to_string()));
    }

    /// with_due_date replaces existing due: token — no duplicates.
    #[test]
    fn test_with_due_date_no_duplicate() {
        let task = Task::parse("(A) 2025-12-01 fix login bug @work +proj1 due:2026-03-01 t:2026-02-01");
        let result = task.with_due_date(Some(date(2026, 6, 15)));

        assert_eq!(result.due_date, Some(date(2026, 6, 15)));
        assert!(result.to_raw().contains("due:2026-06-15"), "should contain new due: {}", result.to_raw());
        assert!(!result.to_raw().contains("due:2026-03-01"), "old due: should be gone: {}", result.to_raw());
        assert_eq!(result.priority, Some('A'));
        assert_eq!(result.threshold_date, Some(date(2026, 2, 1)));
        assert_eq!(result.projects, vec!["proj1".to_string()]);
        assert!(result.contexts.contains(&"work".to_string()));
        // Exactly one due: token
        assert_eq!(result.to_raw().matches("due:").count(), 1, "exactly one due: token: {}", result.to_raw());
    }

    /// with_due_date(None) removes the due: token entirely.
    #[test]
    fn test_with_due_date_removes_due_token() {
        let task = Task::parse("(A) 2025-12-01 fix login bug +proj1 due:2026-03-01");
        let result = task.with_due_date(None);

        assert_eq!(result.due_date, None);
        assert!(!result.to_raw().contains("due:"), "raw should not contain 'due:': {}", result.to_raw());
        assert_eq!(result.priority, Some('A'));
        assert_eq!(result.projects, vec!["proj1".to_string()]);
    }

    /// with_priority preserves projects and contexts without adding duplicates.
    #[test]
    fn test_with_priority_preserves_projects_contexts() {
        let task = Task::parse("(B) 2025-12-01 fix bug @work @home +proj1 +proj2");
        let result = task.with_priority(Some('Z'));

        assert_eq!(result.priority, Some('Z'));
        assert_eq!(result.projects, vec!["proj1".to_string(), "proj2".to_string()]);
        assert_eq!(result.contexts.len(), 2);
        assert!(result.contexts.contains(&"work".to_string()));
        assert!(result.contexts.contains(&"home".to_string()));
        assert!(result.to_raw().contains("+proj1"), "raw should contain +proj1: {}", result.to_raw());
        assert!(result.to_raw().contains("+proj2"), "raw should contain +proj2: {}", result.to_raw());
        assert!(result.to_raw().contains("@work"), "raw should contain @work: {}", result.to_raw());
        assert!(result.to_raw().contains("@home"), "raw should contain @home: {}", result.to_raw());
        // No duplicates
        assert_eq!(result.to_raw().matches("+proj1").count(), 1);
        assert_eq!(result.to_raw().matches("+proj2").count(), 1);
    }
}

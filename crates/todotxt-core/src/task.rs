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

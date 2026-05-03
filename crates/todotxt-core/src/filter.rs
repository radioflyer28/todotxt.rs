use chrono::NaiveDate;
use crate::task::Task;

/// A single filter predicate parsed from a query token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterTerm {
    Include(String),
    Exclude(String),
    Done,
    NotDone,
    DueToday,
    DuePast,
    DueFuture,
    DueActive,
    NegDueToday,
    NegDuePast,
    NegDueFuture,
    NegDueActive,
}

/// A filter that can be applied to a list of tasks.
///
/// Tokens are AND-combined (all must pass). `suppress_hidden` and
/// `suppress_future_threshold` are pre-filters applied before token evaluation.
#[derive(Debug, Clone)]
pub struct Filter {
    pub terms: Vec<FilterTerm>,
    /// When `true` (default), tasks containing `h:1` in their raw text are excluded.
    pub suppress_hidden: bool,
    /// When `true` (default), tasks with a threshold date > today are excluded.
    pub suppress_future_threshold: bool,
}

impl Default for Filter {
    fn default() -> Self {
        Filter {
            terms: Vec::new(),
            suppress_hidden: true,
            suppress_future_threshold: true,
        }
    }
}

impl Filter {
    /// Create an empty filter with both suppression flags defaulting to `true`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a space-separated query string into a `Filter`.
    ///
    /// `DONE` and `-DONE` are matched case-sensitively.
    /// `due:*` tokens are case-insensitive.
    /// Other tokens become substring Include/Exclude terms (case-insensitive matching).
    pub fn from_query(q: &str) -> Self {
        let terms = q
            .split_ascii_whitespace()
            .filter(|s| !s.is_empty())
            .map(|token| {
                // Case-sensitive special tokens first
                if token == "DONE" {
                    return FilterTerm::Done;
                }
                if token == "-DONE" {
                    return FilterTerm::NotDone;
                }
                // due: tokens are case-insensitive
                match token.to_ascii_lowercase().as_str() {
                    "due:today" => return FilterTerm::DueToday,
                    "due:past" => return FilterTerm::DuePast,
                    "due:future" => return FilterTerm::DueFuture,
                    "due:active" => return FilterTerm::DueActive,
                    "-due:today" => return FilterTerm::NegDueToday,
                    "-due:past" => return FilterTerm::NegDuePast,
                    "-due:future" => return FilterTerm::NegDueFuture,
                    "-due:active" => return FilterTerm::NegDueActive,
                    _ => {}
                }
                // Negation prefix
                if let Some(rest) = token.strip_prefix('-') {
                    return FilterTerm::Exclude(rest.to_string());
                }
                FilterTerm::Include(token.to_string())
            })
            .collect();
        Filter { terms, ..Self::default() }
    }

    /// Returns `true` if this filter passes the given task.
    ///
    /// Uses the current local date for `due:*` comparisons.
    pub fn matches(&self, task: &Task) -> bool {
        let today = chrono::Local::now().date_naive();
        self.matches_with_date(task, today)
    }

    /// Like `matches`, but takes an explicit date — useful for testing.
    pub fn matches_with_date(&self, task: &Task, today: NaiveDate) -> bool {
        let raw = task.to_raw();

        // Pre-filter 1: hidden tag suppression — token-level match to avoid false positives on
        // substrings like `h:10`, `auth:1`, etc.
        if self.suppress_hidden && raw.split_ascii_whitespace().any(|t| t == "h:1") {
            return false;
        }

        // Pre-filter 2: future threshold suppression
        if self.suppress_future_threshold {
            if let Some(t) = task.threshold_date {
                if t > today {
                    return false;
                }
            }
        }

        // AND-evaluate all tokens
        for term in &self.terms {
            let passes = match term {
                FilterTerm::Done => task.completed,
                FilterTerm::NotDone => !task.completed,
                FilterTerm::DueToday => task.due_date == Some(today),
                FilterTerm::DuePast => task.due_date.is_some_and(|d| d < today),
                FilterTerm::DueFuture => task.due_date.is_some_and(|d| d > today),
                FilterTerm::DueActive => task.due_date.is_some_and(|d| d <= today),
                FilterTerm::NegDueToday => task.due_date != Some(today),
                FilterTerm::NegDuePast => task.due_date.is_none_or(|d| d >= today),
                FilterTerm::NegDueFuture => task.due_date.is_none_or(|d| d <= today),
                FilterTerm::NegDueActive => task.due_date.is_none_or(|d| d > today),
                FilterTerm::Include(s) => raw
                    .to_ascii_lowercase()
                    .contains(s.to_ascii_lowercase().as_str()),
                FilterTerm::Exclude(s) => !raw
                    .to_ascii_lowercase()
                    .contains(s.to_ascii_lowercase().as_str()),
            };
            if !passes {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use crate::task::Task;

    fn today() -> NaiveDate { chrono::Local::now().date_naive() }
    fn past() -> NaiveDate { NaiveDate::from_ymd_opt(2000, 1, 1).unwrap() }
    fn future() -> NaiveDate { NaiveDate::from_ymd_opt(2099, 12, 31).unwrap() }

    fn task(raw: &str) -> Task { Task::parse(raw) }

    // ── from_query parsing ────────────────────────────────────────────────────

    #[test]
    fn parse_done_token_case_sensitive() {
        assert_eq!(Filter::from_query("DONE").terms, vec![FilterTerm::Done]);
        // lowercase "done" is NOT the Done token — it becomes Include("done")
        assert_eq!(Filter::from_query("done").terms, vec![FilterTerm::Include("done".into())]);
    }

    #[test]
    fn parse_not_done_token() {
        assert_eq!(Filter::from_query("-DONE").terms, vec![FilterTerm::NotDone]);
    }

    #[test]
    fn parse_due_tokens_case_insensitive() {
        assert_eq!(Filter::from_query("due:today").terms, vec![FilterTerm::DueToday]);
        assert_eq!(Filter::from_query("DUE:TODAY").terms, vec![FilterTerm::DueToday]);
        assert_eq!(Filter::from_query("due:past").terms, vec![FilterTerm::DuePast]);
        assert_eq!(Filter::from_query("due:future").terms, vec![FilterTerm::DueFuture]);
        assert_eq!(Filter::from_query("due:active").terms, vec![FilterTerm::DueActive]);
    }

    #[test]
    fn parse_neg_due_tokens() {
        assert_eq!(Filter::from_query("-due:today").terms, vec![FilterTerm::NegDueToday]);
        assert_eq!(Filter::from_query("-due:past").terms, vec![FilterTerm::NegDuePast]);
        assert_eq!(Filter::from_query("-due:future").terms, vec![FilterTerm::NegDueFuture]);
        assert_eq!(Filter::from_query("-due:active").terms, vec![FilterTerm::NegDueActive]);
    }

    #[test]
    fn parse_negation_and_include() {
        let f = Filter::from_query("-@work @home");
        assert_eq!(f.terms, vec![
            FilterTerm::Exclude("@work".into()),
            FilterTerm::Include("@home".into()),
        ]);
    }

    #[test]
    fn parse_empty_query() {
        assert!(Filter::from_query("").terms.is_empty());
        assert!(Filter::from_query("   ").terms.is_empty());
    }

    // ── matches logic ─────────────────────────────────────────────────────────

    #[test]
    fn done_token_matches_only_completed() {
        let f = Filter::from_query("DONE");
        let completed = task("x 2024-01-01 Buy milk");
        let incomplete = task("Buy milk");
        assert!(f.matches_with_date(&completed, today()));
        assert!(!f.matches_with_date(&incomplete, today()));
    }

    #[test]
    fn not_done_token_matches_only_incomplete() {
        let f = Filter::from_query("-DONE");
        assert!(f.matches_with_date(&task("Buy milk"), today()));
        assert!(!f.matches_with_date(&task("x 2024-01-01 Buy milk"), today()));
    }

    #[test]
    fn due_today_matches_today_only() {
        let today = today();
        let f = Filter::from_query("due:today");
        let t_today = task(&format!("Buy milk due:{}", today));
        let t_past = task(&format!("Old task due:{}", past()));
        assert!(f.matches_with_date(&t_today, today));
        assert!(!f.matches_with_date(&t_past, today));
    }

    #[test]
    fn due_past_matches_overdue() {
        let today = today();
        let f = Filter::from_query("due:past");
        let t_past = task(&format!("Old task due:{}", past()));
        let t_future = task(&format!("Future due:{}", future()));
        assert!(f.matches_with_date(&t_past, today));
        assert!(!f.matches_with_date(&t_future, today));
    }

    #[test]
    fn due_active_matches_today_and_past() {
        let today = today();
        let f = Filter::from_query("due:active");
        let t_today = task(&format!("Task due:{}", today));
        let t_past = task(&format!("Old due:{}", past()));
        let t_future = task(&format!("Future due:{}", future()));
        assert!(f.matches_with_date(&t_today, today));
        assert!(f.matches_with_date(&t_past, today));
        assert!(!f.matches_with_date(&t_future, today));
    }

    #[test]
    fn include_term_is_case_insensitive_substring() {
        let f = Filter::from_query("@Home");
        assert!(f.matches_with_date(&task("Buy milk @home"), today()));
        assert!(!f.matches_with_date(&task("Buy milk @work"), today()));
    }

    #[test]
    fn exclude_term_negates_match() {
        let f = Filter::from_query("-@work");
        assert!(f.matches_with_date(&task("Buy milk @home"), today()));
        assert!(!f.matches_with_date(&task("Buy milk @work"), today()));
    }

    #[test]
    fn multiple_terms_are_and_combined() {
        let f = Filter::from_query("@home -DONE");
        assert!(f.matches_with_date(&task("Buy milk @home"), today()));
        assert!(!f.matches_with_date(&task("x 2024-01-01 Buy milk @home"), today()));
        assert!(!f.matches_with_date(&task("Buy milk @work"), today()));
    }

    #[test]
    fn suppress_hidden_excludes_h1_tasks() {
        let f = Filter::new(); // suppress_hidden: true by default
        assert!(!f.matches_with_date(&task("Secret task h:1"), today()));
        assert!(f.matches_with_date(&task("Normal task"), today()));
    }

    #[test]
    fn suppress_hidden_no_false_positive_on_substring() {
        // Tasks with "h:10", "h:11", or embedded substrings must NOT be suppressed
        let f = Filter::new();
        assert!(f.matches_with_date(&task("Task with h:10"), today()));
        assert!(f.matches_with_date(&task("Task auth:1 check"), today()));
    }

    #[test]
    fn suppress_hidden_off_shows_h1_tasks() {
        let f = Filter { suppress_hidden: false, ..Filter::new() };
        assert!(f.matches_with_date(&task("Secret task h:1"), today()));
    }

    #[test]
    fn suppress_future_threshold_excludes_future_threshold() {
        let today = today();
        let f = Filter::new();
        let future_threshold = task(&format!("Future task t:{}", future()));
        let past_threshold = task(&format!("Past task t:{}", past()));
        assert!(!f.matches_with_date(&future_threshold, today));
        assert!(f.matches_with_date(&past_threshold, today));
    }
}

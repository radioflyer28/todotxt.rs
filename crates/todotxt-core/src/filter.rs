use crate::task::Task;
use chrono::NaiveDate;

/// A single filter predicate parsed from a query token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterTerm {
    Include(String),
    Exclude(String),
    Or(Vec<FilterTerm>),
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
    ContextPrefix(String),
    ProjectPrefix(String),
    NegContextPrefix(String),
    NegProjectPrefix(String),
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
    /// `@foo` (no slash) → ContextPrefix; `@foo/bar` (with slash) → Include for exact match
    /// `+client` (no slash) → ProjectPrefix; `+client/acme` (with slash) → Include for exact match
    /// Negated forms work similarly (`-@foo`, `-+client`, etc.)
    /// Other tokens become substring Include/Exclude terms (case-insensitive matching).
    pub fn from_query(q: &str) -> Self {
        let terms = q
            .split_ascii_whitespace()
            .filter(|s| !s.is_empty())
            .filter_map(Self::parse_token)
            .collect();
        Filter {
            terms,
            ..Self::default()
        }
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
            let passes = Self::eval_term(term, task, &raw, today);
            if !passes {
                return false;
            }
        }

        true
    }

    fn parse_token(token: &str) -> Option<FilterTerm> {
        if token.starts_with("-(") {
            return Some(Self::parse_single_token(token));
        }

        if token.contains('|') {
            let inner: Vec<FilterTerm> = token
                .split('|')
                .filter(|part| !part.is_empty())
                .map(Self::parse_single_token)
                .collect();

            return match inner.len() {
                0 => None,
                1 => inner.into_iter().next(),
                _ => Some(FilterTerm::Or(inner)),
            };
        }

        Some(Self::parse_single_token(token))
    }

    fn parse_single_token(token: &str) -> FilterTerm {
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

        if let Some(context_name) = token.strip_prefix("-@") {
            if !context_name.contains('/') {
                return FilterTerm::NegContextPrefix(context_name.to_string());
            }
        }

        if let Some(rest) = token.strip_prefix('@') {
            if !rest.contains('/') {
                return FilterTerm::ContextPrefix(rest.to_string());
            }
        }

        if let Some(project_name) = token.strip_prefix("-+") {
            if !project_name.contains('/') {
                return FilterTerm::NegProjectPrefix(project_name.to_string());
            }
        }

        if let Some(rest) = token.strip_prefix('+') {
            if !rest.contains('/') {
                return FilterTerm::ProjectPrefix(rest.to_string());
            }
        }

        if let Some(rest) = token.strip_prefix('-') {
            return FilterTerm::Exclude(rest.to_string());
        }

        FilterTerm::Include(token.to_string())
    }

    fn eval_term(term: &FilterTerm, task: &Task, raw: &str, today: NaiveDate) -> bool {
        match term {
            FilterTerm::Done => task.completed,
            FilterTerm::NotDone => !task.completed,
            FilterTerm::Or(inner) => inner
                .iter()
                .any(|inner_term| Self::eval_term(inner_term, task, raw, today)),
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
            FilterTerm::ContextPrefix(prefix) => {
                let prefix_lower = prefix.to_ascii_lowercase();
                task.contexts.iter().any(|ctx| {
                    let ctx_lower = ctx.to_ascii_lowercase();
                    ctx_lower == prefix_lower
                        || ctx_lower.starts_with(&format!("{}/", prefix_lower))
                })
            }
            FilterTerm::ProjectPrefix(prefix) => {
                let prefix_lower = prefix.to_ascii_lowercase();
                task.projects.iter().any(|proj| {
                    let proj_lower = proj.to_ascii_lowercase();
                    proj_lower == prefix_lower
                        || proj_lower.starts_with(&format!("{}/", prefix_lower))
                })
            }
            FilterTerm::NegContextPrefix(prefix) => {
                let prefix_lower = prefix.to_ascii_lowercase();
                !task.contexts.iter().any(|ctx| {
                    let ctx_lower = ctx.to_ascii_lowercase();
                    ctx_lower == prefix_lower
                        || ctx_lower.starts_with(&format!("{}/", prefix_lower))
                })
            }
            FilterTerm::NegProjectPrefix(prefix) => {
                let prefix_lower = prefix.to_ascii_lowercase();
                !task.projects.iter().any(|proj| {
                    let proj_lower = proj.to_ascii_lowercase();
                    proj_lower == prefix_lower
                        || proj_lower.starts_with(&format!("{}/", prefix_lower))
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Task;
    use chrono::NaiveDate;

    fn today() -> NaiveDate {
        chrono::Local::now().date_naive()
    }
    fn past() -> NaiveDate {
        NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()
    }
    fn future() -> NaiveDate {
        NaiveDate::from_ymd_opt(2099, 12, 31).unwrap()
    }

    fn task(raw: &str) -> Task {
        Task::parse(raw)
    }

    // ── from_query parsing ────────────────────────────────────────────────────

    #[test]
    fn parse_done_token_case_sensitive() {
        assert_eq!(Filter::from_query("DONE").terms, vec![FilterTerm::Done]);
        // lowercase "done" is NOT the Done token — it becomes Include("done")
        assert_eq!(
            Filter::from_query("done").terms,
            vec![FilterTerm::Include("done".into())]
        );
    }

    #[test]
    fn parse_not_done_token() {
        assert_eq!(Filter::from_query("-DONE").terms, vec![FilterTerm::NotDone]);
    }

    #[test]
    fn parse_due_tokens_case_insensitive() {
        assert_eq!(
            Filter::from_query("due:today").terms,
            vec![FilterTerm::DueToday]
        );
        assert_eq!(
            Filter::from_query("DUE:TODAY").terms,
            vec![FilterTerm::DueToday]
        );
        assert_eq!(
            Filter::from_query("due:past").terms,
            vec![FilterTerm::DuePast]
        );
        assert_eq!(
            Filter::from_query("due:future").terms,
            vec![FilterTerm::DueFuture]
        );
        assert_eq!(
            Filter::from_query("due:active").terms,
            vec![FilterTerm::DueActive]
        );
    }

    #[test]
    fn parse_neg_due_tokens() {
        assert_eq!(
            Filter::from_query("-due:today").terms,
            vec![FilterTerm::NegDueToday]
        );
        assert_eq!(
            Filter::from_query("-due:past").terms,
            vec![FilterTerm::NegDuePast]
        );
        assert_eq!(
            Filter::from_query("-due:future").terms,
            vec![FilterTerm::NegDueFuture]
        );
        assert_eq!(
            Filter::from_query("-due:active").terms,
            vec![FilterTerm::NegDueActive]
        );
    }

    #[test]
    fn parse_negation_and_include() {
        let f = Filter::from_query("-@work @home");
        assert_eq!(
            f.terms,
            vec![
                FilterTerm::NegContextPrefix("work".into()),
                FilterTerm::ContextPrefix("home".into()),
            ]
        );
    }

    #[test]
    fn parse_or_context_token() {
        let f = Filter::from_query("@work|@home");
        assert_eq!(
            f.terms,
            vec![FilterTerm::Or(vec![
                FilterTerm::ContextPrefix("work".into()),
                FilterTerm::ContextPrefix("home".into()),
            ])]
        );
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
    fn or_two_contexts_matches_either_context() {
        let f = Filter::from_query("@work|@home");
        assert!(f.matches_with_date(&task("Task @work"), today()));
        assert!(f.matches_with_date(&task("Task @home"), today()));
        assert!(!f.matches_with_date(&task("Task @gym"), today()));
    }

    #[test]
    fn or_two_projects_matches_either_project() {
        let f = Filter::from_query("+client|+ops");
        assert!(f.matches_with_date(&task("Task +client"), today()));
        assert!(f.matches_with_date(&task("Task +ops"), today()));
        assert!(!f.matches_with_date(&task("Task +sales"), today()));
    }

    #[test]
    fn or_two_priorities_matches_a_or_b() {
        let f = Filter::from_query("(A)|(B)");
        assert!(f.matches_with_date(&task("(A) Alpha task"), today()));
        assert!(f.matches_with_date(&task("(B) Beta task"), today()));
        assert!(!f.matches_with_date(&task("(C) Gamma task"), today()));
    }

    #[test]
    fn or_and_combined_preserves_existing_and_semantics() {
        let f = Filter::from_query("(A)|(B) @work");
        assert!(f.matches_with_date(&task("(A) Alpha @work"), today()));
        assert!(f.matches_with_date(&task("(B) Beta @work"), today()));
        assert!(!f.matches_with_date(&task("(A) Alpha @home"), today()));
        assert!(!f.matches_with_date(&task("(C) Gamma @work"), today()));
    }

    #[test]
    fn or_empty_branch_trailing_is_ignored() {
        let f = Filter::from_query("@work|");
        assert_eq!(f.terms, vec![FilterTerm::ContextPrefix("work".into())]);
        assert!(f.matches_with_date(&task("Task @work"), today()));
        assert!(!f.matches_with_date(&task("Task @home"), today()));
    }

    #[test]
    fn or_empty_branch_leading_is_ignored() {
        let f = Filter::from_query("|@home");
        assert_eq!(f.terms, vec![FilterTerm::ContextPrefix("home".into())]);
        assert!(f.matches_with_date(&task("Task @home"), today()));
        assert!(!f.matches_with_date(&task("Task @work"), today()));
    }

    #[test]
    fn grouped_negation_is_not_special_or_syntax() {
        let f = Filter::from_query("-(@work|@home)");
        assert_eq!(f.terms, vec![FilterTerm::Exclude("(@work|@home)".into())]);
        assert!(f.matches_with_date(&task("Task @work"), today()));
        assert!(f.matches_with_date(&task("Task @home"), today()));
    }

    #[test]
    fn existing_and_unchanged() {
        let f = Filter::from_query("@work +client");
        assert!(f.matches_with_date(&task("Task @work +client"), today()));
        assert!(!f.matches_with_date(&task("Task @work +other"), today()));
        assert!(!f.matches_with_date(&task("Task @home +client"), today()));
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
        let f = Filter {
            suppress_hidden: false,
            ..Filter::new()
        };
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

    // ── hierarchical tag prefix matching (META-02) ─────────────────────────────

    #[test]
    fn parse_context_prefix_no_slash() {
        assert_eq!(
            Filter::from_query("@email").terms,
            vec![FilterTerm::ContextPrefix("email".into())]
        );
    }

    #[test]
    fn parse_exact_context_with_slash() {
        assert_eq!(
            Filter::from_query("@email/waiting").terms,
            vec![FilterTerm::Include("@email/waiting".into())]
        );
    }

    #[test]
    fn parse_project_prefix_no_slash() {
        assert_eq!(
            Filter::from_query("+client").terms,
            vec![FilterTerm::ProjectPrefix("client".into())]
        );
    }

    #[test]
    fn parse_exact_project_with_slash() {
        assert_eq!(
            Filter::from_query("+client/acme").terms,
            vec![FilterTerm::Include("+client/acme".into())]
        );
    }

    #[test]
    fn parse_negated_context_prefix() {
        assert_eq!(
            Filter::from_query("-@email").terms,
            vec![FilterTerm::NegContextPrefix("email".into())]
        );
    }

    #[test]
    fn parse_negated_project_prefix() {
        assert_eq!(
            Filter::from_query("-+client").terms,
            vec![FilterTerm::NegProjectPrefix("client".into())]
        );
    }

    #[test]
    fn context_prefix_matches_exact_context() {
        let f = Filter::from_query("@email");
        assert!(f.matches_with_date(&task("Buy milk @email"), today()));
    }

    #[test]
    fn context_prefix_matches_hierarchical_context() {
        let f = Filter::from_query("@email");
        assert!(f.matches_with_date(&task("Waiting @email/waiting"), today()));
        assert!(f.matches_with_date(&task("Forward @email/forward"), today()));
    }

    #[test]
    fn context_prefix_no_match_different_prefix() {
        let f = Filter::from_query("@email");
        assert!(!f.matches_with_date(&task("Task @work"), today()));
        assert!(!f.matches_with_date(&task("Task @emailer"), today()));
    }

    #[test]
    fn project_prefix_matches_exact_and_hierarchical() {
        let f = Filter::from_query("+client");
        assert!(f.matches_with_date(&task("Work +client"), today()));
        assert!(f.matches_with_date(&task("Acme +client/acme"), today()));
        assert!(!f.matches_with_date(&task("Other +other"), today()));
    }

    #[test]
    fn context_prefix_case_insensitive() {
        let f = Filter::from_query("@EMAIL");
        assert!(f.matches_with_date(&task("Task @email"), today()));
        assert!(f.matches_with_date(&task("Task @email/waiting"), today()));
    }

    #[test]
    fn negated_context_prefix_excludes_matching() {
        let f = Filter::from_query("-@email");
        assert!(f.matches_with_date(&task("Task @work"), today()));
        assert!(!f.matches_with_date(&task("Task @email"), today()));
        assert!(!f.matches_with_date(&task("Task @email/waiting"), today()));
    }

    #[test]
    fn negated_project_prefix_excludes_matching() {
        let f = Filter::from_query("-+client");
        assert!(f.matches_with_date(&task("Task +other"), today()));
        assert!(!f.matches_with_date(&task("Task +client"), today()));
        assert!(!f.matches_with_date(&task("Task +client/acme"), today()));
    }

    #[test]
    fn exact_slash_delimited_context_matches_only_exact() {
        let f = Filter::from_query("@email/waiting");
        assert!(f.matches_with_date(&task("Task @email/waiting"), today()));
        assert!(!f.matches_with_date(&task("Task @email"), today()));
        assert!(!f.matches_with_date(&task("Task @email/forward"), today()));
    }

    #[test]
    fn exact_slash_delimited_project_matches_only_exact() {
        let f = Filter::from_query("+client/acme");
        assert!(f.matches_with_date(&task("Task +client/acme"), today()));
        assert!(!f.matches_with_date(&task("Task +client"), today()));
        assert!(!f.matches_with_date(&task("Task +client/other"), today()));
    }

    #[test]
    fn prefix_and_exact_slash_can_mix() {
        let f = Filter::from_query("@email +client/acme");
        assert!(f.matches_with_date(&task("Task @email +client/acme"), today()));
        assert!(f.matches_with_date(&task("Task @email/waiting +client/acme"), today()));
        assert!(!f.matches_with_date(&task("Task @email +client"), today()));
        assert!(!f.matches_with_date(&task("Task @work +client/acme"), today()));
    }
}

//! State structures for multi-pane TUI model.

use todotxt_core::{SortOrder, TaskList};
use chrono::NaiveDate;
use chrono::Datelike;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub enum DisplayRow {
    Task(usize),
    GroupHeader(String),
}

/// Mode for autocomplete interactions (Phase 33, Plan 02).
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum AutocompleteMode {
    /// Token autocomplete from editor mode (@/+ in text input)
    TokenAutocomplete(char),
    /// Quick setter from Normal mode (@/+ hotkey)
    QuickSetter(char),
    /// Date autocomplete (special trigger)
    DateAutocomplete,
}

/// Represents a single pane view with independent state.
/// Each pane maintains its own filtered task list, selection, and configuration.
#[derive(Debug, Clone)]
pub struct Pane {
    /// Unique identifier for this pane (0-based index when created)
    #[allow(dead_code)]
    pub id: usize,

    /// Filtered and sorted visible task list for this pane
    pub display_rows: Vec<DisplayRow>,

    /// 0-based index into display_rows for the currently selected row
    pub selected: usize,

    /// Query filter state specific to this pane (PANE-03 prep)
    #[allow(dead_code)]
    pub filter_query: String,

    /// Sort order for this pane (PANE-04 prep)
    #[allow(dead_code)]
    pub sort_order: SortOrder,

    /// Per-pane grouping toggle (tracks whether to render group headers)
    #[allow(dead_code)]
    pub grouping: bool,

    /// Name/label for the pane (e.g., "Work", "Personal")
    #[allow(dead_code)]
    pub label: String,

    /// True when pane header/title is selected for label editing.
    pub label_selected: bool,
}

impl Pane {
    /// Create a new pane with default state
    pub fn new(id: usize, label: String) -> Self {
        Pane {
            id,
            display_rows: Vec::new(),
            selected: 0,
            filter_query: String::new(),
            sort_order: SortOrder::FileOrder,
            grouping: false,
            label,
            label_selected: false,
        }
    }

    /// Check if pane has any visible tasks
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.display_rows.is_empty()
    }

    /// Get current selected row if any
    #[allow(dead_code)]
    pub fn selected_row(&self) -> Option<&DisplayRow> {
        self.display_rows.get(self.selected)
    }

    /// Move selection down, clamping to bounds
    #[allow(dead_code)]
    pub fn select_next(&mut self) {
        if !self.display_rows.is_empty() && self.selected < self.display_rows.len() - 1 {
            self.selected += 1;
        }
    }

    /// Move selection up, clamping to bounds
    #[allow(dead_code)]
    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }
}

/// State for the @context / +project autocomplete popup (D-08, D-09 in 11-CONTEXT.md).
/// Extended for quick-setter mode (Phase 33, Plan 02).
#[derive(Debug, Clone)]
pub struct AutocompleteState {
    #[allow(dead_code)]
    pub mode: AutocompleteMode, // Type of autocomplete interaction
    #[allow(dead_code)]
    pub trigger: char,    // '@', '+', or '#'
    pub prefix: String,   // text typed after the trigger (NOT including trigger)
    #[allow(dead_code)]
    pub all_items: Vec<String>, // original candidate pool (used by quick setters)
    pub items: Vec<String>, // filtered token list (without trigger char)
    pub selected: usize,  // current highlight index in popup
    pub focused: bool,    // true when Down arrow moved focus into popup
}

impl AutocompleteState {
    /// Create autocomplete state for token autocomplete from editor
    pub fn new(trigger: char, prefix: String, items: Vec<String>) -> Self {
        let mode = if trigger == '#' {
            AutocompleteMode::DateAutocomplete
        } else {
            AutocompleteMode::TokenAutocomplete(trigger)
        };
        let all_items = items.clone();
        AutocompleteState { mode, trigger, prefix, all_items, items, selected: 0, focused: false }
    }

    /// Create autocomplete state for quick setter from Normal mode
        #[allow(dead_code)]
    pub fn new_quick_setter(trigger: char, prefix: String, items: Vec<String>) -> Self {
        let all_items = items.clone();
        AutocompleteState {
            mode: AutocompleteMode::QuickSetter(trigger),
            trigger,
            prefix,
            all_items,
            items,
            selected: 0,
            focused: false,
        }
    }
}

/// State for the date picker modal (Phase 33, Plan 01).
/// Tracks month/year selection and day suggestions with weekday labels.
#[derive(Debug, Clone)]
pub struct DatePickerState {
    pub month_year: String,   // e.g., "2026-07"
    pub selected_day: Option<u32>, // currently highlighted day
    pub day_input: String,    // optional typed day input (e.g., "14")
    pub suggestions: Vec<String>, // formatted as "01 Mon", "02 Tue", etc.
    pub focused: bool,        // true when navigation has focused the picker (like autocomplete)
}

impl DatePickerState {
    /// Create a new date picker state for a given month.
    /// Validates the month and generates day suggestions.
    #[allow(dead_code)]
    pub fn new(month_year: &str) -> Self {
        let suggestions = generate_date_suggestions(month_year).unwrap_or_default();
        let selected_day = suggestions.first()
            .and_then(|s| s.split_whitespace().next())
            .and_then(|d| d.parse::<u32>().ok());

        DatePickerState {
            month_year: month_year.to_string(),
            selected_day,
            day_input: String::new(),
            suggestions,
            focused: false,
        }
    }

    /// Navigate to the next day in suggestions (with wrapping).
    pub fn select_next(&mut self) {
        if self.suggestions.is_empty() {
            return;
        }
        let current_idx = self.selected_day
            .and_then(|day| {
                self.suggestions.iter().position(|s| {
                    s.split_whitespace().next()
                        .and_then(|d| d.parse::<u32>().ok())
                        .map(|d| d == day)
                        .unwrap_or(false)
                })
            })
            .unwrap_or(0);
        let next_idx = (current_idx + 1).min(self.suggestions.len().saturating_sub(1));
        self.selected_day = self.suggestions[next_idx]
            .split_whitespace()
            .next()
            .and_then(|d| d.parse::<u32>().ok());
    }

    /// Navigate to the previous day in suggestions (with wrapping).
    pub fn select_prev(&mut self) {
        if self.suggestions.is_empty() {
            return;
        }
        let current_idx = self.selected_day
            .and_then(|day| {
                self.suggestions.iter().position(|s| {
                    s.split_whitespace().next()
                        .and_then(|d| d.parse::<u32>().ok())
                        .map(|d| d == day)
                        .unwrap_or(false)
                })
            })
            .unwrap_or(0);
        let prev_idx = current_idx.saturating_sub(1);
        self.selected_day = self.suggestions[prev_idx]
            .split_whitespace()
            .next()
            .and_then(|d| d.parse::<u32>().ok());
    }
}

/// State for the priority picker modal (Phase 34).
/// Lists A–Z priorities plus a "clear" option.
#[derive(Debug, Clone)]
pub struct PriorityPickerState {
    /// Items: "A", "B", ..., "Z", "— (no priority)"
    pub items: Vec<String>,
    /// Index of the currently highlighted item.
    pub selected_idx: usize,
    /// Last typed letter (for type-to-jump); case-normalized to uppercase.
    pub type_char: Option<char>,
    /// True when navigation has focused the picker (mirrors DatePickerState.focused).
    pub focused: bool,
}

impl Default for PriorityPickerState {
    fn default() -> Self {
        Self::new()
    }
}

impl PriorityPickerState {
    pub fn new() -> Self {
        let mut items: Vec<String> = ('A'..='Z').map(|c| c.to_string()).collect();
        items.push("— (no priority)".to_string());
        PriorityPickerState {
            items,
            selected_idx: 0,
            type_char: None,
            focused: false,
        }
    }

    pub fn select_next(&mut self) {
        if self.selected_idx < self.items.len().saturating_sub(1) {
            self.selected_idx += 1;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
        }
    }

    /// Jump to the item starting with `ch` (A–Z, case-insensitive).
    pub fn jump_to(&mut self, ch: char) {
        let target = ch.to_ascii_uppercase().to_string();
        if let Some(idx) = self.items.iter().position(|item| item == &target) {
            self.selected_idx = idx;
            self.type_char = Some(ch.to_ascii_uppercase());
        }
    }

    /// Returns the chosen priority: `Some(char)` for A–Z, `None` for "no priority" item.
    pub fn selected_priority(&self) -> Option<char> {
        self.items.get(self.selected_idx)
            .and_then(|s| s.chars().next())
            .filter(|c| c.is_ascii_uppercase())
    }
}

/// Generate date suggestions for a given month.
/// Returns a Vec of formatted strings: "01 Mon", "02 Tue", etc.
/// Returns empty Vec for invalid month format or out-of-range months.
#[allow(dead_code)]
pub fn generate_date_suggestions(month_year: &str) -> Result<Vec<String>, String> {
    // Parse "YYYY-MM" format
    let parts: Vec<&str> = month_year.split('-').collect();
    if parts.len() != 2 {
        return Ok(Vec::new());
    }

    let year_str = parts[0];
    let month_str = parts[1];

    let year = year_str.parse::<i32>()
        .map_err(|_| "Invalid year".to_string())?;
    let month = month_str.parse::<u32>()
        .map_err(|_| "Invalid month".to_string())?;

    if !(1..=12).contains(&month) {
        return Ok(Vec::new());
    }

    // Generate all days in the month
    let mut suggestions = Vec::new();
    for day in 1..=31 {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            let weekday_abbr = match date.weekday() {
                chrono::Weekday::Mon => "Mon",
                chrono::Weekday::Tue => "Tue",
                chrono::Weekday::Wed => "Wed",
                chrono::Weekday::Thu => "Thu",
                chrono::Weekday::Fri => "Fri",
                chrono::Weekday::Sat => "Sat",
                chrono::Weekday::Sun => "Sun",
            };
            suggestions.push(format!("{:02} {}", day, weekday_abbr));
        }
    }

    Ok(suggestions)
}

/// Rank token matches for quick setters (Phase 33, Plan 02).
/// Returns matches in order: exact matches, prefix matches, near-matches (substring/fuzzy).
/// Per D-05: Show potentially redundant near-matches to expose variants.
#[allow(dead_code)]
pub fn rank_matches(typed_prefix: &str, candidates: Vec<String>) -> Vec<String> {
    if typed_prefix.is_empty() {
        return candidates;
    }

    let typed_lower = typed_prefix.to_lowercase();
    let mut exact_matches = Vec::new();
    let mut prefix_matches = Vec::new();
    let mut near_matches = Vec::new();

    for candidate in candidates {
        let candidate_lower = candidate.to_lowercase();
        if candidate_lower == typed_lower {
            exact_matches.push(candidate);
        } else if candidate_lower.starts_with(&typed_lower) {
            prefix_matches.push(candidate);
        } else if candidate_lower.contains(&typed_lower)
            || is_fuzzy_subsequence(&typed_lower, &candidate_lower)
        {
            near_matches.push(candidate);
        }
    }

    // Combine: exact first, then prefix, then near-matches
    exact_matches.extend(prefix_matches);
    exact_matches.extend(near_matches);
    exact_matches
}

fn is_fuzzy_subsequence(needle: &str, haystack: &str) -> bool {
    let mut needle_chars = needle.chars();
    let mut current = needle_chars.next();

    for ch in haystack.chars() {
        if Some(ch) == current {
            current = needle_chars.next();
            if current.is_none() {
                return true;
            }
        }
    }

    false
}

/// Extract all @context tokens from task list, deduplicated and sorted.
#[allow(dead_code)]
pub fn get_existing_contexts(task_list: &TaskList) -> HashSet<String> {
    dedupe_tokens(task_list.tasks().iter().flat_map(|t| t.contexts.clone()))
}

/// Extract all +project tokens from task list, deduplicated and sorted.
#[allow(dead_code)]
pub fn get_existing_projects(task_list: &TaskList) -> HashSet<String> {
    dedupe_tokens(task_list.tasks().iter().flat_map(|t| t.projects.clone()))
}

fn dedupe_tokens(tokens: impl Iterator<Item = String>) -> HashSet<String> {
    let mut canonical: HashMap<String, String> = HashMap::new();

    for token in tokens {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            continue;
        }

        let lower = trimmed.to_lowercase();
        canonical
            .entry(lower)
            .or_insert_with(|| trimmed.to_string());
    }

    canonical.into_values().collect()
}

/// State for the filter panel input and preset list (Phase 12, Plan 02).
pub struct FilteringState {
    pub editor: tui_textarea::TextArea<'static>,
    pub selected_preset: usize,
    /// Snapshot of `filter_query` captured when the panel was opened (D-02).
    /// Restored on Esc so no destructive clear occurs.
    pub snapshot: String,
}

/// State for the F-key preset definition panel (D-01, D-06, D-07).
pub struct FilterDefiningState {
    /// Row 0: editable active filter with live preview (D-07).
    pub active_editor: tui_textarea::TextArea<'static>,
    /// Preset names in sorted order (index 0 = preset #1).
    pub preset_names: Vec<String>,
    /// One editor per preset slot; index 0 corresponds to preset_names[0].
    pub preset_editors: Vec<tui_textarea::TextArea<'static>>,
    /// Currently focused row: 0 = active filter row, 1–9 = preset row N.
    pub selected_row: usize,
}

/// Snapshot of task list state captured before a mutating action (Phase 36, UNDO-01/02, D-04/D-05).
/// Stored as `Option<UndoEntry>` on `App`; restored via `apply_undo()`.
#[derive(Debug, Clone)]
pub struct UndoEntry {
    /// Full clone of all tasks at the moment of snapshot.
    pub tasks: Vec<todotxt_core::Task>,
    /// Primary cursor position (`App::selected`) at the moment of snapshot.
    pub selected: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pane_new() {
        let pane = Pane::new(0, "Test".to_string());
        assert_eq!(pane.id, 0);
        assert_eq!(pane.label, "Test");
        assert!(pane.display_rows.is_empty());
        assert_eq!(pane.selected, 0);
        assert_eq!(pane.filter_query, "");
        assert_eq!(pane.grouping, false);
        assert!(!pane.label_selected);
    }

    #[test]
    fn test_pane_is_empty() {
        let mut pane = Pane::new(0, "Test".to_string());
        assert!(pane.is_empty());

        pane.display_rows.push(DisplayRow::Task(0));
        assert!(!pane.is_empty());
    }

    #[test]
    fn test_pane_selected_row() {
        let mut pane = Pane::new(0, "Test".to_string());
        pane.display_rows = vec![
            DisplayRow::Task(0),
            DisplayRow::Task(1),
            DisplayRow::Task(2),
        ];

        assert_eq!(pane.selected_row(), Some(&DisplayRow::Task(0)));

        pane.selected = 2;
        assert_eq!(pane.selected_row(), Some(&DisplayRow::Task(2)));

        pane.selected = 5; // Out of bounds
        assert_eq!(pane.selected_row(), None);
    }

    #[test]
    fn test_pane_select_next() {
        let mut pane = Pane::new(0, "Test".to_string());
        pane.display_rows = vec![
            DisplayRow::Task(0),
            DisplayRow::Task(1),
            DisplayRow::Task(2),
        ];

        assert_eq!(pane.selected, 0);
        pane.select_next();
        assert_eq!(pane.selected, 1);
        pane.select_next();
        assert_eq!(pane.selected, 2);
        pane.select_next(); // Should not go beyond last
        assert_eq!(pane.selected, 2);
    }

    #[test]
    fn test_pane_select_prev() {
        let mut pane = Pane::new(0, "Test".to_string());
        pane.display_rows = vec![
            DisplayRow::Task(0),
            DisplayRow::Task(1),
            DisplayRow::Task(2),
        ];

        pane.selected = 2;
        pane.select_prev();
        assert_eq!(pane.selected, 1);
        pane.select_prev();
        assert_eq!(pane.selected, 0);
        pane.select_prev(); // Should not go below 0
        assert_eq!(pane.selected, 0);
    }

    #[test]
    fn test_pane_selection_empty() {
        let mut pane = Pane::new(0, "Test".to_string());
        // select_next on empty pane should do nothing
        pane.select_next();
        assert_eq!(pane.selected, 0);
        
        // select_prev on empty pane should do nothing
        pane.select_prev();
        assert_eq!(pane.selected, 0);
    }

    #[test]
    fn test_generate_date_suggestions_valid_month() {
        let suggestions = generate_date_suggestions("2026-07").expect("Should generate suggestions");
        assert!(!suggestions.is_empty(), "July 2026 should have days");
        assert_eq!(suggestions.len(), 31, "July should have 31 days");
        
        // Check format of first suggestion (should be "01 <weekday>")
        assert!(suggestions[0].starts_with("01"), "First day should be 01");
        let parts: Vec<&str> = suggestions[0].split_whitespace().collect();
        assert_eq!(parts.len(), 2, "Should have day and weekday");
        assert_eq!(parts[0], "01", "First part should be 01");
        assert!(parts[1].len() == 3, "Weekday should be 3 letters (e.g., 'Wed')");
        
        // Check format of middle suggestion (should be "14 <weekday>")
        assert!(suggestions[13].starts_with("14"), "14th day should start with 14");
        let parts: Vec<&str> = suggestions[13].split_whitespace().collect();
        assert_eq!(parts.len(), 2, "Should have day and weekday");
    }

    #[test]
    fn test_generate_date_suggestions_february_leap_year() {
        let suggestions = generate_date_suggestions("2024-02").expect("Should generate suggestions");
        assert_eq!(suggestions.len(), 29, "February 2024 (leap year) should have 29 days");
    }

    #[test]
    fn test_generate_date_suggestions_february_non_leap_year() {
        let suggestions = generate_date_suggestions("2023-02").expect("Should generate suggestions");
        assert_eq!(suggestions.len(), 28, "February 2023 (non-leap year) should have 28 days");
    }

    #[test]
    fn test_generate_date_suggestions_invalid_month() {
        let suggestions = generate_date_suggestions("2026-13").expect("Should not error");
        assert!(suggestions.is_empty(), "Month 13 should return empty Vec");
    }

    #[test]
    fn test_generate_date_suggestions_invalid_format() {
        let suggestions = generate_date_suggestions("2026/07").expect("Should not error");
        assert!(suggestions.is_empty(), "Invalid format should return empty Vec");
    }

    #[test]
    fn test_date_picker_state_new() {
        let picker = DatePickerState::new("2026-07");
        assert_eq!(picker.month_year, "2026-07");
        assert_eq!(picker.selected_day, Some(1));
        assert!(!picker.suggestions.is_empty());
        assert!(!picker.focused);
    }

    #[test]
    fn test_date_picker_select_next() {
        let mut picker = DatePickerState::new("2026-07");
        picker.select_next();
        assert_eq!(picker.selected_day, Some(2));
        picker.select_next();
        assert_eq!(picker.selected_day, Some(3));
    }

    #[test]
    fn test_date_picker_select_prev() {
        let mut picker = DatePickerState::new("2026-07");
        picker.selected_day = Some(3);
        picker.select_prev();
        assert_eq!(picker.selected_day, Some(2));
        picker.select_prev();
        assert_eq!(picker.selected_day, Some(1));
    }

    #[test]
    fn test_date_picker_select_prev_at_start() {
        let mut picker = DatePickerState::new("2026-07");
        picker.selected_day = Some(1);
        picker.select_prev();
        assert_eq!(picker.selected_day, Some(1), "Should not go before first day");
    }

    #[test]
    fn test_date_picker_select_next_at_end() {
        let mut picker = DatePickerState::new("2026-07");
        picker.selected_day = Some(31);
        picker.select_next();
        assert_eq!(picker.selected_day, Some(31), "Should not go beyond last day");
    }

    #[test]
    fn test_rank_matches_exact() {
        let candidates = vec!["email".to_string(), "work".to_string(), "personal".to_string()];
        let result = rank_matches("email", candidates);
        assert_eq!(result[0], "email", "Exact match should be first");
    }

    #[test]
    fn test_rank_matches_prefix() {
        let candidates = vec!["email".to_string(), "work".to_string(), "waiting".to_string()];
        let result = rank_matches("wa", candidates);
        assert_eq!(result[0], "waiting", "Prefix match should be first");
    }

    #[test]
    fn test_rank_matches_case_insensitive() {
        let candidates = vec!["Email".to_string(), "WORK".to_string()];
        let result = rank_matches("email", candidates);
        assert_eq!(result[0], "Email", "Should match case-insensitively");
    }

    #[test]
    fn test_rank_matches_substring() {
        let candidates = vec!["waiting".to_string(), "email".to_string()];
        let result = rank_matches("ait", candidates);
        assert_eq!(result[0], "waiting", "Substring match should appear");
    }

    #[test]
    fn test_rank_matches_order_exact_prefix_substring() {
        let candidates = vec!["work".to_string(), "works".to_string(), "network".to_string()];
        let result = rank_matches("work", candidates);
        assert_eq!(result[0], "work", "Exact match first");
        assert_eq!(result[1], "works", "Prefix match second");
        assert_eq!(result[2], "network", "Substring match third");
    }

    #[test]
    fn test_rank_matches_empty_prefix() {
        let candidates = vec!["email".to_string(), "work".to_string()];
        let result = rank_matches("", candidates);
        assert_eq!(result.len(), 2, "Empty prefix should return all candidates");
    }

    #[test]
    fn test_autocomplete_state_new_token() {
        let ac = AutocompleteState::new('@', "em".to_string(), vec!["email".to_string()]);
        assert_eq!(ac.mode, AutocompleteMode::TokenAutocomplete('@'));
        assert_eq!(ac.trigger, '@');
        assert_eq!(ac.prefix, "em");
        assert!(!ac.focused);
    }

    #[test]
    fn test_autocomplete_state_new_date() {
        let ac = AutocompleteState::new('#', "2026-07".to_string(), vec!["01 Mon".to_string()]);
        assert_eq!(ac.mode, AutocompleteMode::DateAutocomplete);
        assert_eq!(ac.trigger, '#');
    }

    #[test]
    fn test_autocomplete_state_new_quick_setter() {
        let ac = AutocompleteState::new_quick_setter('@', "".to_string(), vec!["email".to_string()]);
        assert_eq!(ac.mode, AutocompleteMode::QuickSetter('@'));
        assert_eq!(ac.trigger, '@');
        assert_eq!(ac.prefix, "");
    }
}

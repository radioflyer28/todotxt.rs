//! State structures for multi-pane TUI model.

use todotxt_core::SortOrder;

#[derive(Debug, Clone, PartialEq)]
pub enum DisplayRow {
    Task(usize),
    GroupHeader(String),
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

    /// Name/label for the pane (e.g., "Work", "Personal")
    #[allow(dead_code)]
    pub label: String,
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
            label,
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
#[derive(Debug, Clone)]
pub struct AutocompleteState {
    pub trigger: char,    // '@' or '+'
    pub prefix: String,   // text typed after the trigger (NOT including trigger)
    pub items: Vec<String>, // filtered token list (without trigger char)
    pub selected: usize,  // current highlight index in popup
    pub focused: bool,    // true when Down arrow moved focus into popup
}

impl AutocompleteState {
    pub fn new(trigger: char, prefix: String, items: Vec<String>) -> Self {
        AutocompleteState { trigger, prefix, items, selected: 0, focused: false }
    }
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

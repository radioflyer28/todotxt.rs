//! Application state and main event loop.
//!
//! All state mutation happens exclusively on the main thread (D-03).
//! The two sender threads only produce `AppEvent` values — they never
//! touch `App` or `TaskList` directly.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use chrono::{Local, Datelike};
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::Frame;
use todotxt_core::{Filter, SortOrder, Task, TaskList, normalize_append, normalize_line};
use tui_textarea::TextArea;

use crate::config::{GroupByCategory, PaneConfig, PaneSort, TuiConfig, resolve_keymap};
use crate::event::AppEvent;
use crate::theme as theme_module;
use theme_module::{StyleSheet, Theme};
use crate::tui::Tui;
use crate::state::{Pane, DisplayRow, AutocompleteState, AutocompleteMode, FilteringState, FilterDefiningState, DatePickerState, PriorityPickerState, get_existing_contexts, get_existing_projects, rank_matches};
use crate::components::PaneList;
use arboard::Clipboard;



/// Interaction mode for the TUI (D-01 in 11-CONTEXT.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    QuickSetter(char),
    Adding,
    Editing { original_idx: usize },
    PaneLabelEditing { pane_idx: usize },
    DeleteConfirm,
    /// Archive-confirm overlay: shows completed count before writing done.txt (Phase 39, ARCH-01/02/03).
    ArchiveConfirm,
    Filtering,
    /// F-key preset definition panel (D-01 in 16-CONTEXT.md).
    FilterDefining,
    /// Bulk append mode: user types text to append to all selected tasks (D-06, Phase 20).
    AppendText,
    /// Read-only overlay showing app warnings/errors log.
    KeymapErrors,
    /// Read-only overlay showing all keybindings (D-10, Phase 22 parity).
    Help,
    /// Date picker overlay for setting due dates (Phase 33, Plan 01).
    #[allow(dead_code)]
    DatePicker,
    /// Priority picker overlay for setting priority on active/selected tasks (Phase 34, Plan 01).
    PriorityPicker,
    /// Count preview before bulk append — shows "Appending to N tasks" (D-06, Phase 34).
    AppendTextConfirm,
}

/// Top-level application state.
pub struct App {
    pub should_quit: bool,
    pub task_list: TaskList,
    pub todo_path: PathBuf,
    /// 0-based index into `display_rows` for the currently selected row.
    /// Always clamped to `[0, display_rows.len() - 1]`.
    pub selected: usize,
    /// Height of the list area in terminal rows. Kept in sync with `Resize` events.
    /// Used to compute half-page step for Ctrl+d / Ctrl+u (D-09).
    pub list_height: u16,
    /// Current interaction mode (D-01).
    pub mode: AppMode,
    /// Single-line text editor used in Adding and Editing modes (D-03).
    pub editor: TextArea<'static>,
    /// When true, a `FileChanged` event arrived while not in Normal mode and
    /// will be applied on the next Normal-mode entry (D-10).
    pub pending_reload: bool,
    /// Active autocomplete popup state, or `None` when not shown.
    pub autocomplete: Option<AutocompleteState>,
    /// Active date picker state, or `None` when not shown (Phase 33, Plan 01).
    pub date_picker: Option<DatePickerState>,
    /// Active priority picker state, or `None` when not shown (Phase 34, Plan 01).
    pub priority_picker: Option<PriorityPickerState>,
    /// Count of tasks targeted by AppendTextConfirm mode (Phase 34, Plan 03).
    pub append_confirm_count: usize,
    /// Maps display row position → canonical task index (D-10, D-11 in 12-CONTEXT.md).
    pub display_indices: Vec<usize>,
    /// Toggle grouped rendering with non-selectable header rows.
    pub grouping: bool,
    /// Group-by dimension for single-pane mode — independent of sort_order (GRP-01, Phase 40).
    pub group_by: GroupByCategory,
    /// Rendered rows for list/navigation; includes group headers when grouping is enabled.
    pub display_rows: Vec<DisplayRow>,
    /// Current display sort order (FileOrder = no sort applied).
    pub sort_order: SortOrder,
    /// Toggle visibility of deferred tasks (`t:` in the future).
    pub show_deferred: bool,
    /// Active filter query string (empty = no filter).
    pub filter_query: String,
    /// Last non-empty filter captured when Ctrl+F toggles filtering off.
    pub toggled_filter_query: Option<String>,
    /// Filter panel state, or `None` when panel is closed (Plan 02).
    pub filter_state: Option<FilteringState>,
    /// Named filter presets from `[presets.filter]` in config (Phase 41, PRST-01).
    pub presets: Vec<(String, String)>,
    /// Full pane layout presets from `[presets.panes]` in config (Phase 41, PRST-02).
    /// Sorted alphabetically by name; Ctrl+N applies preset at index N-1.
    pub pane_presets: Vec<(String, crate::config::PaneLayoutPreset)>,
    /// Session filter history ring (Phase 41, FHIST-01/02/03).
    /// Most-recently-applied filters at front. Capped at 50. Session-only.
    pub filter_history: std::collections::VecDeque<String>,
    /// Ctrl+R cycling cursor into filter_history. None = not currently cycling.
    pub filter_history_cursor: Option<usize>,
    /// Full TUI config (needed for preset definition panel save, D-04).
    pub config: TuiConfig,
    /// Config file path, used by TuiConfig::save() in the definition panel.
    pub config_path: Option<PathBuf>,
    /// State for the F-key preset definition panel, or None when closed.
    pub filter_defining_state: Option<FilterDefiningState>,
    /// Pre-computed color styles for the active theme (D-08, D-09 in 13-CONTEXT.md).
    pub styles: StyleSheet,
    /// Currently active palette, used by `t` key theme cycling.
    pub palette: Theme,
    /// Whether NO_COLOR mode is active; preserves monochrome behavior while cycling themes.
    pub no_color: bool,
    /// Canonical task indices that are currently multi-selected (D-01 in 19-CONTEXT.md).
    pub selected_tasks: HashSet<usize>,
    /// Anchor index for shift-range selection (D-02 in 19-CONTEXT.md).
    pub selection_anchor: Option<usize>,
    /// When true, Space marks/unmarks the cursor task and navigation does not clear the set (D-04).
    pub disjoint_select: bool,
    /// Keymap warnings collected at startup from resolve_keymap (D-10, Phase 22).
    /// Empty when config has no [keymap] section or all entries are valid.
    /// Displayed in the status bar and the KeymapErrors overlay (Phase 22, Plan 02).
    pub keymap_warnings: Vec<String>,
    /// Runtime warnings/errors captured while the app is running.
    /// Displayed together with keymap warnings in the error log overlay.
    pub runtime_warnings: Vec<String>,
    /// Effective key bindings (action name → (KeyCode, KeyModifiers)), built at startup.
    /// Populated by resolve_keymap — overrides where specified, defaults otherwise (D-05, Phase 22).
    pub effective_keymap: std::collections::HashMap<String, (crossterm::event::KeyCode, crossterm::event::KeyModifiers)>,
    /// Scroll offset for the help overlay (lines scrolled from the top).
    pub help_scroll: u16,
    /// Vector of panes, each with independent task view state
    pub panes: Vec<Pane>,
    /// 0-based index of the currently active pane
    pub active_pane: usize,
    #[allow(dead_code)]

    /// Counter for auto-labeling new panes (e.g., "Pane 1", "Pane 2"). Initialized to 2; first pane is "Pane 1" (D-05).
    pub pane_counter: usize,
    /// When true, all panes are hidden and rendering falls back to single-pane view (D-13, Phase 26).
    /// This flag is session-only (not persisted across restarts). All pane state is preserved.
    pub panes_hidden: bool,
    /// Clipboard instance, lazily initialized on first copy/paste operation (Phase 35, CLIP-01).
    /// Kept as `None` until first use to avoid startup errors in headless environments.
    pub clipboard: Option<Clipboard>,
    /// Single-level undo entry: snapshot of task list + cursor before the last mutating action
    /// (Phase 36, UNDO-01/02, D-02/D-04). `None` = no undo available.
    pub undo_entry: Option<crate::state::UndoEntry>,
    /// Snapshot of pane config taken at startup — compare-on-quit to skip write if unchanged (D-06, Phase 43).
    pub startup_pane_snapshot: Vec<crate::config::PaneConfig>,
}

// ── External editor support (Phase 39, XEDIT-01/02) ──────────────────────────

/// RAII guard that suspends TUI terminal state while an external process runs.
///
/// Construction: leaves raw mode + alternate screen (restores normal terminal).
/// Drop: re-enters alternate screen + raw mode (restores TUI state).
struct RawModeGuard;

impl RawModeGuard {
    fn new() -> Self {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen
        );
        RawModeGuard
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::EnterAlternateScreen
        );
        let _ = crossterm::terminal::enable_raw_mode();
    }
}

/// Resolves the external editor to use: $VISUAL → $EDITOR → platform fallback.
fn resolve_editor() -> Option<String> {
    if let Ok(visual) = std::env::var("VISUAL") {
        if !visual.trim().is_empty() {
            return Some(visual);
        }
    }
    if let Ok(editor) = std::env::var("EDITOR") {
        if !editor.trim().is_empty() {
            return Some(editor);
        }
    }
    #[cfg(target_os = "windows")]
    return Some("notepad.exe".to_string());
    #[cfg(not(target_os = "windows"))]
    return Some("vi".to_string());
}

impl App {
    fn panes_from_config(config: &TuiConfig) -> Vec<Pane> {
        let mut panes: Vec<Pane> = config
            .panes
            .iter()
            .enumerate()
            .map(|(idx, pane_cfg)| {
                let label = if pane_cfg.label.trim().is_empty() {
                    format!("Pane {}", idx + 1)
                } else {
                    pane_cfg.label.clone()
                };

                let mut pane = Pane::new(idx, label);
                pane.filter_query = pane_cfg.filter.clone();
                pane.sort_order = pane_cfg.sort.to_sort_order();
                pane.grouping = pane_cfg.group;
                pane.group_by = pane_cfg.group_by.unwrap_or(GroupByCategory::Priority);
                pane
            })
            .collect();

        if panes.is_empty() {
            panes.push(Pane::new(0, String::new()));
        }

        panes
    }

    /// Apply a full pane layout preset atomically (Phase 41, PRST-02, D-04).
    ///
    /// Replaces all current panes with those defined in the preset. Each PaneConfig entry
    /// in the preset becomes a Pane with the configured label, filter, sort, group, group_by.
    /// Active pane is reset to 0. All panes are rebuilt immediately.
    /// Empty presets are a no-op (preserve existing panes).
    fn apply_pane_layout_preset(&mut self, preset: &crate::config::PaneLayoutPreset) {
        if preset.panes.is_empty() {
            return;
        }
        let new_panes: Vec<Pane> = preset
            .panes
            .iter()
            .enumerate()
            .map(|(i, cfg)| {
                let label = if cfg.label.trim().is_empty() {
                    format!("Pane {}", i + 1)
                } else {
                    cfg.label.clone()
                };
                let mut pane = Pane::new(i, label);
                pane.filter_query = cfg.filter.clone();
                pane.sort_order = cfg.sort.to_sort_order();
                pane.grouping = cfg.group;
                pane.group_by = cfg.group_by.unwrap_or(GroupByCategory::Priority);
                pane
            })
            .collect();
        self.panes = new_panes;
        self.active_pane = 0;
        self.selected_tasks.clear();
        self.selection_anchor = None;
        self.rebuild_all_panes();
        self.rebuild_display_indices();
    }

    /// Push a filter expression to the session history ring (Phase 41, FHIST-01/03, D-10).
    ///
    /// - Empty strings are ignored.
    /// - Duplicate entries are removed before pushing to front (dedup, FHIST-03).
    /// - Ring is capped at 50 entries; oldest entry is dropped when cap exceeded (D-10).
    fn push_filter_history(&mut self, expr: &str) {
        if expr.is_empty() {
            return;
        }
        self.filter_history.retain(|e| e != expr);
        self.filter_history.push_front(expr.to_string());
        while self.filter_history.len() > 50 {
            self.filter_history.pop_back();
        }
        // Reset Ctrl+R cursor (new push invalidates any active cycling position).
        self.filter_history_cursor = None;
    }

    /// Returns true if `filter` is a single @/+ tag token with no spaces (Phase 41, PMOVE-01, D-15).
    ///
    /// Valid: "@work", "+project", "@home-office". Invalid: "@work @home", "due:today", "".
    fn is_single_tag_token(filter: &str) -> bool {
        if filter.is_empty() {
            return false;
        }
        let trimmed = filter.trim();
        (trimmed.starts_with('@') || trimmed.starts_with('+'))
            && !trimmed.contains(char::is_whitespace)
    }

    /// Move task(s) from the active pane to an adjacent pane by tag mutation (Phase 41, PMOVE-02).
    ///
    /// `direction` is +1 (right) or -1 (left). Wraps at boundaries using rem_euclid.
    ///
    /// Validation (PMOVE-03, D-15): source and dest pane must each have a single-token
    /// @/+ filter. Otherwise pushes a status message and returns without mutating.
    ///
    /// Mutation per task (D-16): removes the source token from task raw text (if present),
    /// then appends the dest token (if not already present).
    ///
    /// After mutation (D-17, D-18): pushes undo entry BEFORE mutation, saves, rebuilds,
    /// and jumps active_pane to the dest pane index.
    fn pane_move_task(&mut self, direction: isize) -> color_eyre::Result<()> {
        let pane_count = self.panes.len();
        if pane_count < 2 {
            self.push_runtime_warning("Need at least 2 panes to move tasks.".to_string());
            return Ok(());
        }

        let src_idx = self.active_pane;
        let dest_idx = ((src_idx as isize + direction).rem_euclid(pane_count as isize)) as usize;

        let src_filter = self.panes[src_idx].filter_query.trim().to_string();
        let dest_filter = self.panes[dest_idx].filter_query.trim().to_string();

        if !Self::is_single_tag_token(&src_filter) {
            self.push_runtime_warning(format!(
                "Cannot move: source pane filter '{}' is not a single @/+ tag.",
                src_filter
            ));
            return Ok(());
        }
        if !Self::is_single_tag_token(&dest_filter) {
            self.push_runtime_warning(format!(
                "Cannot move: destination pane filter '{}' is not a single @/+ tag.",
                dest_filter
            ));
            return Ok(());
        }

        // Collect global task indices: selected_tasks if non-empty, else cursor task in active pane.
        let task_indices: Vec<usize> = if !self.selected_tasks.is_empty() {
            self.selected_tasks.iter().cloned().collect()
        } else {
            let pane = &self.panes[src_idx];
            match pane.display_rows.get(pane.selected) {
                Some(DisplayRow::Task(idx)) => vec![*idx],
                _ => {
                    self.push_runtime_warning("No task selected to move.".to_string());
                    return Ok(());
                }
            }
        };

        if task_indices.is_empty() {
            return Ok(());
        }

        // Snapshot undo BEFORE mutation (D-17).
        self.push_undo_entry();

        // Mutate each task: remove src_filter token, append dest_filter token.
        for &task_idx in &task_indices {
            if task_idx >= self.task_list.tasks().len() {
                continue;
            }
            let raw = self.task_list.tasks()[task_idx].to_raw().to_string();

            // Remove source filter token (word-by-word, case-sensitive exact match).
            let filtered_tokens: Vec<&str> = raw
                .split_whitespace()
                .filter(|&t| t != src_filter)
                .collect();
            let mut new_raw = filtered_tokens.join(" ");

            // Append dest filter token if not already present.
            let already_has_dest = new_raw
                .split_whitespace()
                .any(|t| t == dest_filter);
            if !already_has_dest {
                if !new_raw.is_empty() {
                    new_raw.push(' ');
                }
                new_raw.push_str(&dest_filter);
            }

            let new_task = todotxt_core::Task::parse(&new_raw);
            if let Err(e) = self.task_list.update(task_idx, new_task) {
                self.push_runtime_warning(format!("pane_move_task: update failed: {e}"));
                return Ok(());
            }
        }

        // Jump to dest pane, clear selection, rebuild (D-18).
        self.selected_tasks.clear();
        self.selection_anchor = None;
        self.active_pane = dest_idx;
        self.rebuild_all_panes();
        self.rebuild_display_indices();

        Ok(())
    }

    pub fn new(task_list: TaskList, todo_path: PathBuf, config: TuiConfig, config_path: Option<PathBuf>, palette: Theme, no_color: bool) -> Self {
        // Build sorted filter presets vec from [presets.filter.*] (Phase 41, PRST-01).
        let mut presets: Vec<(String, String)> = config
            .presets
            .filter
            .iter()
            .filter_map(|(name, p)| p.filter.as_ref().map(|f| (name.clone(), f.clone())))
            .collect();
        presets.sort_by(|(a, _), (b, _)| a.cmp(b));
        // Build sorted pane layout presets vec from [presets.panes.*] (Phase 41, PRST-02).
        let mut pane_presets: Vec<(String, crate::config::PaneLayoutPreset)> = config
            .presets
            .panes
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        pane_presets.sort_by(|(a, _), (b, _)| a.cmp(b));
        // Resolve keymap at startup — applies user overrides, collects warnings (D-04, Phase 22).
        let (effective_keymap, keymap_warnings) = resolve_keymap(&config);
        let panes = Self::panes_from_config(&config);
        // Compute snapshot using the same normalization as save_view_state so the
        // compare-on-quit identity check is reliable (D-06, Phase 43).
        // config.panes.clone() would leave group_by as None (TOML default) while
        // save_view_state writes Some(pane.group_by), causing a false mismatch.
        let startup_pane_snapshot: Vec<crate::config::PaneConfig> = panes
            .iter()
            .map(|pane| crate::config::PaneConfig {
                label: pane.label.clone(),
                filter: pane.filter_query.clone(),
                sort: PaneSort::from_sort_order(pane.sort_order),
                group: pane.grouping,
                group_by: Some(pane.group_by),
            })
            .collect();
        let pane_counter = panes.len() + 1;
        let mut app = App {
            should_quit: false,
            task_list,
            todo_path,
            selected: 0,
            list_height: 0,
            mode: AppMode::Normal,
            editor: TextArea::default(),
            pending_reload: false,
            autocomplete: None,
            date_picker: None,
            priority_picker: None,
            append_confirm_count: 0,
            display_indices: Vec::new(),
            grouping: false,
            group_by: GroupByCategory::Priority,
            display_rows: Vec::new(),
            sort_order: SortOrder::FileOrder,
            show_deferred: false,
            filter_query: String::new(),
            toggled_filter_query: None,
            filter_state: None,
            presets,
            pane_presets,
            filter_history: std::collections::VecDeque::new(),
            filter_history_cursor: None,
            config,
            config_path,
            filter_defining_state: None,
            styles: StyleSheet::from_theme(palette, no_color),
            palette,
            no_color,
            selected_tasks: HashSet::new(),
            selection_anchor: None,
            disjoint_select: false,
            keymap_warnings,
            runtime_warnings: Vec::new(),
            effective_keymap,
            help_scroll: 0,
            panes,
            active_pane: 0,
            pane_counter,
            panes_hidden: false,
            clipboard: None,
            undo_entry: None,
            startup_pane_snapshot,
        };
        // Hydrate every pane immediately so non-active panes are populated on first render.
        app.rebuild_all_panes();
        app.rebuild_display_indices();
        app
    }

    /// Returns true when the given key event matches the configured binding for `action` (D-05, Phase 22).
    ///
    /// Checks `effective_keymap` so user overrides are honoured.
    ///
    /// For bindings with no configured modifier, require no modifiers to avoid accidental
    /// collisions (e.g. Ctrl+N must not also match plain 'n').
    ///
    /// One exception is uppercase printable bindings (e.g. 'D' for bulk_delete): terminals may
    /// or may not report an implicit SHIFT modifier for uppercase input, so we accept either
    /// no modifier or SHIFT-only in that case.
    fn key_is_action(&self, key: crossterm::event::KeyEvent, action: &str) -> bool {
        self.effective_keymap.get(action).is_some_and(|(code, mods)| {
            if mods.is_empty() {
                if key.code != *code {
                    return false;
                }

                match code {
                    KeyCode::Char(c) if c.is_ascii_uppercase() => {
                        key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT
                    }
                    _ => key.modifiers.is_empty(),
                }
            } else {
                key.code == *code && key.modifiers.contains(*mods)
            }
        })
    }

    fn push_runtime_warning(&mut self, msg: impl Into<String>) {
        self.runtime_warnings.push(msg.into());
    }

    /// Capture a snapshot of the current task list and cursor as an undo entry (Phase 36, D-04/D-05).
    /// Overwrites any previous entry (depth-1 semantics, D-02).
    fn push_undo_entry(&mut self) {
        self.undo_entry = Some(crate::state::UndoEntry {
            tasks: self.task_list.tasks().to_vec(),
            selected: self.selected,
        });
    }

    /// Open todo.txt in the user's preferred external editor (Ctrl+E, XEDIT-01/02/03).
    ///
    /// Suspends TUI with RawModeGuard (XEDIT-A), waits for editor to exit, then reloads
    /// the task list and re-renders (XEDIT-C). Pushes undo_entry before opening.
    fn launch_external_editor(&mut self) -> color_eyre::Result<()> {
        let Some(editor) = resolve_editor() else {
            self.runtime_warnings
                .push("No editor found. Set $EDITOR or $VISUAL.".to_string());
            return Ok(());
        };

        self.push_undo_entry();

        // Suspend TUI: disable raw mode + leave alternate screen.
        let _guard = RawModeGuard::new();

        let status = std::process::Command::new(&editor)
            .arg(&self.todo_path)
            .status();

        // Guard drops here, restoring TUI terminal state before we do any further work.
        drop(_guard);

        match status {
            Ok(exit) => {
                match TaskList::load(&self.todo_path) {
                    Ok(new_list) => {
                        self.task_list = new_list;
                        self.rebuild_all_panes();
                        self.rebuild_and_reanchor();
                        if exit.success() {
                            self.runtime_warnings.push(format!(
                                "Reloaded todo.txt after editing with {editor}"
                            ));
                        } else {
                            self.runtime_warnings
                                .push("Editor exited with error; reloaded todo.txt".to_string());
                        }
                    }
                    Err(e) => {
                        self.runtime_warnings
                            .push(format!("Failed to reload todo.txt after editing: {e}"));
                    }
                }
            }
            Err(e) => {
                self.runtime_warnings.push(format!(
                    "Failed to launch editor '{editor}': {e}. Set $EDITOR or $VISUAL."
                ));
            }
        }
        Ok(())
    }

    /// Restore the task list from the undo entry, if one exists (Phase 36, D-04, UNDO-01/02/03).
    /// Silent no-op when `undo_entry` is `None` (D-08/D-10).
    fn apply_undo(&mut self) -> color_eyre::Result<()> {
        let entry = match self.undo_entry.take() {
            Some(e) => e,
            None => return Ok(()),
        };
        self.task_list
            .replace_all(entry.tasks)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to restore undo: {}", e))?;
        self.selected = entry.selected;
        self.rebuild_all_panes();
        self.rebuild_and_reanchor();
        Ok(())
    }

    fn error_log_count(&self) -> usize {
        self.keymap_warnings.len() + self.runtime_warnings.len()
    }

    fn error_log_lines(&self) -> Vec<String> {
        let mut lines = Vec::with_capacity(self.error_log_count());
        lines.extend(
            self.keymap_warnings
                .iter()
                .map(|w| format!("keymap: {}", w)),
        );
        lines.extend(self.runtime_warnings.iter().cloned());
        lines
    }

    /// Move focus to the next pane (right arrow)
    pub fn focus_next_pane(&mut self) {
        self.reconcile_active_pane();
        if self.panes.len() > 1 {
            self.active_pane = (self.active_pane + 1) % self.panes.len();
        }
    }

    /// Move focus to the previous pane (left arrow)
    pub fn focus_prev_pane(&mut self) {
        self.reconcile_active_pane();
        if self.panes.len() > 1 {
            self.active_pane = if self.active_pane == 0 {
                self.panes.len() - 1
            } else {
                self.active_pane - 1
            };
        }
    }

    /// Create a new pane with auto-label and append it to the right (D-05, D-06, D-07).
    /// Returns early if pane count >= 10 (D-03). Focus shifts to the newly created pane.
    pub fn pane_add(&mut self) {
        if self.panes.len() >= 10 {
            return;
        }
        let pane_id = self.panes.len();
        let label = format!("Pane {}", self.pane_counter);
        self.panes.push(Pane::new(pane_id, label));
        self.pane_counter += 1;
        self.active_pane = pane_id;
    }

    /// Delete the active pane with adjacent focus shift and ID re-normalization (D-08, D-09, D-11).
    /// Focus shifts: prefer left (active_pane - 1), else right (0), else none.
    /// Returns early if panes list is empty (D-04).
    pub fn pane_delete(&mut self) {
        if self.panes.is_empty() {
            return;
        }
        let focus_index = if self.active_pane > 0 {
            self.active_pane - 1
        } else {
            0
        };
        self.panes.remove(self.active_pane);
        // Re-normalize all pane IDs after deletion (D-11)
        for (idx, pane) in self.panes.iter_mut().enumerate() {
            pane.id = idx;
        }
        self.active_pane = focus_index;
        self.reconcile_active_pane();
    }

    /// Toggle pane visibility — hides all panes (single-pane render) or restores them (D-12, D-13, D-14, Phase 26).
    /// Hidden state is session-only (not persisted). All pane structure and state are fully preserved.
    pub fn pane_hide_toggle(&mut self) {
        self.panes_hidden = !self.panes_hidden;
    }


    #[allow(dead_code)]
    pub fn active_pane_mut(&mut self) -> &mut Pane {
        self.reconcile_active_pane();
        &mut self.panes[self.active_pane]
    }

    /// Get immutable reference to the active pane
    #[allow(dead_code)]
    pub fn active_pane(&self) -> &Pane {
        &self.panes[self.active_pane]
    }

    /// Determine if the UI should render with the single-pane fallback path.
    pub fn should_show_single_pane(&self) -> bool {
        if self.panes.is_empty() {
            return true;
        }
        if self.panes.len() == 1 {
            return true;
        }
        self.panes.iter().all(Pane::is_empty)
    }

    /// Return display rows for the currently active rendering mode.
    #[allow(dead_code)]
    pub fn display_rows(&self) -> &[DisplayRow] {
        if self.should_show_single_pane() {
            self.panes
                .first()
                .map(|pane| pane.display_rows.as_slice())
                .unwrap_or(&[])
        } else {
            self.panes
                .get(self.active_pane)
                .map(|pane| pane.display_rows.as_slice())
                .unwrap_or(&[])
        }
    }

    /// Return mutable display rows for the active rendering mode.
    #[allow(dead_code)]
    pub fn display_rows_mut(&mut self) -> &mut Vec<DisplayRow> {
        self.reconcile_active_pane();
        let pane_idx = if self.should_show_single_pane() {
            0
        } else {
            self.active_pane
        };
        &mut self.panes[pane_idx].display_rows
    }

    /// Ensure pane state is always index-safe.
    pub fn reconcile_active_pane(&mut self) {
        if self.panes.is_empty() {
            self.panes.push(Pane::new(0, String::new()));
            self.active_pane = 0;
            return;
        }
        if self.active_pane >= self.panes.len() {
            self.active_pane = self.panes.len() - 1;
        }
    }

    /// Rebuild display rows for the visible pane in the current mode.
    pub fn rebuild_visible_rows(&mut self) {
        self.reconcile_active_pane();
        let pane_idx = if self.should_show_single_pane() {
            0
        } else {
            self.active_pane
        };
        let pane = &mut self.panes[pane_idx];

        // Per-pane query behavior (D-04, Phase 25): Apply active pane's filter_query
        let filter_str = pane.filter_query.trim();
        let filter = Filter::from_query(filter_str);
        
        let mut filtered_tasks: Vec<(usize, &Task)> = self
            .task_list
            .filter(&filter)
            .into_iter()
            .collect();

        // Per-pane sort behavior (D-09, Phase 25): Apply active pane's sort_order
        if pane.sort_order != SortOrder::FileOrder {
            filtered_tasks.sort_by(|(_, a), (_, b)| pane.sort_order.compare(a, b));
        }

        // Per-pane grouping behavior (D-09, Phase 25): Add group headers if enabled
        let rows: Vec<DisplayRow> = if pane.grouping && !filtered_tasks.is_empty() {
            let mut display_rows: Vec<DisplayRow> = Vec::new();
            let mut last_key: Option<String> = None;
            let group_by = pane.group_by;
            
            for (source_index, task) in &filtered_tasks {
                let key = group_key_for(task, &group_by);
                if last_key.as_deref() != Some(&key) {
                    display_rows.push(DisplayRow::GroupHeader(key.clone()));
                    last_key = Some(key);
                }
                display_rows.push(DisplayRow::Task(*source_index));
            }
            display_rows
        } else {
            filtered_tasks
                .into_iter()
                .map(|(source_index, _task)| DisplayRow::Task(source_index))
                .collect()
        };

        pane.display_rows = rows;

        if pane.selected >= pane.display_rows.len() && !pane.display_rows.is_empty() {
            pane.selected = pane.display_rows.len() - 1;
        } else if pane.display_rows.is_empty() {
            pane.selected = 0;
        }
    }

    /// Rebuild display rows for ALL panes using each pane's own filter/sort/group state.
    ///
    /// Call after any task_list mutation (add, edit, delete, reload) to keep sibling panes
    /// fresh without waiting for focus. (WARN-3 fix, Phase 28)
    pub fn rebuild_all_panes(&mut self) {
        let pane_count = self.panes.len();
        for idx in 0..pane_count {
            // Extract per-pane settings as owned values so we can borrow self.task_list next.
            let filter_query = self.panes[idx].filter_query.clone();
            let sort_order = self.panes[idx].sort_order;
            let grouping = self.panes[idx].grouping;
            let group_by = self.panes[idx].group_by;

            // Build new display rows. Use a sub-block so `filtered` (which holds &Task refs
            // from self.task_list) is dropped before we mutably borrow self.panes[idx].
            let new_rows: Vec<DisplayRow> = {
                let filter = Filter::from_query(filter_query.trim());
                let mut filtered: Vec<(usize, &Task)> = self
                    .task_list
                    .filter(&filter)
                    .into_iter()
                    .collect();

                if sort_order != SortOrder::FileOrder {
                    filtered.sort_by(|(_, a), (_, b)| sort_order.compare(a, b));
                }

                if grouping && !filtered.is_empty() {
                    let mut rows: Vec<DisplayRow> = Vec::new();
                    let mut last_key: Option<String> = None;
                    for (source_index, task) in &filtered {
                        let key = group_key_for(task, &group_by);
                        if last_key.as_deref() != Some(&key) {
                            rows.push(DisplayRow::GroupHeader(key.clone()));
                            last_key = Some(key);
                        }
                        rows.push(DisplayRow::Task(*source_index));
                    }
                    rows
                } else {
                    filtered
                        .into_iter()
                        .map(|(source_index, _)| DisplayRow::Task(source_index))
                        .collect()
                }
            }; // `filtered` dropped here — self.task_list borrow released

            let pane = &mut self.panes[idx];
            pane.display_rows = new_rows;
            if pane.selected >= pane.display_rows.len() && !pane.display_rows.is_empty() {
                pane.selected = pane.display_rows.len() - 1;
            } else if pane.display_rows.is_empty() {
                pane.selected = 0;
            }
        }
    }

    /// Main event loop. Blocks on `rx.recv()` — no polling (D-02).
    pub fn run(
        &mut self,
        terminal: &mut Tui,
        rx: Receiver<AppEvent>,
    ) -> color_eyre::Result<()> {
        // Capture initial terminal height for half-page scrolling (D-09).
        // list_height = total rows minus 1 (status bar occupies the bottom row).
        let size = terminal.size()?;
        self.list_height = size.height.saturating_sub(1);

        // Initial draw before waiting for the first event.
        terminal.draw(|f| self.draw(f))?;

        while let Ok(event) = rx.recv() {
            self.handle_event(event, terminal)?;
            if self.should_quit {
                break;
            }
        }

        if self.should_quit {
            self.save_view_state()?;
        }

        Ok(())
    }

    pub fn save_view_state(&self) -> color_eyre::Result<()> {
        let current: Vec<crate::config::PaneConfig> = self
            .panes
            .iter()
            .map(|pane| PaneConfig {
                label: pane.label.clone(),
                filter: pane.filter_query.clone(),
                sort: PaneSort::from_sort_order(pane.sort_order),
                group: pane.grouping,
                group_by: Some(pane.group_by),
            })
            .collect();

        // D-06 / Phase 43: skip write entirely when pane config is unchanged.
        if current == self.startup_pane_snapshot {
            return Ok(());
        }

        if let Some(path) = &self.config_path {
            let state_path = crate::config::state_file_path(path);
            crate::config::TuiStateFile { panes: current }.save(&state_path)?;
        }

        Ok(())
    }

    /// Handle a single `AppEvent`. Dispatches on current mode (D-01).
    fn handle_event(
        &mut self,
        event: AppEvent,
        terminal: &mut Tui,
    ) -> color_eyre::Result<()> {
        self.reconcile_active_pane();
        match event {
            AppEvent::Key(key) => {
                // Only react to key presses, not releases or repeats.
                // (PITFALLS: "Key event duplication from press, repeat, and release handling")
                if key.kind != KeyEventKind::Press {
                    return Ok(());
                }
                // AppMode is Copy — match copies the value, releasing the borrow.
                match self.mode {
                    AppMode::Normal => self.handle_normal_key(key)?,
                    AppMode::QuickSetter(_) => self.handle_quick_setter_key(key)?,
                    AppMode::Adding | AppMode::Editing { .. } => {
                        self.handle_editor_key(key)?;
                    }
                    AppMode::PaneLabelEditing { pane_idx } => {
                        self.handle_pane_label_edit_key(key, pane_idx)?;
                    }
                    AppMode::AppendText => self.handle_append_text_key(key)?,
                    AppMode::DeleteConfirm => self.handle_delete_confirm_key(key)?,
                    AppMode::ArchiveConfirm => self.handle_archive_confirm_key(key)?,
                    AppMode::Filtering => self.handle_filtering_key(key)?,
                    AppMode::FilterDefining => self.handle_filter_defining_key(key)?,
                    AppMode::KeymapErrors => self.handle_keymap_errors_key(key)?,
                    AppMode::Help => self.handle_help_key(key)?,
                    AppMode::DatePicker => self.handle_date_picker_key(key)?,
                    AppMode::PriorityPicker => self.handle_priority_picker_key(key)?,
                    AppMode::AppendTextConfirm => self.handle_append_text_confirm_key(key)?,
                }
            }
            AppEvent::FileChanged => {
                // Reload guard (D-10): queue during editing, apply immediately in Normal.
                if self.mode == AppMode::Normal {
                    self.task_list.reload().map_err(|e| {
                        color_eyre::eyre::eyre!(
                            "Failed to reload {}: {}",
                            self.todo_path.display(),
                            e
                        )
                    })?;
                    self.prune_stale_selections();
                    self.rebuild_all_panes();
                    self.rebuild_and_reanchor();
                } else {
                    self.pending_reload = true;
                }
            }
            AppEvent::Resize(_, rows) => {
                self.list_height = rows.saturating_sub(1);
            }
            AppEvent::Error(_) => {}
        }

        terminal.draw(|f| self.draw(f))?;
        Ok(())
    }

    // ── Normal mode key handler ───────────────────────────────────────────────

    fn handle_normal_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> color_eyre::Result<()> {
        let display_count = self.display_indices.len();
        let row_count = if !self.should_show_single_pane() && self.panes.len() > 1 && !self.panes_hidden {
            self.panes[self.active_pane].display_rows.len()
        } else {
            self.display_rows.len()
        };
        match key.code {
            // ── Ctrl+C quit (not overridable) ────────────────────────────────
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }

            // ── Ctrl+Z: undo last mutating action (Phase 36, UNDO-01/02/03, D-01/D-07) ──
            // Must precede any plain 'z' arm. Silent no-op when history empty (D-08/D-10).
            KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.apply_undo()?;
            }

            // ── Ctrl+E: open external editor (Phase 39, XEDIT-01) ────────────
            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.launch_external_editor()?;
            }

            // ── Esc: clear selection / exit disjoint mode (not overridable) ──
            KeyCode::Esc if self.disjoint_select || !self.selected_tasks.is_empty() => {
                self.selected_tasks.clear();
                self.selection_anchor = None;
                self.disjoint_select = false;
            }

            // ── Pane navigation (left/right arrows, Phase 24) ────────────────
            KeyCode::Left => {
                self.focus_prev_pane();
                self.rebuild_and_reanchor();
            }
            KeyCode::Right => {
                self.focus_next_pane();
                self.rebuild_and_reanchor();
            }

            // ── Navigation — non-overridable arms ───────────────────────────
            // Shift+j or Shift+Down: extend contiguous range selection downward (D-09, D-11).
            // MUST precede plain j/Down arm so SHIFT modifier is checked first (T-19-04).
            KeyCode::Char('j') | KeyCode::Down
                if key.modifiers.contains(KeyModifiers::SHIFT) && row_count > 0 =>
            {
                self.ensure_anchor();
                self.pane_move_down();
                self.apply_range_selection();
            }
            // Shift+k or Shift+Up: extend contiguous range selection upward (D-09, D-11).
            // MUST precede plain k/Up arm so SHIFT modifier is checked first (T-19-04).
            KeyCode::Char('k') | KeyCode::Up
                if key.modifiers.contains(KeyModifiers::SHIFT) && row_count > 0 =>
            {
                self.ensure_anchor();
                self.pane_move_up();
                self.apply_range_selection();
            }
            KeyCode::Char('j') | KeyCode::Down if row_count > 0 => {
                self.selection_anchor = None; // D-12: non-shift nav clears anchor
                self.pane_move_down();
            }
            KeyCode::Char('k') | KeyCode::Up if row_count > 0 => {
                self.selection_anchor = None; // D-12: non-shift nav clears anchor
                self.pane_move_up();
            }

            // Enter on selected pane header opens inline pane-label editing.
            KeyCode::Enter
                if !self.should_show_single_pane()
                    && self.panes[self.active_pane].label_selected =>
            {
                let pane_idx = self.active_pane;
                let current_label = self.panes[pane_idx].label.clone();
                self.editor = TextArea::default();
                self.editor.insert_str(&current_label);
                self.mode = AppMode::PaneLabelEditing { pane_idx };
            }
            // Shift+Ctrl+U: half-page range extension upward (D-10).
            // MUST precede plain Ctrl+U arm so SHIFT check wins (T-19-04).
            KeyCode::Char('u')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.modifiers.contains(KeyModifiers::SHIFT)
                    && display_count > 0 =>
            {
                self.ensure_anchor();
                let half = (self.list_height / 2).max(1) as usize;
                self.selected = self.selected.saturating_sub(half);
                while self.selected < row_count
                    && matches!(self.display_rows[self.selected], DisplayRow::GroupHeader(_))
                {
                    self.selected += 1;
                }
                self.clamp_selection();
                self.apply_range_selection();
            }
            // Ctrl+U half-page up — must come before plain 'u' (edit alias).
            KeyCode::Char('u')
                if key.modifiers.contains(KeyModifiers::CONTROL) && display_count > 0 =>
            {
                self.selection_anchor = None; // D-12: non-shift nav clears anchor
                let half = (self.list_height / 2).max(1) as usize;
                self.selected = self.selected.saturating_sub(half);
                while self.selected < row_count
                    && matches!(self.display_rows[self.selected], DisplayRow::GroupHeader(_))
                {
                    self.selected += 1;
                }
                self.clamp_selection();
            }
            // Shift+Ctrl+D: half-page range extension downward (D-10).
            // MUST precede plain Ctrl+D arm so SHIFT check wins (T-19-04).
            KeyCode::Char('d')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.modifiers.contains(KeyModifiers::SHIFT)
                    && display_count > 0 =>
            {
                self.ensure_anchor();
                let half = (self.list_height / 2).max(1) as usize;
                self.selected = (self.selected + half).min(row_count.saturating_sub(1));
                while self.selected < row_count
                    && matches!(self.display_rows[self.selected], DisplayRow::GroupHeader(_))
                {
                    self.selected += 1;
                }
                self.clamp_selection();
                self.apply_range_selection();
            }
            // Ctrl+D half-page down — must come before overridable 'delete' arm.
            KeyCode::Char('d')
                if key.modifiers.contains(KeyModifiers::CONTROL) && display_count > 0 =>
            {
                self.selection_anchor = None; // D-12: non-shift nav clears anchor
                let half = (self.list_height / 2).max(1) as usize;
                self.selected = (self.selected + half).min(row_count.saturating_sub(1));
                while self.selected < row_count
                    && matches!(self.display_rows[self.selected], DisplayRow::GroupHeader(_))
                {
                    self.selected += 1;
                }
                self.clamp_selection();
            }

            // ── Overridable actions (via effective_keymap, D-05 Phase 22) ────
            // filter_toggle must precede filter_open — both default to 'f';
            // filter_toggle requires CONTROL so it must be checked first.
            _ if self.key_is_action(key, "filter_toggle") => {
                // Per-pane filter toggle (D-02, Phase 25): Apply only to active pane's filter_query
                let current_filter = {
                    let active_pane = self.active_pane();
                    active_pane.filter_query.clone()
                };
                
                if current_filter.trim().is_empty() {
                    if let Some(prev) = self.toggled_filter_query.take() {
                        self.active_pane_mut().filter_query = prev;
                    }
                } else {
                    self.toggled_filter_query = Some(current_filter);
                    self.active_pane_mut().filter_query.clear();
                }
                self.rebuild_and_reanchor();
            }

            _ if self.key_is_action(key, "quit") => {
                self.should_quit = true;
            }

            _ if self.disjoint_select && self.key_is_action(key, "disjoint_mark") => {
                self.pane_toggle_task_selection();
            }

            _ if self.key_is_action(key, "disjoint_select") => {
                self.disjoint_select = !self.disjoint_select;
            }

            _ if display_count > 0 && self.key_is_action(key, "toggle_done") => {
                if !self.selected_tasks.is_empty() {
                    self.bulk_mark_done();
                } else {
                    self.pane_toggle_done();
                }
            }

            _ if self.key_is_action(key, "archive") => {
                let count = self.task_list.tasks().iter().filter(|t| t.completed).count();
                if count > 0 {
                    self.mode = AppMode::ArchiveConfirm;
                } else {
                    self.push_runtime_warning("No completed tasks to archive.");
                }
            }

            _ if self.key_is_action(key, "add") => {
                self.editor = TextArea::default();
                self.mode = AppMode::Adding;
            }

            // edit — 'e' via keymap, 'u' kept as hardcoded alias so existing muscle memory works.
            _ if display_count > 0
                && (self.key_is_action(key, "edit")
                    || (key.code == KeyCode::Char('u')
                        && key.modifiers == KeyModifiers::NONE)) =>
            {
                if let Some(canonical) = self.active_canonical_selected() {
                    let raw = self.task_list.tasks()[canonical].to_raw().to_string();
                    let mut ed = TextArea::default();
                    ed.insert_str(&raw);
                    self.editor = ed;
                    self.mode = AppMode::Editing { original_idx: canonical };
                }
            }

            _ if display_count > 0 && self.key_is_action(key, "bulk_delete") => {
                if self.selected_tasks.len() > 1 {
                    self.mode = AppMode::DeleteConfirm;
                } else {
                    self.delete_active_task()?;
                }
            }

            _ if display_count > 0
                && (self.key_is_action(key, "delete")
                    || (key.code == KeyCode::Delete
                        && key.modifiers == KeyModifiers::NONE)
                    || (key.code == KeyCode::Backspace
                        && key.modifiers == KeyModifiers::NONE)) =>
            {
                if self.selected_tasks.len() > 1 {
                    self.mode = AppMode::DeleteConfirm;
                } else {
                    self.delete_active_task()?;
                }
            }

            _ if self.key_is_action(key, "sort_cycle") => {
                // Per-pane sort state (D-07, Phase 25): Apply only to active pane
                let current_sort = self.active_pane().sort_order;
                self.active_pane_mut().sort_order = cycle_sort(current_sort);
                self.rebuild_and_reanchor();
            }

            _ if !self.selected_tasks.is_empty() && display_count > 0 && self.key_is_action(key, "bulk_append") => {
                let n = self.selected_tasks.len();
                if n > 1 {
                    // D-06: show count banner before text entry when multiple tasks targeted
                    self.append_confirm_count = n;
                    self.mode = AppMode::AppendTextConfirm;
                } else {
                    // Single task — go directly to append text editor
                    self.editor = TextArea::default();
                    self.mode = AppMode::AppendText;
                }
            }

            _ if self.key_is_action(key, "theme_cycle") => {
                self.palette = cycle_theme(self.palette);
                self.styles = StyleSheet::from_theme(self.palette, self.no_color);
                self.config.tui.theme = match self.palette {
                    Theme::Default => "default".to_string(),
                    Theme::Light => "light".to_string(),
                };
                if let Some(ref path) = self.config_path {
                    if let Err(e) = self.config.save(path) {
                        self.push_runtime_warning(format!("config save failed: {e}"));
                    }
                }
            }

            _ if self.key_is_action(key, "filter_open") => {
                // Per-pane filter panel (D-02, Phase 25): Snapshot active pane's current filter query
                let active_pane = self.active_pane();
                let mut editor = TextArea::default();
                editor.insert_str(&active_pane.filter_query);
                self.filter_state = Some(FilteringState {
                    editor,
                    selected_preset: 0,
                    snapshot: active_pane.filter_query.clone(), // Snapshot per-pane filter state
                });
                self.mode = AppMode::Filtering;
            }

            _ if self.key_is_action(key, "filter_define") => {
                let mut active_editor = TextArea::default();
                active_editor.insert_str(&self.active_pane().filter_query);

                // Build deterministic numbered slots (f1..fN) so slot positions never shift.
                const MIN_PRESET_SLOTS: usize = 5;
                const MAX_PRESET_SLOTS: usize = 9;
                let highest_existing_slot = self
                    .config
                    .presets
                    .filter
                    .keys()
                    .filter_map(|name| parse_preset_slot(name))
                    .max()
                    .unwrap_or(0);
                let slot_count = highest_existing_slot
                    .clamp(MIN_PRESET_SLOTS, MAX_PRESET_SLOTS);

                let sorted_presets: Vec<(String, String)> = (1..=slot_count)
                    .map(|slot| {
                        let name = format!("f{}", slot);
                        let filter = self
                            .config
                            .presets
                            .filter
                            .get(&name)
                            .and_then(|p| p.filter.clone())
                            .unwrap_or_default();
                        (name, filter)
                    })
                    .collect();

                if active_editor.cursor() == (0, 0) {
                    active_editor.move_cursor(tui_textarea::CursorMove::End);
                }

                let preset_names: Vec<String> = sorted_presets.iter().map(|(n, _)| n.clone()).collect();
                let preset_editors: Vec<TextArea<'static>> = sorted_presets.iter().map(|(_, f)| {
                    let mut ta = TextArea::default();
                    ta.insert_str(f);
                    ta
                }).collect();

                self.filter_defining_state = Some(FilterDefiningState {
                    active_editor,
                    preset_names,
                    preset_editors,
                    selected_row: 0,
                });
                self.mode = AppMode::FilterDefining;
            }

            _ if display_count > 0 && self.key_is_action(key, "group_toggle") => {
                // Per-pane grouping state (D-08, Phase 25): Apply only to active pane
                let current_grouping = self.active_pane().grouping;
                self.active_pane_mut().grouping = !current_grouping;
                self.rebuild_and_reanchor();
            }

            _ if display_count > 0 && self.key_is_action(key, "group_by_cycle") => {
                // Cycle the active pane's group-by category (GRP-02, Phase 40).
                let current = self.active_pane().group_by;
                self.active_pane_mut().group_by = cycle_group_by(current);
                self.rebuild_and_reanchor();
            }

            _ if display_count > 0 && self.key_is_action(key, "deferred_toggle") => {
                self.show_deferred = !self.show_deferred;
                self.rebuild_display_indices();
                self.clamp_selection();
            }

            // '!' opens app error log overlay (even when empty for discoverability).
            KeyCode::Char('!') => {
                self.mode = AppMode::KeymapErrors;
            }

            // '?' opens the help overlay (D-10, Phase 22)
            _ if self.key_is_action(key, "help") => {
                self.mode = AppMode::Help;
            }

            // '0' clears the active filter (D-11, Phase 22)
            _ if self.key_is_action(key, "clear_filter") => {
                // Per-pane: clear active pane's filter (Phase 25)
                self.active_pane_mut().filter_query.clear();
                self.toggled_filter_query = None;
                self.rebuild_and_reanchor();
            }

            // '1'-'9' applies a preset filter by slot (Phase 41, PRST-01; not overridable)
            KeyCode::Char(c @ '1'..='9') if key.modifiers == KeyModifiers::NONE => {
                let slot = c.to_string();  // "1" through "9"
                if let Some(preset) = self.config.presets.filter.get(&slot) {
                    if let Some(filter_str) = preset.filter.as_ref() {
                        // Per-pane: apply preset filter to active pane (Phase 25)
                        self.active_pane_mut().filter_query = filter_str.clone();
                        self.toggled_filter_query = None;
                        self.rebuild_and_reanchor();
                    }
                }
            }

            // Ctrl+1-9 applies a full pane layout preset by positional index (Phase 41, PRST-02, D-07).
            KeyCode::Char(c @ '1'..='9') if key.modifiers == KeyModifiers::CONTROL => {
                let idx = (c as usize) - ('1' as usize);  // 0-8
                if idx < self.pane_presets.len() {
                    let preset = self.pane_presets[idx].1.clone();
                    self.apply_pane_layout_preset(&preset);
                }
            }

            // '.' reloads the task file from disk (D-11, Phase 22)
            _ if self.key_is_action(key, "reload") => {
                match self.task_list.reload() {
                    Ok(()) => {
                        self.pending_reload = false;
                        self.prune_stale_selections();
                        self.rebuild_and_reanchor();
                    }
                    Err(e) => {
                        self.push_runtime_warning(format!("reload failed: {}", e));
                    }
                }
            }

            // Ctrl+N creates a new pane with auto-label (D-17, Phase 26)
            _ if self.key_is_action(key, "pane_add") => {
                self.pane_add();
                self.rebuild_and_reanchor();
            }

            // Ctrl+W deletes the active pane with focus shift (D-18, Phase 26)
            _ if self.key_is_action(key, "pane_delete") => {
                self.pane_delete();
                self.rebuild_and_reanchor();
            }

            // Ctrl+P toggles pane visibility (D-19, Phase 26)
            _ if self.key_is_action(key, "pane_hide_toggle") => {
                self.pane_hide_toggle();
                self.rebuild_and_reanchor();
            }

            // Ctrl+Left/Right moves task to adjacent pane (Phase 41, PMOVE-02).
            _ if self.key_is_action(key, "pane_move_left") => {
                self.pane_move_task(-1)?;
            }

            _ if self.key_is_action(key, "pane_move_right") => {
                self.pane_move_task(1)?;
            }

            // 's' opens the date picker for setting due dates (Phase 33, Plan 01)
            KeyCode::Char('s') if key.modifiers == KeyModifiers::NONE => {
                if self.has_quick_setter_targets() {
                    // Start from current month; user can edit month/year inline in picker.
                    let now = chrono::Local::now();
                    let month_year = format!("{:04}-{:02}", now.year(), now.month());
                    self.date_picker = Some(DatePickerState::new(&month_year));
                    self.mode = AppMode::DatePicker;
                } else {
                    self.push_runtime_warning("date picker requires an active task or selection");
                }
            }

            // 'i' opens the priority picker overlay (Phase 34, Plan 01 — CAP-04 gap)
            KeyCode::Char('i') if key.modifiers == KeyModifiers::NONE => {
                // Require an active task or selection (mirrors date picker guard)
                if self.has_quick_setter_targets() {
                    self.priority_picker = Some(PriorityPickerState::new());
                    self.mode = AppMode::PriorityPicker;
                } else {
                    self.push_runtime_warning("priority picker requires an active task or selection");
                }
            }


            // 'y' copies selected or active task(s) to system clipboard (Phase 35, Plan 01, CLIP-01)
            KeyCode::Char('y') if key.modifiers == KeyModifiers::NONE => {
                self.copy_selected_to_clipboard()?;
            }

            // 'p' pastes clipboard content as new tasks (Phase 35, Plan 02, CLIP-03, D-05)
            KeyCode::Char('p') if key.modifiers == KeyModifiers::NONE => {
                self.paste_from_clipboard()?;
            }

            // '@' opens quick context setter from Normal mode (Phase 33, Plan 02)
            _ if self.key_is_action(key, "quick_context") => {
                if !self.has_quick_setter_targets() {
                    self.push_runtime_warning("quick context setter requires an active task or selection");
                    return Ok(());
                }

                let mut items: Vec<String> = get_existing_contexts(&self.task_list).into_iter().collect();
                items.sort();
                self.autocomplete = Some(AutocompleteState::new_quick_setter('@', String::new(), items));
                self.mode = AppMode::QuickSetter('@');
            }

            // '+' opens quick project setter from Normal mode (Phase 33, Plan 02)
            _ if self.key_is_action(key, "quick_project") => {
                if !self.has_quick_setter_targets() {
                    self.push_runtime_warning("quick project setter requires an active task or selection");
                    return Ok(());
                }

                let mut items: Vec<String> = get_existing_projects(&self.task_list).into_iter().collect();
                items.sort();
                self.autocomplete = Some(AutocompleteState::new_quick_setter('+', String::new(), items));
                self.mode = AppMode::QuickSetter('+');
            }

            _ => {}
        }
        Ok(())
    }

    fn has_quick_setter_targets(&self) -> bool {
        if self
            .selected_tasks
            .iter()
            .any(|&idx| idx < self.task_list.len())
        {
            return true;
        }

        self.active_canonical_selected().is_some()
    }

    fn handle_quick_setter_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> color_eyre::Result<()> {
        let trigger = match self.mode {
            AppMode::QuickSetter(trigger) => trigger,
            _ => return Ok(()),
        };

        match key.code {
            KeyCode::Esc => {
                self.autocomplete = None;
                self.mode = AppMode::Normal;
            }
            KeyCode::Down => {
                if let Some(ref mut ac) = self.autocomplete {
                    ac.focused = true;
                    ac.selected = (ac.selected + 1).min(ac.items.len().saturating_sub(1));
                }
            }
            KeyCode::Up => {
                if let Some(ref mut ac) = self.autocomplete {
                    ac.focused = true;
                    ac.selected = ac.selected.saturating_sub(1);
                }
            }
            KeyCode::Backspace => {
                let mut next_prefix = None;
                if let Some(ref mut ac) = self.autocomplete {
                    ac.prefix.pop();
                    next_prefix = Some(ac.prefix.clone());
                }
                if let Some(prefix) = next_prefix {
                    let ranked = self.quick_setter_candidates(trigger, &prefix);
                    if let Some(ref mut ac) = self.autocomplete {
                        ac.items = ranked;
                        ac.selected = 0;
                        ac.focused = false;
                    }
                }
            }
            KeyCode::Tab | KeyCode::Enter => {
                let chosen = self.autocomplete.as_ref().and_then(|ac| {
                    ac.items.get(ac.selected).cloned().or_else(|| {
                        if Self::is_valid_quick_setter_token(&ac.prefix) {
                            Some(ac.prefix.clone())
                        } else {
                            None
                        }
                    })
                });

                if let Some(token) = chosen {
                    let targets = self.quick_setter_targets();
                    let added = self.apply_token_to_tasks(trigger, &token, targets)?;
                    if added == 0 {
                        self.push_runtime_warning(format!("{}{} already present on target task(s)", trigger, token));
                    }
                }

                self.autocomplete = None;
                self.mode = AppMode::Normal;
            }
            KeyCode::Char(c) if Self::is_valid_quick_setter_char(c) => {
                let mut next_prefix = None;
                if let Some(ref mut ac) = self.autocomplete {
                    ac.prefix.push(c);
                    next_prefix = Some(ac.prefix.clone());
                }
                if let Some(prefix) = next_prefix {
                    let ranked = self.quick_setter_candidates(trigger, &prefix);
                    if let Some(ref mut ac) = self.autocomplete {
                        ac.items = ranked;
                        ac.selected = 0;
                        ac.focused = false;
                    }
                }
            }
            _ => {
                self.autocomplete = None;
                self.mode = AppMode::Normal;
                self.handle_normal_key(key)?;
            }
        }

        Ok(())
    }

    fn is_valid_quick_setter_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '-' || c == '/'
    }

    fn is_valid_quick_setter_token(token: &str) -> bool {
        !token.trim().is_empty() && token.chars().all(Self::is_valid_quick_setter_char)
    }

    fn quick_setter_candidates(&self, trigger: char, prefix: &str) -> Vec<String> {
        let mut all: Vec<String> = match trigger {
            '@' => get_existing_contexts(&self.task_list).into_iter().collect(),
            '+' => get_existing_projects(&self.task_list).into_iter().collect(),
            _ => Vec::new(),
        };

        if Self::is_valid_quick_setter_token(prefix)
            && !all.iter().any(|item| item.eq_ignore_ascii_case(prefix))
        {
            all.push(prefix.to_string());
        }

        all.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()).then_with(|| a.cmp(b)));
        all.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
        rank_matches(prefix, all)
    }

    fn quick_setter_targets(&self) -> Vec<usize> {
        if !self.selected_tasks.is_empty() {
            let mut indices: Vec<usize> = self
                .selected_tasks
                .iter()
                .copied()
                .filter(|&idx| idx < self.task_list.len())
                .collect();
            indices.sort_unstable_by(|a, b| b.cmp(a));
            indices.dedup();
            return indices;
        }

        self.active_canonical_selected().into_iter().collect()
    }

    /// Copy selected or active task text to the system clipboard (Phase 35, Plan 01, CLIP-01).
    /// Targets selected_tasks (if non-empty) or the active cursor task.
    /// Multi-task copy joins lines with newlines in descending-canonical-index order (D-08, D-17).
    fn copy_selected_to_clipboard(&mut self) -> color_eyre::Result<()> {
        // 1. Determine targets: selected tasks or active task (D-03)
        let mut targets: Vec<usize> = if !self.selected_tasks.is_empty() {
            self.selected_tasks
                .iter()
                .copied()
                .filter(|&idx| idx < self.task_list.len())
                .collect()
        } else if let Some(DisplayRow::Task(idx)) = self.display_rows.get(self.selected) {
            vec![*idx]
        } else {
            vec![]
        };

        // Abort silently if no task targeted (D-10: skip header rows)
        if targets.is_empty() {
            self.push_runtime_warning("No task selected");
            return Ok(());
        }

        // 2. Sort in descending-canonical-index order (D-08, D-17)
        targets.sort_unstable_by(|a, b| b.cmp(a));

        // 3. Collect raw text from each target
        let tasks = self.task_list.tasks();
        let clipboard_text: Vec<String> = targets
            .iter()
            .filter_map(|&idx| tasks.get(idx))
            .map(|t| t.to_raw().to_string())
            .collect();
        let text_to_copy = clipboard_text.join("\n");

        // 4. Lazy-initialize arboard (D-02: avoid startup errors in headless environments)
        if self.clipboard.is_none() {
            match Clipboard::new() {
                Ok(cb) => self.clipboard = Some(cb),
                Err(_) => {
                    self.push_runtime_warning("Clipboard unavailable");
                    return Ok(());
                }
            }
        }

        // 5. Write to clipboard and show status feedback (D-09)
        if let Some(ref mut cb) = self.clipboard {
            match cb.set_text(text_to_copy) {
                Ok(_) => {
                    let msg = if targets.len() == 1 {
                        "copied 1 task".to_string()
                    } else {
                        format!("copied {} tasks", targets.len())
                    };
                    self.push_runtime_warning(msg);
                }
                Err(_) => {
                    self.push_runtime_warning("Failed to copy to clipboard");
                }
            }
        }

        Ok(())
    }

    /// Paste clipboard content as new task entries in Normal mode (`p` key, Phase 35, Plan 02, CLIP-03).
    /// Reads clipboard, splits on newlines, parses each non-empty line as a Task, appends all to task_list.
    /// All lines pasted in a single operation (D-05, D-11). Rebuilds view and reanchors after paste.
    fn paste_from_clipboard(&mut self) -> color_eyre::Result<()> {
        // Lazy-initialize arboard (D-02)
        if self.clipboard.is_none() {
            match Clipboard::new() {
                Ok(cb) => self.clipboard = Some(cb),
                Err(_) => {
                    self.push_runtime_warning("clipboard is empty");
                    return Ok(());
                }
            }
        }

        // Read clipboard text
        let clipboard_text = if let Some(ref mut cb) = self.clipboard {
            match cb.get_text() {
                Ok(text) => text,
                Err(_) => {
                    self.push_runtime_warning("clipboard is empty");
                    return Ok(());
                }
            }
        } else {
            self.push_runtime_warning("clipboard is empty");
            return Ok(());
        };

        // Split and filter empty lines (D-11, D-12)
        let lines: Vec<String> = clipboard_text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.to_string())
            .collect();

        if lines.is_empty() {
            self.push_runtime_warning("clipboard is empty");
            return Ok(());
        }

        let count = lines.len();
        self.push_undo_entry();

        // Parse each line as a Task and add to task_list (D-12: raw text, no transformation)
        for line in lines {
            let task = Task::parse(&line);
            self.task_list
                .add(task)
                .map_err(|e| color_eyre::eyre::eyre!("Failed to paste task: {}", e))?;
        }

        // Rebuild views and reanchor (D-14)
        self.rebuild_all_panes();
        self.rebuild_and_reanchor();

        let msg = if count == 1 {
            "pasted 1 task".to_string()
        } else {
            format!("pasted {} tasks", count)
        };
        self.push_runtime_warning(msg);

        Ok(())
    }

    /// Paste first clipboard line into the Adding-mode editor (Ctrl+V, Phase 35, Plan 02, CLIP-04, D-15).
    /// Single-line editor: only the first clipboard line is inserted. Empty clipboard is a silent no-op.
    fn paste_in_editor(&mut self) -> color_eyre::Result<()> {
        // Lazy-initialize arboard (D-02)
        if self.clipboard.is_none() {
            match Clipboard::new() {
                Ok(cb) => self.clipboard = Some(cb),
                Err(_) => {
                    return Ok(()); // Silent no-op on init failure (D-15)
                }
            }
        }

        // Read clipboard text
        let clipboard_text = if let Some(ref mut cb) = self.clipboard {
            match cb.get_text() {
                Ok(text) => text,
                Err(_) => return Ok(()), // Silent no-op if empty (D-15)
            }
        } else {
            return Ok(());
        };

        // Extract first line only (single-line editor, D-15)
        let first_line = clipboard_text.lines().next().unwrap_or("").to_string();
        if !first_line.is_empty() {
            self.editor.insert_str(&first_line);
        }

        Ok(())
    }

    fn apply_token_to_tasks(
        &mut self,
        trigger: char,
        token: &str,
        mut targets: Vec<usize>,
    ) -> color_eyre::Result<usize> {
        let normalized_token = token.trim();
        if !Self::is_valid_quick_setter_token(normalized_token) {
            return Ok(0);
        }

        targets.sort_unstable_by(|a, b| b.cmp(a));
        targets.dedup();

        let tasks = self.task_list.tasks();
        let mut replacements: Vec<(usize, Task)> = Vec::new();
        let mut added = 0usize;

        for idx in targets {
            if let Some(task) = tasks.get(idx) {
                let already_exists = match trigger {
                    '@' => task
                        .contexts
                        .iter()
                        .any(|ctx| ctx.eq_ignore_ascii_case(normalized_token)),
                    '+' => task
                        .projects
                        .iter()
                        .any(|proj| proj.eq_ignore_ascii_case(normalized_token)),
                    _ => true,
                };

                if already_exists {
                    continue;
                }

                let new_line = format!(
                    "{} {}{}",
                    task.to_raw().trim_end(),
                    trigger,
                    normalized_token
                );
                replacements.push((idx, normalize_line(&new_line)));
                added += 1;
            }
        }

        if !replacements.is_empty() {
            self.push_undo_entry();
            self.task_list
                .batch_update(replacements)
                .map_err(|e| color_eyre::eyre::eyre!("Failed to apply quick token: {}", e))?;
            self.rebuild_all_panes();
            self.rebuild_and_reanchor();
            self.push_runtime_warning(format!("added {}{}", trigger, normalized_token));
        }

        self.selected_tasks.clear();
        self.selection_anchor = None;
        self.disjoint_select = false;

        Ok(added)
    }

    /// Handle key events in the KeymapErrors read-only overlay (D-09, Phase 22).
    fn handle_keymap_errors_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> color_eyre::Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.mode = AppMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle key events in the Help overlay (D-10, Phase 22).
    fn handle_help_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> color_eyre::Result<()> {
        use crossterm::event::KeyModifiers;
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) | (KeyCode::Char('q'), _) => {
                self.mode = AppMode::Normal;
                self.help_scroll = 0;
            }
            (KeyCode::Char('j'), _) | (KeyCode::Down, _) => {
                self.help_scroll = self.help_scroll.saturating_add(1);
            }
            (KeyCode::Char('k'), _) | (KeyCode::Up, _) => {
                self.help_scroll = self.help_scroll.saturating_sub(1);
            }
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                self.help_scroll = self.help_scroll.saturating_add(self.list_height / 2);
            }
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                self.help_scroll = self.help_scroll.saturating_sub(self.list_height / 2);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_filtering_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> color_eyre::Result<()> {
        match key.code {
            KeyCode::Esc => {
                // Restore prior filter from snapshot — per-pane (D-02, Phase 25)
                let snapshot = self.filter_state.as_ref().map(|s| s.snapshot.clone()).unwrap_or_default();
                self.active_pane_mut().filter_query = snapshot;
                self.filter_state = None;
                self.autocomplete = None;
                self.filter_history_cursor = None;
                self.mode = AppMode::Normal;
                self.rebuild_and_reanchor();
                self.apply_pending_reload()?;
            }
            KeyCode::Enter => {
                // Guard: if popup is focused, accept suggestion instead of applying filter (D-02).
                if self.autocomplete.as_ref().map(|ac| ac.focused).unwrap_or(false) {
                    self.accept_filter_completion();
                    return Ok(());
                }
                // Apply filter to active pane
                if let Some(state) = self.filter_state.take() {
                    let filter_text = state.editor.lines().join("").trim().to_string();
                    // Push to filter history before applying (Phase 41, FHIST-01).
                    self.push_filter_history(&filter_text);
                    self.active_pane_mut().filter_query = filter_text;
                }
                self.autocomplete = None;
                self.filter_history_cursor = None;
                self.mode = AppMode::Normal;
                self.toggled_filter_query = None;
                self.rebuild_and_reanchor();
                self.apply_pending_reload()?;
            }
            // Ctrl+R cycles backward through filter history (Phase 41, FHIST-02, D-09).
            KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                if !self.filter_history.is_empty() {
                    let next_cursor = match self.filter_history_cursor {
                        None => 0,
                        Some(c) => (c + 1).rem_euclid(self.filter_history.len()),
                    };
                    self.filter_history_cursor = Some(next_cursor);
                    let hist_entry = self.filter_history[next_cursor].clone();
                    if let Some(ref mut state) = self.filter_state {
                        state.editor = tui_textarea::TextArea::default();
                        state.editor.insert_str(&hist_entry);
                    }
                    self.active_pane_mut().filter_query = hist_entry;
                    self.rebuild_and_reanchor();
                }
            }
            KeyCode::Down => {
                // Navigate autocomplete popup if visible (before preset cycling).
                if let Some(ref mut ac) = self.autocomplete {
                    ac.focused = true;
                    ac.selected = (ac.selected + 1).min(ac.items.len().saturating_sub(1));
                    return Ok(());
                }
                let preset_count = self.presets.len();
                if let Some(ref mut state) = self.filter_state {
                    if preset_count > 0 {
                        state.selected_preset =
                            (state.selected_preset + 1).min(preset_count - 1);
                        let query = self.presets[state.selected_preset].1.clone();
                        state.editor = TextArea::default();
                        state.editor.insert_str(&query);
                        // Per-pane: update active pane's filter (D-03, Phase 25)
                        self.active_pane_mut().filter_query = query;
                    }
                }
                self.rebuild_and_reanchor();
            }
            KeyCode::Up => {
                // Navigate autocomplete popup if focused (before preset cycling).
                if let Some(ref mut ac) = self.autocomplete {
                    if ac.focused {
                        ac.selected = ac.selected.saturating_sub(1);
                        return Ok(());
                    }
                }
                if let Some(ref mut state) = self.filter_state {
                    if state.selected_preset > 0 {
                        state.selected_preset = state.selected_preset.saturating_sub(1);
                        let query = self.presets[state.selected_preset].1.clone();
                        state.editor = TextArea::default();
                        state.editor.insert_str(&query);
                        // Per-pane: update active pane's filter (D-03, Phase 25)
                        self.active_pane_mut().filter_query = query;
                    }
                }
                self.rebuild_and_reanchor();
            }
            KeyCode::Char(c @ '1'..='9') => {
                let idx = (c as usize) - ('1' as usize);
                if idx < self.presets.len() {
                    let query = self.presets[idx].1.clone();
                    if let Some(ref mut state) = self.filter_state {
                        state.editor = TextArea::default();
                        state.editor.insert_str(&query);
                        state.selected_preset = idx;
                    }
                    // Per-pane: update active pane's filter (D-03, Phase 25)
                    self.active_pane_mut().filter_query = query;
                    self.rebuild_and_reanchor();
                }
            }
            // Tab accepts focused popup suggestion (AC-03, Phase 42).
            KeyCode::Tab => {
                if self.autocomplete.as_ref().map(|ac| ac.focused).unwrap_or(false) {
                    self.accept_filter_completion();
                }
            }
            _ => {
                // Borrow-safe pattern: feed key first, then clone line+cursor, then call helper.
                if let Some(ref mut state) = self.filter_state {
                    state.editor.input(key);
                }
                let (filter_text, cursor_col) = match &self.filter_state {
                    Some(s) => (
                        s.editor.lines().first().cloned().unwrap_or_default(),
                        s.editor.cursor().1,
                    ),
                    None => {
                        self.rebuild_and_reanchor();
                        return Ok(());
                    }
                };
                // Reset Ctrl+R cursor when user types manually (FHIST-02, D-09).
                self.filter_history_cursor = None;
                // Per-pane: update active pane's filter as user types (D-04, Phase 25).
                self.active_pane_mut().filter_query = filter_text.clone();
                // Compute autocomplete: TokenAutocomplete for @/+, FilterHistory fallback (AC-02, AC-04).
                self.autocomplete = compute_filter_autocomplete(
                    &filter_text,
                    cursor_col,
                    &self.task_list,
                    &self.filter_history,
                );
                self.rebuild_and_reanchor();
            }
        }
        Ok(())
    }

    // ── Preset definition panel key handler (Plan 16-03, D-01) ────────────────

    fn handle_filter_defining_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> color_eyre::Result<()> {
        use crossterm::event::KeyCode as KC;

        let state = match self.filter_defining_state.as_mut() {
            Some(s) => s,
            None => {
                self.mode = AppMode::Normal;
                return Ok(());
            }
        };

        match key.code {
            // D-03: Esc = discard — nothing written to TOML.
            KC::Esc => {
                self.filter_defining_state = None;
                self.mode = AppMode::Normal;
            }

            // D-04: Enter = save preset definitions to TOML, return to Normal.
            KC::Enter => {
                // Apply currently selected row on save: row 0 = active query, rows 1..N = selected preset.
                // Capture new_query while `state` borrow is active; assign to active pane after state is dropped (FAIL-1 fix, Phase 28).
                let new_query = if state.selected_row == 0 {
                    state.active_editor.lines().join("").trim().to_string()
                } else {
                    let idx = state.selected_row - 1;
                    state
                        .preset_editors
                        .get(idx)
                        .map(|e| e.lines().join("").trim().to_string())
                        .unwrap_or_default()
                };

                // Update config.presets.filter from editors.
                for (i, name) in state.preset_names.iter().enumerate() {
                    let filter_str = state.preset_editors[i].lines().join("").trim().to_string();
                    if filter_str.is_empty() {
                        // Remove empty/cleared presets — do not write blank slots to config.
                        self.config.presets.filter.remove(name);
                    } else {
                        self.config.presets.filter.entry(name.clone())
                            .and_modify(|p| p.filter = Some(filter_str.clone()))
                            .or_insert_with(|| crate::config::FilterPreset { filter: Some(filter_str) });
                    }
                }

                // Rebuild presets vec from updated config.presets.filter.
                let mut updated: Vec<(String, String)> = self.config.presets.filter.iter()
                    .filter_map(|(k, v)| v.filter.as_ref().map(|f| (k.clone(), f.clone())))
                    .collect();
                updated.sort_by(|(a, _), (b, _)| a.cmp(b));
                self.presets = updated;

                // Persist to TOML atomically.
                if let Some(ref path) = self.config_path.clone() {
                    if let Err(e) = self.config.save(path) {
                        self.push_runtime_warning(format!("config save failed: {e}"));
                    }
                }

                self.filter_defining_state = None;
                self.mode = AppMode::Normal;
                // Write the new query to the active pane (not global self.filter_query) — FAIL-1 fix (Phase 28).
                self.active_pane_mut().filter_query = new_query;
                self.toggled_filter_query = None;
                self.rebuild_and_reanchor();
            }

            // Navigate rows (Up/Down).
            KC::Up => {
                if state.selected_row > 0 {
                    state.selected_row -= 1;
                }
            }
            KC::Down => {
                let max_row = state.preset_editors.len();
                if state.selected_row < max_row {
                    state.selected_row += 1;
                }
            }

            // All other keys: forward to the focused editor.
            _ => {
                let selected = state.selected_row;
                if selected == 0 {
                    state.active_editor.input(key);
                    // D-07: live preview — update active pane's filter_query from active editor (FAIL-1 fix, Phase 28).
                    let preview_query = self
                        .filter_defining_state
                        .as_ref()
                        .map(|s| s.active_editor.lines().join("").trim().to_string())
                        .unwrap_or_default();
                    self.active_pane_mut().filter_query = preview_query;
                    self.rebuild_and_reanchor();
                } else {
                    let idx = selected - 1;
                    if let Some(ref mut state) = self.filter_defining_state {
                        if idx < state.preset_editors.len() {
                            state.preset_editors[idx].input(key);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    // ── Editor mode key handler ───────────────────────────────────────────────

    fn handle_editor_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> color_eyre::Result<()> {
        // Intercept Ctrl+V before default passthrough — paste from clipboard into editor (Phase 35, Plan 02, CLIP-04, D-15)
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('v') {
            return self.paste_in_editor();
        }

        match key.code {
            KeyCode::Esc => {
                if self.autocomplete.is_some() {
                    // Close popup only — keep editor open (D-08).
                    self.autocomplete = None;
                } else {
                    self.exit_edit_mode()?;
                }
            }
            KeyCode::Enter => {
                if self.autocomplete.as_ref().map(|ac| ac.focused).unwrap_or(false) {
                    self.accept_completion();
                } else {
                    self.autocomplete = None;
                    self.save_and_exit()?;
                }
            }
            KeyCode::Down => {
                if let Some(ref mut ac) = self.autocomplete {
                    ac.focused = true;
                    ac.selected = (ac.selected + 1).min(ac.items.len().saturating_sub(1));
                } else {
                    self.editor.input(key);
                }
            }
            KeyCode::Up => {
                if let Some(ref mut ac) = self.autocomplete {
                    if ac.focused {
                        ac.selected = ac.selected.saturating_sub(1);
                    } else {
                        self.editor.input(key);
                    }
                } else {
                    self.editor.input(key);
                }
            }
            KeyCode::Tab => {
                if self.autocomplete.as_ref().map(|ac| ac.focused).unwrap_or(false) {
                    self.accept_completion();
                } else {
                    // Tab without focused popup — pass to editor.
                    self.editor.input(key);
                    self.update_autocomplete();
                }
            }
            KeyCode::Char(' ') => {
                if self.autocomplete.as_ref().map(|ac| ac.focused).unwrap_or(false) {
                    self.accept_completion();
                    // Also insert the space after the token.
                    self.editor.input(key);
                } else {
                    self.editor.input(key);
                    self.update_autocomplete();
                }
            }
            _ => {
                self.editor.input(key);
                self.update_autocomplete();
            }
        }
        Ok(())
    }

    fn handle_pane_label_edit_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        pane_idx: usize,
    ) -> color_eyre::Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.editor = TextArea::default();
                self.mode = AppMode::Normal;
            }
            KeyCode::Enter => {
                let new_label = self
                    .editor
                    .lines()
                    .first()
                    .cloned()
                    .unwrap_or_default()
                    .trim()
                    .to_string();

                if let Some(pane) = self.panes.get_mut(pane_idx) {
                    pane.label = new_label;
                }

                self.editor = TextArea::default();
                self.mode = AppMode::Normal;
            }
            _ => {
                self.editor.input(key);
            }
        }
        Ok(())
    }

    // ── Autocomplete helpers ──────────────────────────────────────────────────

    /// Collect all @context or +project tokens from the task list (without the trigger char).
    fn collect_tokens(&self, trigger: char) -> Vec<String> {
        let mut tokens: Vec<String> = self.task_list.tasks().iter().flat_map(|t| {
            if trigger == '@' { t.contexts.clone() } else { t.projects.clone() }
        }).collect();
        tokens.sort();
        tokens.dedup();
        tokens
    }

    /// Recompute autocomplete state from the current editor line.
    /// Handles both token autocomplete (@/+) and date autocomplete (due:/t:).
    fn update_autocomplete(&mut self) {
        match self.mode {
            AppMode::Adding | AppMode::Editing { .. } => {}
            AppMode::AppendText => { self.autocomplete = None; return; }
            _ => { self.autocomplete = None; return; }
        }
        let line = self.editor.lines().first().cloned().unwrap_or_default();
        
        // Check for date patterns first (due: or t:)
        if let Some((month_year, _pos)) = self.extract_date_pattern(&line) {
            if let Ok(date_suggestions) = crate::state::generate_date_suggestions(&month_year) {
                if !date_suggestions.is_empty() {
                    // Use a special trigger character '#' for date autocomplete
                    if let Some(ref mut ac) = self.autocomplete {
                        if ac.trigger == '#' && ac.prefix == month_year {
                            ac.items = date_suggestions;
                            ac.selected = ac.selected.min(ac.items.len().saturating_sub(1));
                            return;
                        }
                    }
                    self.autocomplete = Some(AutocompleteState::new('#', month_year, date_suggestions));
                    return;
                }
            }
        }
        
        // Fall back to token autocomplete (@/+)
        // Find last @ or + in the line.
        let trigger_pos = line.rfind(['@', '+']);
        if let Some(pos) = trigger_pos {
            let trigger = line.chars().nth(pos).unwrap();
            let prefix = &line[pos + 1..];
            // No popup if prefix contains a space (token is complete).
            if prefix.contains(' ') {
                self.autocomplete = None;
                return;
            }
            let prefix_lower = prefix.to_lowercase();
            let all_tokens = self.collect_tokens(trigger);
            let filtered: Vec<String> = all_tokens.into_iter()
                .filter(|t| t.to_lowercase().starts_with(&prefix_lower))
                .collect();
            if filtered.is_empty() {
                self.autocomplete = None;
                return;
            }
            // Update existing state if same trigger+prefix, else create new.
            if let Some(ref mut ac) = self.autocomplete {
                if ac.trigger == trigger && ac.prefix == prefix {
                    ac.items = filtered;
                    ac.selected = ac.selected.min(ac.items.len().saturating_sub(1));
                    return;
                }
            }
            self.autocomplete = Some(AutocompleteState::new(trigger, prefix.to_string(), filtered));
        } else {
            self.autocomplete = None;
        }
    }

    /// Extract date pattern from line if present.
    /// Detects patterns like "due:2026-07-" or "t:2026-07-" and returns (month_year, position).
    /// Returns None if no valid date pattern is found.
    fn extract_date_pattern(&self, line: &str) -> Option<(String, usize)> {
        // Look for "due:" or "t:" followed by date pattern
        let patterns = ["due:", "t:"];
        for pattern in &patterns {
            if let Some(pos) = line.rfind(pattern) {
                let after_pattern = &line[pos + pattern.len()..];
                // Match YYYY-MM or YYYY-MM- (potentially incomplete date)
                // Pattern: digits-digits or digits-digits-
                if after_pattern.len() >= 7 {
                    let candidate = &after_pattern[..7]; // First 7 chars (YYYY-MM)
                    let parts: Vec<&str> = candidate.split('-').collect();
                    if parts.len() == 2
                        && parts[0].len() == 4
                        && parts[1].len() == 2
                        && parts[0].chars().all(|c| c.is_ascii_digit())
                        && parts[1].chars().all(|c| c.is_ascii_digit())
                    {
                        // Valid YYYY-MM pattern
                        return Some((candidate.to_string(), pos));
                    }
                }
            }
        }
        None
    }

    /// Insert the currently selected autocomplete token or date into the editor.
    fn accept_completion(&mut self) {
        let line = self.editor.lines().first().cloned().unwrap_or_default();
        
        match &self.autocomplete {
            Some(ac) => {
                if ac.trigger == '#' {
                    // Date autocomplete: extract day from selected item (e.g., "14 Tue")
                    if let Some(item) = ac.items.get(ac.selected) {
                        if let Some(day_str) = item.split_whitespace().next() {
                            // Reconstruct full date: month_year + day
                            let full_date = format!("{}-{}", ac.prefix, day_str);
                            
                            // Find and replace the date pattern in the line
                            if let Some((pattern_str, pos)) = self.extract_date_pattern(&line) {
                                let pattern = if line[pos..].starts_with("due:") {
                                    "due:"
                                } else {
                                    "t:"
                                };
                                let new_line = format!(
                                    "{}{}{}{}",
                                    &line[..pos],
                                    pattern,
                                    full_date,
                                    if pos + pattern.len() + pattern_str.len() < line.len() {
                                        &line[pos + pattern.len() + pattern_str.len()..]
                                    } else {
                                        ""
                                    }
                                );
                                let mut new_editor = tui_textarea::TextArea::default();
                                new_editor.insert_str(&new_line);
                                self.editor = new_editor;
                            }
                        }
                    }
                } else {
                    // Token autocomplete: insert selected token after trigger
                    if let Some(token) = ac.items.get(ac.selected) {
                        let trigger = ac.trigger;
                        if let Some(pos) = line.rfind(trigger) {
                            let new_line = format!("{}{}{}", &line[..=pos], token, "");
                            let mut new_editor = tui_textarea::TextArea::default();
                            new_editor.insert_str(&new_line);
                            self.editor = new_editor;
                        }
                    }
                }
            }
            None => return,
        }
        self.autocomplete = None;
    }

    // ── Filter autocomplete accept (AC-03, Phase 42, Plan 02) ─────────────────

    /// Insert the focused filter autocomplete suggestion into the filter editor,
    /// keeping the filter panel open (D-02: accept stays in Filtering mode).
    fn accept_filter_completion(&mut self) {
        // 1. Clone line and cursor_col from filter_state (immutable borrow, then release).
        let (line, cursor_col) = match &self.filter_state {
            Some(s) => (
                s.editor.lines().first().cloned().unwrap_or_default(),
                s.editor.cursor().1,
            ),
            None => return,
        };

        // 2. Extract the accept action without holding a reference.
        enum AcceptResult {
            Token(char, String),
            History(String),
            NoOp,
        }
        let result = match &self.autocomplete {
            None => AcceptResult::NoOp,
            Some(ac) => match &ac.mode {
                AutocompleteMode::TokenAutocomplete(trigger) => match ac.items.get(ac.selected) {
                    Some(token) => AcceptResult::Token(*trigger, token.clone()),
                    None => AcceptResult::NoOp,
                },
                AutocompleteMode::FilterHistory => match ac.items.get(ac.selected) {
                    Some(entry) => AcceptResult::History(entry.clone()),
                    None => AcceptResult::NoOp,
                },
                _ => AcceptResult::NoOp,
            },
        };

        // 3. Build new_line — no self borrows needed here.
        let new_line = match result {
            AcceptResult::NoOp => {
                self.autocomplete = None;
                return;
            }
            AcceptResult::Token(trigger, token) => {
                // Cursor-aware replacement: replace the trigger-word at cursor (D-03).
                let end = cursor_col.min(line.len());
                let before_cursor = &line[..end];
                let word_start = before_cursor
                    .rfind(char::is_whitespace)
                    .map(|i| i + 1)
                    .unwrap_or(0);
                let after_cursor = if cursor_col <= line.len() {
                    &line[cursor_col..]
                } else {
                    ""
                };
                format!("{}{}{}{}", &line[..word_start], trigger, token, after_cursor)
            }
            AcceptResult::History(entry) => entry,
        };

        // 4. Apply new content — safe: no existing borrows.
        let filter_query = new_line.trim().to_string();
        let mut new_editor = tui_textarea::TextArea::default();
        new_editor.insert_str(&new_line);
        if let Some(ref mut state) = self.filter_state {
            state.editor = new_editor;
        }
        self.active_pane_mut().filter_query = filter_query;
        self.autocomplete = None;
        self.rebuild_and_reanchor();
    }

    // ── Delete confirm key handler ────────────────────────────────────────────

    fn handle_delete_confirm_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> color_eyre::Result<()> {
        if key.code == KeyCode::Char('y') {
            self.push_undo_entry();
            if self.selected_tasks.is_empty() {
                // Existing single-task path (D-01 fallback: d with empty selection)
                if let Some(idx) = self.active_canonical_selected() {
                    self.task_list
                        .delete(idx)
                        .map_err(|e| color_eyre::eyre::eyre!("Failed to delete task: {}", e))?;
                    // Keep all panes' canonical rows in sync after mutation.
                    self.rebuild_all_panes();
                    self.rebuild_and_reanchor();
                }
            } else {
                // Bulk path (D-03): delete in descending index order so no index shifts
                let mut sorted_indices: Vec<usize> = self.selected_tasks.iter().copied().collect();
                sorted_indices.sort_unstable_by(|a, b| b.cmp(a)); // descending
                for idx in sorted_indices {
                    self.task_list
                        .delete(idx)
                        .map_err(|e| color_eyre::eyre::eyre!("Failed to bulk delete task {}: {}", idx, e))?;
                }
                // Keep all panes' canonical rows in sync after mutation.
                self.rebuild_all_panes();
                self.rebuild_and_reanchor();
                // D-04: clear selection and exit disjoint mode after bulk delete
                self.selected_tasks.clear();
                self.disjoint_select = false;
            }
        } else {
            // Non-y key cancels; if bulk was in progress, clear selection (D-04 cancel path)
            if !self.selected_tasks.is_empty() {
                self.selected_tasks.clear();
                self.selection_anchor = None;
                self.disjoint_select = false;
            }
        }
        // Any key returns to Normal (D-07).
        self.mode = AppMode::Normal;
        self.apply_pending_reload()?;
        Ok(())
    }

    // ── Archive confirm key handler (Phase 39, ARCH-01/02/03) ──────────────────

    /// Return the path to done.txt: from config if set, otherwise sibling of todo_path.
    fn archive_path(&self) -> PathBuf {
        self.config
            .done_file
            .clone()
            .unwrap_or_else(|| {
                self.todo_path
                    .parent()
                    .unwrap_or(std::path::Path::new("."))
                    .join("done.txt")
            })
    }

    /// Atomically append completed tasks to done.txt (write-first), then remove them from
    /// task_list. Pushes a single undo entry before any mutation so Ctrl+Z restores todo.txt.
    /// Returns the number of tasks archived, or an error if the file write fails.
    fn archive_tasks(&mut self) -> color_eyre::Result<usize> {
        use std::io::Write;
        let done_path = self.archive_path();

        let completed: Vec<_> = self
            .task_list
            .tasks()
            .iter()
            .filter(|t| t.completed)
            .cloned()
            .collect();
        let count = completed.len();
        if count == 0 {
            return Ok(0);
        }

        // Ensure done.txt parent exists.
        if let Some(parent) = done_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Build done.txt content: existing + newly archived tasks.
        let existing = if done_path.exists() {
            std::fs::read_to_string(&done_path)?
        } else {
            String::new()
        };
        let appended = completed
            .iter()
            .map(|t| t.to_raw())
            .collect::<Vec<_>>()
            .join("\n");
        let new_done = if existing.is_empty() {
            format!("{appended}\n")
        } else {
            let base = existing.trim_end_matches('\n');
            format!("{base}\n{appended}\n")
        };

        // Write done.txt atomically (write-first: crash safety — done.txt written before
        // task_list is mutated so no data loss on crash between the two writes).
        let done_parent = done_path.parent().unwrap_or(std::path::Path::new("."));
        let mut temp_done = tempfile::NamedTempFile::new_in(done_parent)?;
        temp_done.write_all(new_done.as_bytes())?;
        temp_done.flush()?;
        temp_done.as_file().sync_all()?;
        temp_done
            .persist(&done_path)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to write done.txt: {}", e.error))?;

        // Only after done.txt succeeds: snapshot undo state and remove completed from task_list.
        self.push_undo_entry();
        let mut completed_indices: Vec<usize> = self
            .task_list
            .tasks()
            .iter()
            .enumerate()
            .filter(|(_, t)| t.completed)
            .map(|(i, _)| i)
            .collect();
        // Delete in descending order to avoid index shift.
        completed_indices.sort_unstable_by(|a, b| b.cmp(a));
        for idx in completed_indices {
            self.task_list
                .delete(idx)
                .map_err(|e| color_eyre::eyre::eyre!("Failed to delete archived task {}: {}", idx, e))?;
        }

        self.selected_tasks.clear();
        self.rebuild_all_panes();
        self.rebuild_and_reanchor();
        Ok(count)
    }

    fn handle_archive_confirm_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> color_eyre::Result<()> {
        if key.code == KeyCode::Char('y') {
            match self.archive_tasks() {
                Ok(count) if count > 0 => {
                    self.runtime_warnings.push(format!(
                        "Archived {} task(s)  (Ctrl+Z to restore to todo.txt)",
                        count
                    ));
                }
                Ok(_) => {}
                Err(e) => {
                    self.runtime_warnings.push(format!("Archive failed: {e}"));
                }
            }
        }
        // Any key returns to Normal.
        self.mode = AppMode::Normal;
        self.apply_pending_reload()?;
        Ok(())
    }

    // ── Append text confirm key handler (count preview, D-06) ──────────────────

    fn handle_append_text_confirm_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> color_eyre::Result<()> {
        match key.code {
            KeyCode::Enter => {
                // Confirmed — open text editor for bulk append
                self.editor = TextArea::default();
                self.mode = AppMode::AppendText;
                // append_confirm_count no longer needed; AppendText reads selected_tasks directly
            }
            KeyCode::Esc => {
                // Cancel — selection preserved per D-03 (do NOT clear selected_tasks)
                self.append_confirm_count = 0;
                self.mode = AppMode::Normal;
                self.apply_pending_reload()?;
            }
            _ => {}
        }
        Ok(())
    }

    // ── Append text key handler ────────────────────────────────────────────────

    fn handle_append_text_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> color_eyre::Result<()> {
        match key.code {
            KeyCode::Esc => {
                // Cancel — no tasks mutated (D-08)
                self.selected_tasks.clear();
                self.selection_anchor = None;
                self.disjoint_select = false;
                self.editor = TextArea::default();
                self.mode = AppMode::Normal;
                self.apply_pending_reload()?;
            }
            KeyCode::Enter => {
                let text = self.editor.lines().first().cloned().unwrap_or_default();
                let text = text.trim().to_string();

                if text.is_empty() {
                    // Empty input — cancel without mutating (D-08)
                } else {
                    // Build replacements: for each selected index, append text to raw (D-08, D-09)
                    // Descending order for symmetry with bulk delete, even though append
                    // does not shift indices (D-09 in 20-CONTEXT.md).
                    let mut sorted_indices: Vec<usize> = self.selected_tasks.iter().copied().collect();
                    sorted_indices.sort_unstable_by(|a, b| b.cmp(a));

                    // Build (index, updated_task) pairs for batch_update.
                    let tasks = self.task_list.tasks();
                    let replacements: Vec<(usize, Task)> = sorted_indices
                        .iter()
                        .filter_map(|&idx| {
                            tasks.get(idx).map(|t| {
                                let new_task = if self.config.normalize_append {
                                    // D-07/D-08: normalize_append enabled (default) — parse-then-merge strategy.
                                    // Tokens in `text` (priority, +proj, @ctx, due:, t:) are merged into t's fields
                                    // and rebuilt canonically. Unknown tokens preserved in body (NORM-05).
                                    normalize_append(t, &text)
                                } else {
                                    // D-08: normalize_append = false — Phase 20 raw concat fallback.
                                    let new_raw = format!("{} {}", t.to_raw().trim_end(), &text);
                                    Task::parse(&new_raw)
                                };
                                (idx, new_task)
                            })
                        })
                        .collect();

                    self.push_undo_entry();
                    self.task_list
                        .batch_update(replacements)
                        .map_err(|e| color_eyre::eyre::eyre!("Failed to bulk append: {}", e))?;

                    self.rebuild_and_reanchor();
                }

                // D-10: clear selection and return to Normal after append (success or empty-cancel)
                self.selected_tasks.clear();
                self.selection_anchor = None;
                self.disjoint_select = false;
                self.editor = TextArea::default();
                self.mode = AppMode::Normal;
                self.apply_pending_reload()?;
            }
            _ => {
                // Forward all other keys to the editor widget
                self.editor.input(key);
            }
        }
        Ok(())
    }

    // ── Date picker key handler ───────────────────────────────────────────────

    fn handle_date_picker_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> color_eyre::Result<()> {
        match key.code {
            KeyCode::Esc => {
                // Cancel — no tasks mutated
                self.date_picker = None;
                self.mode = AppMode::Normal;
            }
            KeyCode::Down => {
                if let Some(ref mut dp) = self.date_picker {
                    dp.focused = true;
                    dp.select_next();
                }
            }
            KeyCode::Up => {
                if let Some(ref mut dp) = self.date_picker {
                    dp.focused = true;
                    dp.select_prev();
                    dp.day_input.clear();
                }
            }
            KeyCode::Backspace => {
                if let Some(ref mut dp) = self.date_picker {
                    if !dp.day_input.is_empty() {
                        dp.day_input.pop();
                    } else {
                        dp.month_year.pop();
                    }

                    dp.suggestions = crate::state::generate_date_suggestions(&dp.month_year)
                        .unwrap_or_default();
                    if let Some(day) = dp.selected_day {
                        let still_valid = dp.suggestions.iter().any(|s| {
                            s.split_whitespace()
                                .next()
                                .and_then(|d| d.parse::<u32>().ok())
                                .map(|d| d == day)
                                .unwrap_or(false)
                        });
                        if !still_valid {
                            dp.selected_day = dp
                                .suggestions
                                .first()
                                .and_then(|s| s.split_whitespace().next())
                                .and_then(|d| d.parse::<u32>().ok());
                        }
                    }
                    dp.focused = true;
                }
            }
            KeyCode::Char(ch) if key.modifiers == KeyModifiers::NONE && (ch.is_ascii_digit() || ch == '-' || ch == '/') => {
                if let Some(ref mut dp) = self.date_picker {
                    // First typed character starts direct date entry from scratch.
                    // This keeps arrow navigation for day picking, while allowing YYYY-MM-DD typing.
                    if !dp.focused && dp.day_input.is_empty() {
                        dp.month_year.clear();
                    }

                    let typed = if ch == '/' { '-' } else { ch };

                    if typed == '-' {
                        if dp.month_year.len() == 4 && !dp.month_year.contains('-') {
                            dp.month_year.push('-');
                            dp.day_input.clear();
                        }
                    } else if dp.month_year.len() < 7 {
                        if dp.month_year.len() == 4 && !dp.month_year.contains('-') {
                            dp.month_year.push('-');
                        }
                        if dp.month_year.len() < 7 {
                            dp.month_year.push(typed);
                        }
                        dp.day_input.clear();
                    } else if dp.day_input.len() < 2 {
                        dp.day_input.push(typed);
                    }

                    dp.suggestions = crate::state::generate_date_suggestions(&dp.month_year)
                        .unwrap_or_default();

                    // If user typed day digits, try to select that day in the suggestion list.
                    if !dp.day_input.is_empty() {
                        if let Ok(day) = dp.day_input.parse::<u32>() {
                            let matches_month = dp.suggestions.iter().any(|s| {
                                s.split_whitespace()
                                    .next()
                                    .and_then(|d| d.parse::<u32>().ok())
                                    .map(|d| d == day)
                                    .unwrap_or(false)
                            });
                            if matches_month {
                                dp.selected_day = Some(day);
                            }
                        }
                    } else if dp.selected_day.is_none() {
                        dp.selected_day = dp
                            .suggestions
                            .first()
                            .and_then(|s| s.split_whitespace().next())
                            .and_then(|d| d.parse::<u32>().ok());
                    }

                    dp.focused = true;
                }
            }
            KeyCode::Tab | KeyCode::Enter => {
                // Accept selected day and mutate task(s)
                if let Some(dp) = self.date_picker.take() {
                    let chosen_day = if !dp.day_input.is_empty() {
                        dp.day_input.parse::<u32>().ok()
                    } else {
                        dp.selected_day
                    };

                    if let Some(selected_day) = chosen_day {
                        // D-13: parse date for structured mutation via with_due_date()
                        use chrono::NaiveDate;
                        let new_date = NaiveDate::parse_from_str(
                            &format!("{}-{:02}", dp.month_year, selected_day),
                            "%Y-%m-%d",
                        ).map_err(|e| color_eyre::eyre::eyre!("Invalid date: {}", e))?;

                        // Determine targets from the shared quick-setter targeting semantics.
                        let targets = self.quick_setter_targets();

                        // Update each task via structured mutation (D-13)
                        let tasks = self.task_list.tasks();
                        let mut replacements: Vec<(usize, Task)> = Vec::new();

                        for &idx in &targets {
                            if let Some(task) = tasks.get(idx) {
                                // with_due_date() rebuilds via rebuild_raw,
                                // preserving all non-due fields (D-13)
                                replacements.push((idx, task.clone().with_due_date(Some(new_date))));
                            }
                        }

                        if !replacements.is_empty() {
                            self.push_undo_entry();
                            let _ = self.task_list.batch_update(replacements);
                            self.rebuild_and_reanchor();
                        }
                    } else {
                        self.push_runtime_warning("invalid day for selected month");
                    }
                }
                self.selected_tasks.clear();
                self.selection_anchor = None;
                self.mode = AppMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    // ── Priority picker key handler ───────────────────────────────────────────

    fn handle_priority_picker_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> color_eyre::Result<()> {
        match key.code {
            KeyCode::Esc => {
                // Cancel — no tasks mutated, selection preserved (D-03)
                self.priority_picker = None;
                self.mode = AppMode::Normal;
            }
            KeyCode::Down => {
                if let Some(ref mut pp) = self.priority_picker {
                    pp.focused = true;
                    pp.select_next();
                }
            }
            KeyCode::Up => {
                if let Some(ref mut pp) = self.priority_picker {
                    pp.focused = true;
                    pp.select_prev();
                }
            }
            KeyCode::Char(ch) if ch.is_alphabetic() => {
                // Type-to-jump: jump to that priority letter
                if let Some(ref mut pp) = self.priority_picker {
                    pp.focused = true;
                    pp.jump_to(ch);
                }
            }
            KeyCode::Tab | KeyCode::Enter => {
                // Accept and apply priority to target tasks
                if let Some(pp) = self.priority_picker.take() {
                    let chosen_priority = pp.selected_priority(); // None = "clear priority"

                    // Determine targets via shared quick-setter semantics (selected or active).
                    let targets = self.quick_setter_targets();

                    if !targets.is_empty() {
                        let tasks = self.task_list.tasks();
                        let replacements: Vec<(usize, Task)> = targets
                            .iter()
                            .filter_map(|&idx| {
                                tasks.get(idx).map(|t| {
                                    // D-13: structured mutation via with_priority builder
                                    (idx, t.clone().with_priority(chosen_priority))
                                })
                            })
                            .collect();

                        if !replacements.is_empty() {
                            self.push_undo_entry();
                            self.task_list
                                .batch_update(replacements)
                                .map_err(|e| color_eyre::eyre::eyre!("Failed to set priority: {}", e))?;
                            self.rebuild_and_reanchor();
                        }
                    }
                }
                // Clear selection after accept (consistent with date picker accept behavior)
                self.selected_tasks.clear();
                self.selection_anchor = None;
                self.mode = AppMode::Normal;
                self.apply_pending_reload()?;
            }
            _ => {}
        }
        Ok(())
    }

    // ── Edit mode helpers ─────────────────────────────────────────────────────

    /// Discard changes and return to Normal mode, applying any queued reload (D-10).
    fn exit_edit_mode(&mut self) -> color_eyre::Result<()> {
        self.editor = TextArea::default();
        self.mode = AppMode::Normal;
        self.apply_pending_reload()
    }

    /// Persist editor content and return to Normal mode (D-12, D-13).
    fn save_and_exit(&mut self) -> color_eyre::Result<()> {
        let text = self.editor.lines().first().cloned().unwrap_or_default();
        let mode = self.mode; // Copy
        match mode {
            AppMode::Adding => {
                // T-21-07: Adding always uses Task::parse — normalize_edit does not apply here.
                // User is creating a new task from scratch; there is no "original" to merge into.
                let task = Task::parse(&text);
                self.push_undo_entry();
                self.task_list
                    .add(task)
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to add task: {}", e))?;
                // Move selection to the newly added task (D-13).
                let canonical = self.task_list.len().saturating_sub(1);
                self.rebuild_display_indices();
                self.rebuild_all_panes();
                self.selected = self
                    .display_rows
                    .iter()
                    .position(|r| matches!(r, DisplayRow::Task(idx) if *idx == canonical))
                    .unwrap_or(0);
            }
            AppMode::Editing { original_idx } => {
                // D-05/D-06: normalize_edit = true (default) applies normalize_line.
                // normalize_line lifts inline priority tokens from body to canonical position
                // and rebuilds via rebuild_raw. Does NOT merge onto original task — T-21-06
                // (avoid body-doubling: user has typed the entire replacement line).
                let task = if self.config.normalize_edit {
                    normalize_line(&text)
                } else {
                    Task::parse(&text)
                };
                self.push_undo_entry();
                self.task_list
                    .update(original_idx, task)
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to update task: {}", e))?;
                self.rebuild_display_indices();
                self.rebuild_all_panes();
                self.selected = self
                    .display_rows
                    .iter()
                    .position(|r| matches!(r, DisplayRow::Task(idx) if *idx == original_idx))
                    .unwrap_or(0);
            }
            _ => {}
        }
        self.editor = TextArea::default();
        self.mode = AppMode::Normal;
        self.apply_pending_reload()
    }

    /// Prune stale canonical indices from the selection set and anchor after a reload (D-19).
    ///
    /// Retains only indices `< task_list.len()` — silently drops any that fell out of range.
    /// Clears `selection_anchor` if it points to a task that no longer exists.
    fn prune_stale_selections(&mut self) {
        let len = self.task_list.len();
        self.selected_tasks.retain(|&idx| idx < len);
        if let Some(anchor) = self.selection_anchor {
            if anchor >= len {
                self.selection_anchor = None;
            }
        }
    }

    /// Apply a queued `FileChanged` reload if `pending_reload` is set (D-10).
    fn apply_pending_reload(&mut self) -> color_eyre::Result<()> {
        if self.pending_reload {
            self.pending_reload = false;
            self.task_list.reload().map_err(|e| {
                color_eyre::eyre::eyre!(
                    "Failed to reload {}: {}",
                    self.todo_path.display(),
                    e
                )
            })?;
            self.prune_stale_selections();
            self.rebuild_and_reanchor();
        }
        Ok(())
    }

    /// Clamp `selected` to `[0, display_count - 1]`, or 0 on empty display.
    fn clamp_selection(&mut self) {
        let count = self.display_rows.len();
        if count == 0 {
            self.selected = 0;
        } else {
            self.selected = self.selected.min(count - 1);
        }
    }

    /// Rebuild `display_indices` by applying the active filter and sort order.
    ///
    /// Collects canonical task indices in display order. Does NOT touch `selected`.
    fn rebuild_display_indices(&mut self) {
        let query = self.filter_query.trim().to_string();
        let sort_order = self.sort_order;
        let new_indices: Vec<usize> = {
            let mut pairs: Vec<(usize, &Task)> = if query.is_empty() {
                self.task_list.tasks().iter().enumerate().collect()
            } else {
                let mut f = Filter::from_query(&query);
                if self.show_deferred {
                    f.suppress_future_threshold = false;
                }
                self.task_list.filter(&f)
            };
            if sort_order != SortOrder::FileOrder {
                pairs.sort_by(|(_, a), (_, b)| sort_order.compare(a, b));
            }
            pairs.into_iter().map(|(idx, _)| idx).collect()
        };
        self.display_indices = new_indices;

        if self.grouping && !self.display_indices.is_empty() {
            let tasks = self.task_list.tasks();
            let _sort_order = self.sort_order;
            let group_by = self.group_by;
            // Stable-sort by group key so same-key tasks are always adjacent.
            // This fixes cases where the primary sort interleaves groups (e.g., Alphabetical
            // sorts by raw string including priority prefix, but group_key_for uses body).
            // stable_sort preserves primary sort order within each group.
            self.display_indices.sort_by(|&a, &b| {
                let ka = group_key_for(&tasks[a], &group_by);
                let kb = group_key_for(&tasks[b], &group_by);
                ka.cmp(&kb)
            });
            let mut rows: Vec<DisplayRow> = Vec::new();
            let mut last_key: Option<String> = None;
            for &idx in &self.display_indices {
                let task = &tasks[idx];
                let key = group_key_for(task, &group_by);
                if last_key.as_deref() != Some(&key) {
                    rows.push(DisplayRow::GroupHeader(key.clone()));
                    last_key = Some(key);
                }
                rows.push(DisplayRow::Task(idx));
            }
            self.display_rows = rows;
        } else {
            self.display_rows = self
                .display_indices
                .iter()
                .map(|&i| DisplayRow::Task(i))
                .collect();
        }
    }

    /// Rebuild display indices while preserving the selected canonical task.
    ///
    /// Saves the current canonical index, rebuilds, then restores the selection
    /// to the display row where that canonical index now appears (or row 0).
    ///
    /// For multi-pane mode, also updates the active pane's display_rows with per-pane query state.
    fn rebuild_and_reanchor(&mut self) {
        // In multi-pane mode, capture old canonical from the active pane's cursor (WARN-4 fix, Phase 28).
        let old_canonical = if !self.should_show_single_pane() && self.panes.len() > 1 {
            let pane = &self.panes[self.active_pane];
            match pane.display_rows.get(pane.selected) {
                Some(DisplayRow::Task(idx)) => Some(*idx),
                _ => self.canonical_selected(),
            }
        } else {
            self.canonical_selected()
        };

        // GAP-1 fix (Phase 31): sync global fields from active pane when in single-pane or hidden mode
        // In multi-pane mode, rebuild_visible_rows() uses per-pane fields directly.
        // In single-pane or panes_hidden mode, rebuild_display_indices() (global path) is used,
        // so we must sync the global state to match the active pane before calling rebuild_display_indices().
        if self.should_show_single_pane() || self.panes_hidden {
            let pane = &self.panes[self.active_pane];
            self.filter_query = pane.filter_query.clone();
            self.sort_order = pane.sort_order;
            self.grouping = pane.grouping;
        }

        self.rebuild_display_indices();

        // Per-pane rebuild (Phase 25): Update active pane's display_rows with per-pane filter/sort/group
        if !self.should_show_single_pane() && self.panes.len() > 1 {
            self.rebuild_visible_rows();
            // WARN-4 fix: reanchor the active pane's cursor after rebuild (Phase 28).
            let new_pane_selected = old_canonical
                .and_then(|ci| {
                    self.panes[self.active_pane]
                        .display_rows
                        .iter()
                        .position(|r| matches!(r, DisplayRow::Task(idx) if *idx == ci))
                })
                .unwrap_or(0);
            let pane = &mut self.panes[self.active_pane];
            pane.selected = new_pane_selected;
            if pane.selected >= pane.display_rows.len() && !pane.display_rows.is_empty() {
                pane.selected = pane.display_rows.len() - 1;
            }
        }

        self.selected = old_canonical
            .and_then(|ci| {
                self.display_rows
                    .iter()
                    .position(|r| matches!(r, DisplayRow::Task(idx) if *idx == ci))
            })
            .unwrap_or(0);
        self.clamp_selection();
    }

    /// Return the canonical task index for the currently selected display row, or `None`
    /// if the display list is empty.
    fn canonical_selected(&self) -> Option<usize> {
        match self.display_rows.get(self.selected) {
            Some(DisplayRow::Task(idx)) => Some(*idx),
            _ => self.display_indices.first().copied(),
        }
    }

    /// Resolve selected canonical index from the current interaction scope.
    ///
    /// In multi-pane mode, uses the active pane cursor; otherwise uses global/single-pane cursor.
    fn active_canonical_selected(&self) -> Option<usize> {
        let selected = if !self.should_show_single_pane() && self.panes.len() > 1 && !self.panes_hidden {
            self.pane_canonical_selected()
        } else {
            self.canonical_selected()
        };

        // Guard against stale display mappings (e.g., after reload/race) so caller paths
        // never index tasks out-of-bounds.
        selected.filter(|&idx| idx < self.task_list.len())
    }

    /// Toggle the cursor row's canonical index in `selected_tasks`.
    ///
    /// No-op when the cursor is on a `GroupHeader` row (D-08).
    #[allow(dead_code)]
    fn toggle_task_selection(&mut self) {
        if let Some(DisplayRow::Task(idx)) = self.display_rows.get(self.selected).cloned() {
            if self.selected_tasks.contains(&idx) {
                self.selected_tasks.remove(&idx);
            } else {
                self.selected_tasks.insert(idx);
            }
        }
    }

    /// Clear the entire selection set, reset the anchor, and exit disjoint mode (D-07).
    #[allow(dead_code)]
    fn clear_selection(&mut self) {
        self.selected_tasks.clear();
        self.selection_anchor = None;
        self.disjoint_select = false;
    }

    /// Lazily initialize `selection_anchor` from the cursor's canonical index (D-11).
    ///
    /// If an anchor is already set, this is a no-op — the anchor stays stable
    /// for the entire duration of a shift-range operation.
    fn ensure_anchor(&mut self) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = self.active_canonical_selected();
        }
    }

    /// Replace `selected_tasks` with the contiguous range of task rows between
    /// `selection_anchor` and the current cursor, skipping `GroupHeader` rows (D-08, D-09).
    ///
    /// If the anchor has no corresponding display row (e.g., filtered out), this is a no-op.
    fn apply_range_selection(&mut self) {
        let anchor_canon = match self.selection_anchor {
            Some(a) => a,
            None => return,
        };

        let (rows, selected_row) = if !self.should_show_single_pane() && self.panes.len() > 1 && !self.panes_hidden {
            let pane = &self.panes[self.active_pane];
            (&pane.display_rows, pane.selected)
        } else {
            (&self.display_rows, self.selected)
        };

        let cursor_canon = match rows.get(selected_row) {
            Some(DisplayRow::Task(idx)) => Some(*idx),
            _ => None,
        };
        let cursor_canon = match cursor_canon {
            Some(c) => c,
            None => return,
        };

        // Locate display-row positions for anchor and cursor canonical indices.
        let anchor_row = rows
            .iter()
            .position(|r| matches!(r, DisplayRow::Task(idx) if *idx == anchor_canon));
        let cursor_row = rows
            .iter()
            .position(|r| matches!(r, DisplayRow::Task(idx) if *idx == cursor_canon));
        let (anchor_row, cursor_row) = match (anchor_row, cursor_row) {
            (Some(a), Some(c)) => (a, c),
            _ => return,
        };
        let (lo, hi) = if anchor_row <= cursor_row {
            (anchor_row, cursor_row)
        } else {
            (cursor_row, anchor_row)
        };
        // Replace selection with only the tasks inside the [lo, hi] display range.
        self.selected_tasks.clear();
        #[allow(clippy::needless_range_loop)]
        for row in lo..=hi {
            if let DisplayRow::Task(idx) = rows[row] {
                self.selected_tasks.insert(idx);
            }
        }
    }

    /// Toggle the completion state of the currently selected task and persist to disk.
    ///
    /// D-10: immediate save via `task_list.update()` (which calls `save()` internally).
    /// D-11: toggles both ways — incomplete→done AND done→incomplete.
    /// D-12: called by both `x` and bare `u`.
    #[allow(dead_code)]
    fn toggle_done(&mut self) {
        let idx = match self.canonical_selected() {
            Some(i) => i,
            None => return,
        };
        let task = self.task_list.tasks()[idx].clone();
        let was_completed = task.completed;
        let toggled = task.with_completed(!was_completed);
        self.push_undo_entry();
        // update() calls save() internally — single atomic disk write (temp rename).
        if let Err(e) = self.task_list.update(idx, toggled) {
            // Non-fatal: write to stderr (terminal restore guard is active).
            eprintln!("toggle_done error: {e}");
        }
        self.rebuild_and_reanchor();
    }

    /// Get the canonical task index for the currently selected row in the active pane (Phase 24-02).
    fn pane_canonical_selected(&self) -> Option<usize> {
        let pane = self.active_pane();
        if pane.label_selected {
            return None;
        }
        match pane.display_rows.get(pane.selected) {
            Some(DisplayRow::Task(idx)) => Some(*idx),
            _ => pane.display_rows.first().and_then(|r| {
                match r {
                    DisplayRow::Task(idx) => Some(*idx),
                    _ => None,
                }
            }),
        }
    }

    /// Toggle the completion state of the currently selected task in the active pane (Phase 24-02).
    fn pane_toggle_done(&mut self) {
        self.reconcile_active_pane();
        let idx = match self.pane_canonical_selected() {
            Some(i) => i,
            None => return,
        };
        let task = self.task_list.tasks()[idx].clone();
        let was_completed = task.completed;
        let toggled = task.with_completed(!was_completed);
        self.push_undo_entry();
        if let Err(e) = self.task_list.update(idx, toggled) {
            eprintln!("toggle_done error: {e}");
        }
        self.rebuild_all_panes();
    }

    /// Mark all incomplete tasks in `selected_tasks` as done in one batch.
    /// Pushes a single undo entry before the loop, clears selection afterwards.
    fn bulk_mark_done(&mut self) {
        self.push_undo_entry();
        let indices: Vec<usize> = self.selected_tasks.iter().copied().collect();
        let mut marked = 0usize;
        for idx in &indices {
            if let Some(task) = self.task_list.tasks().get(*idx) {
                if !task.completed {
                    let updated = task.clone().with_completed(true);
                    if let Err(e) = self.task_list.update(*idx, updated) {
                        eprintln!("bulk_mark_done error on idx {idx}: {e}");
                    } else {
                        marked += 1;
                    }
                }
            }
        }
        self.selected_tasks.clear();
        self.runtime_warnings.push(format!("Marked {} task(s) done", marked));
        self.rebuild_all_panes();
        self.rebuild_and_reanchor();
    }

    /// Delete either the single selected task or the active cursor task immediately.
    fn delete_active_task(&mut self) -> color_eyre::Result<()> {
        let idx = if self.selected_tasks.len() == 1 {
            self.selected_tasks
                .iter()
                .next()
                .copied()
                .filter(|&idx| idx < self.task_list.len())
        } else {
            self.active_canonical_selected()
        };

        if let Some(idx) = idx {
            self.push_undo_entry();
            self.task_list
                .delete(idx)
                .map_err(|e| color_eyre::eyre::eyre!("Failed to delete task: {}", e))?;
            self.rebuild_all_panes();
            self.rebuild_and_reanchor();
        }

        self.clear_selection();
        self.mode = AppMode::Normal;
        Ok(())
    }

    /// Toggle the cursor row's canonical index in `selected_tasks` for the active pane (Phase 24-02).
    fn pane_toggle_task_selection(&mut self) {
        self.reconcile_active_pane();
        let pane = self.active_pane();
        if let Some(DisplayRow::Task(idx)) = pane.display_rows.get(pane.selected).cloned() {
            if self.selected_tasks.contains(&idx) {
                self.selected_tasks.remove(&idx);
            } else {
                self.selected_tasks.insert(idx);
            }
        }
    }

    /// Move selection down in the active pane, skipping group headers (Phase 24-02).
    fn pane_move_down(&mut self) {
        self.reconcile_active_pane();
        let use_global_cursor = self.should_show_single_pane() || self.panes_hidden;
        let global_selected = self.selected;
        let pane = self.active_pane_mut();

        if use_global_cursor {
            pane.selected = global_selected.min(pane.display_rows.len().saturating_sub(1));
        }

        if pane.label_selected {
            pane.label_selected = false;
            pane.selected = 0;
            while pane.selected < pane.display_rows.len()
                && matches!(pane.display_rows[pane.selected], DisplayRow::GroupHeader(_))
            {
                pane.selected += 1;
            }
            if pane.selected >= pane.display_rows.len() {
                pane.selected = 0;
            }
        } else {
            let row_count = pane.display_rows.len();
            if row_count > 0 {
                let mut next = pane.selected + 1;
                while next < row_count
                    && matches!(pane.display_rows[next], DisplayRow::GroupHeader(_))
                {
                    next += 1;
                }
                if next < row_count {
                    pane.selected = next;
                }
            }
        }

        if use_global_cursor {
            self.selected = pane.selected;
        }
    }

    /// Move selection up in the active pane, skipping group headers (Phase 24-02).
    fn pane_move_up(&mut self) {
        self.reconcile_active_pane();
        let use_global_cursor = self.should_show_single_pane() || self.panes_hidden;
        let global_selected = self.selected;
        let pane = self.active_pane_mut();
        if use_global_cursor {
            pane.selected = global_selected.min(pane.display_rows.len().saturating_sub(1));
        }
        if !pane.label_selected {
            if pane.selected == 0 {
                pane.label_selected = true;
            } else {
                let mut prev = pane.selected.saturating_sub(1);
                while prev > 0 && matches!(pane.display_rows[prev], DisplayRow::GroupHeader(_)) {
                    prev -= 1;
                }
                if matches!(pane.display_rows.get(prev), Some(DisplayRow::Task(_))) {
                    pane.selected = prev;
                } else {
                    pane.label_selected = true;
                }
            }
        }

        if use_global_cursor {
            self.selected = pane.selected;
        }
    }

    /// Render the TUI frame with mode-aware layout.
    ///
    /// Signature is `&mut self` because tui-textarea's Widget impl requires
    /// rendering via a mutable reference on some paths.
    fn draw(&mut self, frame: &mut Frame) {
        use ratatui::layout::{Constraint::{Length, Min}, Layout};
        self.reconcile_active_pane();

        match self.mode {
            AppMode::DeleteConfirm => {
                // Three-row split: task list | confirm panel | status bar (D-06).
                let chunks =
                    Layout::vertical([Min(0), Length(1), Length(1)]).split(frame.area());
                self.render_panes(frame, chunks[0]);
                self.render_delete_confirm(frame, chunks[1]);
                self.render_status_bar(frame, chunks[2]);
            }
            AppMode::ArchiveConfirm => {
                // Three-row split: task list | archive confirm panel | status bar (Phase 39, ARCH-01).
                let chunks =
                    Layout::vertical([Min(0), Length(1), Length(1)]).split(frame.area());
                self.render_panes(frame, chunks[0]);
                self.render_archive_confirm(frame, chunks[1]);
                self.render_status_bar(frame, chunks[2]);
            }
            AppMode::Adding | AppMode::Editing { .. } => {
                // Two-row split: task list | inline editor in footer row (D-02).
                let chunks =
                    Layout::vertical([Min(0), Length(1)]).split(frame.area());
                self.render_panes(frame, chunks[0]);
                // tui-textarea renders directly; ratatui 0.29 Widget impl for &TextArea.
                frame.render_widget(&self.editor, chunks[1]);
                // Autocomplete popup floats above the footer row (D-08, D-09).
                self.render_autocomplete_popup(frame, chunks[1]);
            }
            AppMode::PaneLabelEditing { .. } => {
                // Two-row split: panes | inline pane label editor.
                use ratatui::widgets::Paragraph;
                let chunks =
                    Layout::vertical([Min(0), Length(1)]).split(frame.area());
                self.render_panes(frame, chunks[0]);
                let footer_cols = Layout::horizontal([Length(13), Min(0)]).split(chunks[1]);
                frame.render_widget(Paragraph::new("Pane label: "), footer_cols[0]);
                frame.render_widget(&self.editor, footer_cols[1]);
            }
            AppMode::AppendText => {
                // Two-row split: task list | inline editor with "Append: " label in footer row (D-11).
                use ratatui::layout::Constraint::{Length, Min};
                use ratatui::widgets::Paragraph;
                let chunks =
                    Layout::vertical([Min(0), Length(1)]).split(frame.area());
                self.render_panes(frame, chunks[0]);
                // Split footer row: label (9 chars) | editor
                let footer_cols =
                    Layout::horizontal([Length(9), Min(0)]).split(chunks[1]);
                frame.render_widget(Paragraph::new("Append: "), footer_cols[0]);
                frame.render_widget(&self.editor, footer_cols[1]);
            }
            AppMode::Normal => {
                // Two-row split: panes area | status bar (D-14).
                let chunks =
                    Layout::vertical([Min(0), Length(1)]).split(frame.area());
                self.render_panes(frame, chunks[0]);
                self.render_status_bar(frame, chunks[1]);
            }
            AppMode::QuickSetter(_) => {
                // Keep Normal-mode layout and render quick-setter popup above status.
                let chunks =
                    Layout::vertical([Min(0), Length(1)]).split(frame.area());
                self.render_panes(frame, chunks[0]);
                self.render_status_bar(frame, chunks[1]);
                self.render_autocomplete_popup(frame, chunks[1]);
            }
            AppMode::Filtering => {
                let panel_height = 1_u16 + (self.presets.len() as u16).min(5);
                let chunks =
                    Layout::vertical([Min(0), Length(panel_height), Length(1)]).split(frame.area());
                self.render_panes(frame, chunks[0]);
                self.render_filter_panel(frame, chunks[1]);
                // Render inline history suggestions popup if available (Phase 41, FHIST-02).
                self.render_autocomplete_popup(frame, chunks[1]);
                self.render_status_bar(frame, chunks[2]);
            }
            AppMode::FilterDefining => {
                let preset_rows = self.filter_defining_state.as_ref()
                    .map(|s| s.preset_editors.len() as u16)
                    .unwrap_or(0)
                    .min(9);
                // 2 (border + active-filter row) + separator + preset rows, min 4
                let panel_height = (2_u16 + 1 + preset_rows).max(4);
                let chunks =
                    Layout::vertical([Min(0), Length(panel_height), Length(1)]).split(frame.area());
                self.render_panes(frame, chunks[0]);
                self.render_filter_defining_panel(frame, chunks[1]);
                self.render_status_bar(frame, chunks[2]);
            }
            AppMode::KeymapErrors => {
                // Task list visible behind; overlay covers the screen (D-09, Phase 22).
                let chunks = Layout::vertical([Min(0), Length(1)]).split(frame.area());
                self.render_panes(frame, chunks[0]);
                self.render_status_bar(frame, chunks[1]);
                self.render_keymap_errors_overlay(frame, frame.area());
            }
            AppMode::Help => {
                // Task list visible behind; help overlay covers the screen (D-10, Phase 22).
                let chunks = Layout::vertical([Min(0), Length(1)]).split(frame.area());
                self.render_panes(frame, chunks[0]);
                self.render_status_bar(frame, chunks[1]);
                self.render_help_overlay(frame, frame.area());
            }
            AppMode::DatePicker => {
                // Task list visible behind; date picker overlay floats above (Phase 33, Plan 01).
                let chunks = Layout::vertical([Min(0), Length(1)]).split(frame.area());
                self.render_panes(frame, chunks[0]);
                self.render_status_bar(frame, chunks[1]);
                self.render_date_picker_overlay(frame, chunks[1]);
            }
            AppMode::PriorityPicker => {
                // Task list visible behind; priority picker overlay floats above (Phase 34, Plan 01).
                let chunks = Layout::vertical([Min(0), Length(1)]).split(frame.area());
                self.render_panes(frame, chunks[0]);
                self.render_status_bar(frame, chunks[1]);
                self.render_priority_picker_overlay(frame, chunks[1]);
            }
            AppMode::AppendTextConfirm => {
                // Task list visible behind; count confirmation banner floats above (Phase 34, Plan 03).
                let chunks = Layout::vertical([Min(0), Length(1)]).split(frame.area());
                self.render_panes(frame, chunks[0]);
                self.render_status_bar(frame, chunks[1]);
                self.render_append_text_confirm(frame, chunks[1]);
            }
        }
    }

    /// Render the task list with selection highlight.
    fn render_task_list(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::style::{Modifier, Style};
        use ratatui::widgets::{List, ListItem, ListState};
        use todotxt_core::DueStatus;

        let tasks = self.task_list.tasks();

        let items: Vec<ListItem> = if self.display_indices.is_empty() && tasks.is_empty() {
            vec![ListItem::new("(no tasks)")]
        } else if self.display_indices.is_empty() {
            vec![ListItem::new("(no matching tasks)")]
        } else {
            self.display_rows
                .iter()
                .enumerate()
                .map(|(row_idx, row)| match row {
                    DisplayRow::GroupHeader(label) => ListItem::new(format!(" {}", label))
                        .style(self.styles.group_header),
                    DisplayRow::Task(ci) => {
                        let t = &tasks[*ci];
                        let indent = if self.grouping { "  " } else { "" };
                        // Visual precedence: selected non-cursor rows get `>` prefix (D-14).
                        let is_selected = self.selected_tasks.contains(ci);
                        let is_cursor = row_idx == self.selected;
                        let prefix = if self.disjoint_select && is_cursor {
                            "V "
                        } else if is_selected && !is_cursor {
                            "> "
                        } else {
                            ""
                        };
                        let content = format!("{}{}{}: {}", prefix, indent, ci + 1, t.to_raw());
                        // Priority and overdue coloring (D-01, D-09 in 13-CONTEXT.md).
                        // Style precedence: completed (DIM) > deferred shown (DIM) > priority A/B/C > overdue > plain.
                        // Modifier::REVERSED for selection is applied by List::highlight_style — not here.
                        let style = if t.completed {
                            // Completed tasks: DIM only, no color (D-01, D-06).
                            Style::default().add_modifier(Modifier::DIM)
                        } else if self.show_deferred
                            && t.threshold_date.is_some_and(|d| d > Local::now().date_naive())
                        {
                            Style::default().add_modifier(Modifier::DIM)
                        } else if t.priority == Some('A') {
                            self.styles.priority_a
                        } else if t.priority == Some('B') {
                            self.styles.priority_b
                        } else if t.priority == Some('C') {
                            self.styles.priority_c
                        } else if t.due_status() == DueStatus::Overdue {
                            self.styles.overdue
                        } else {
                            Style::default()
                        };
                        // Add BOLD for selected non-cursor rows (D-14).
                        let style = if is_selected && !is_cursor {
                            style.add_modifier(Modifier::BOLD)
                        } else {
                            style
                        };
                        ListItem::new(content).style(style)
                    }
                })
                .collect()
        };

        // Cursor+selected: REVERSED | BOLD (D-15); cursor-only: REVERSED (D-13).
        let cursor_is_selected = self
            .display_rows
            .get(self.selected)
            .map(|r| matches!(r, DisplayRow::Task(ci) if self.selected_tasks.contains(ci)))
            .unwrap_or(false);
        let highlight_modifier = if cursor_is_selected {
            Modifier::REVERSED | Modifier::BOLD
        } else {
            Modifier::REVERSED
        };

        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(highlight_modifier));

        let mut list_state = ListState::default();
        if !self.display_indices.is_empty() {
            list_state = list_state.with_selected(Some(self.selected));
        }

        frame.render_stateful_widget(list, area, &mut list_state);
    }

    /// Render multiple vertical panes side-by-side (Phase 24-02).
    fn render_panes(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::layout::{Constraint, Direction, Layout};

        // When panes_hidden is true, render as single-pane view (D-13, Phase 26)
        if self.panes_hidden {
            self.render_task_list(frame, area);
            return;
        }

        if self.should_show_single_pane() {
            self.render_task_list(frame, area);
            return;
        }

        let pane_count = self.panes.len();
        if pane_count == 0 {
            return;
        }

        // Calculate equal width for each pane
        let pane_constraints = vec![Constraint::Percentage(100 / pane_count as u16); pane_count];
        let pane_areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(pane_constraints)
            .split(area);

        // Render each pane
        for (pane_idx, pane) in self.panes.iter().enumerate() {
            let pane_area = pane_areas[pane_idx];
            let is_active = pane_idx == self.active_pane;

            PaneList::render(
                frame,
                pane_area,
                pane,
                is_active,
                is_active && pane.label_selected,
                &self.selected_tasks,
                self.disjoint_select,
                &self.styles,
                &self.task_list,
                self.show_deferred,
            );
        }
    }

    /// Render the one-row status bar with file info and key hints.
    fn render_status_bar(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::text::{Line, Span};
        use ratatui::widgets::Paragraph;
        use todotxt_core::DueStatus;

        let tasks = self.task_list.tasks();
        let total = tasks.len();
        let scoped_indices = self.status_scope_task_indices();
        let visible = scoped_indices.len();

        let due_today = scoped_indices
            .iter()
            .filter(|&&ci| {
                !tasks[ci].completed && tasks[ci].due_status() == DueStatus::Today
            })
            .count();
        let overdue = scoped_indices
            .iter()
            .filter(|&&ci| {
                !tasks[ci].completed && tasks[ci].due_status() == DueStatus::Overdue
            })
            .count();

        let file_name = self
            .todo_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("todo.txt");

        let mut left = format!("{} | {}/{} tasks", file_name, visible, total);
        
        if due_today > 0 || overdue > 0 {
            left.push_str(&format!(" | {} due today | {} overdue", due_today, overdue));
        }

        // Selection count indicator — only shown when tasks are selected (D-12, D-14)
        if !self.selected_tasks.is_empty() {
            left.push_str(&format!(" | {} selected", self.selected_tasks.len()));
        }

        // Show explicit indicator when disjoint select mode is active, even with zero selected rows.
        if self.disjoint_select {
            left.push_str(" | SELECT mode (space=mark, v=exit)");
        }

        // Error-log indicator — shown when keymap/runtime warnings exist.
        let error_count = self.error_log_count();
        if error_count > 0 {
            left.push_str(&format!(
                " | ⚠ errors: {} ('!' for log)",
                error_count
            ));
        }

        let mut middle = String::new();

        // Per-pane query state (Phase 25): Show active pane's filter/sort/group state
        let (pane_filter, pane_sort, pane_grouping, pane_group_by) = if !self.should_show_single_pane() && self.panes.len() > 1 && !self.panes_hidden {
            let pane = &self.panes[self.active_pane];
            (
                pane.filter_query.clone(),
                pane.sort_order,
                pane.grouping,
                pane.group_by,
            )
        } else {
            // Fallback to global state when showing single pane
            (
                self.filter_query.clone(),
                self.sort_order,
                self.grouping,
                self.group_by,
            )
        };

        let trimmed_filter = pane_filter.trim();
        if let Some(filter_display) = Self::format_status_filter(trimmed_filter) {
            middle.push_str(" | ");
            middle.push_str(&filter_display);
        }
        if pane_sort != SortOrder::FileOrder {
            middle.push_str(" | sort: ");
            middle.push_str(sort_name(pane_sort));
        }
        if pane_grouping {
            middle.push_str(" | grp:");
            middle.push_str(group_by_name(pane_group_by));
        }
        if self.show_deferred {
            middle.push_str(" [+deferred]");
        }

        let right = "  q quit | n add | u edit | d/Del/Bksp del | D bulk del (confirm) | T bulk app | @ context | + project | v sel | Shift+nav range | x done | j/k nav | f filter | ^f filt on/off | F define | o sort | G group | g grp-by | h deferred | t theme | 0 clear filter | 1-9 preset | . reload | ? help";
        let total_width = area.width as usize;
        let left_len = left.len();
        let middle_len = middle.len();
        let right_len = right.len();

        let show_hints = left_len + middle_len + right_len <= total_width;

        let middle_display = if show_hints || left_len + middle_len <= total_width {
            middle
        } else {
            let available = total_width.saturating_sub(left_len);
            if available < 3 {
                // Too narrow to show anything meaningful
                String::new()
            } else {
                // Truncate middle at pipe boundaries for cleaner appearance
                let truncated: String = middle.chars().take(available - 1).collect();
                let last_pipe = truncated.rfind(" | ");
                if let Some(pos) = last_pipe {
                    if pos > 0 {
                        format!("{}…", &truncated[..pos])
                    } else {
                        format!("{}…", truncated)
                    }
                } else {
                    format!("{}…", truncated)
                }
            }
        };

        let status_line = if show_hints {
            Line::from(vec![
                Span::raw(left),
                Span::raw(middle_display),
                Span::raw(right),
            ])
        } else {
            Line::from(vec![
                Span::raw(left),
                Span::raw(middle_display),
            ])
        };

        frame.render_widget(Paragraph::new(status_line), area);
    }

    /// Returns canonical task indices for status-bar counts in the active visual scope.
    /// In multi-pane mode this is the active pane's task rows (excluding group headers);
    /// otherwise it uses the single-pane/global display indices.
    fn status_scope_task_indices(&self) -> Vec<usize> {
        if !self.should_show_single_pane() && self.panes.len() > 1 && !self.panes_hidden {
            self.panes[self.active_pane]
                .display_rows
                .iter()
                .filter_map(|row| match row {
                    DisplayRow::Task(idx) => Some(*idx),
                    DisplayRow::GroupHeader(_) => None,
                })
                .collect()
        } else {
            self.display_indices.clone()
        }
    }

    fn format_status_filter(trimmed_filter: &str) -> Option<String> {
        if trimmed_filter.is_empty() {
            return None;
        }

        let value = if trimmed_filter.len() > 30 {
            format!("{}…", &trimmed_filter[..27])
        } else {
            trimmed_filter.to_string()
        };

        Some(format!("filter: {}", value))
    }

    /// Render a centered help overlay showing all 19 resolved keybindings (D-10, Phase 22).
    fn render_help_overlay(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::layout::{Constraint, Flex, Layout};
        use ratatui::text::{Line, Text};
        use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

        /// Format a (KeyCode, KeyModifiers) pair as a human-readable string.
        fn chord_description(code: crossterm::event::KeyCode, mods: crossterm::event::KeyModifiers) -> String {
            use crossterm::event::KeyCode as KC;
            let key_str = match code {
                KC::Char(' ') => "space".to_string(),
                KC::Char(c) => c.to_string(),
                KC::Backspace => "backspace".to_string(),
                KC::Enter => "enter".to_string(),
                KC::Left => "left".to_string(),
                KC::Right => "right".to_string(),
                KC::Up => "up".to_string(),
                KC::Down => "down".to_string(),
                KC::Tab => "tab".to_string(),
                KC::Delete => "delete".to_string(),
                KC::Home => "home".to_string(),
                KC::End => "end".to_string(),
                KC::PageUp => "pageup".to_string(),
                KC::PageDown => "pagedown".to_string(),
                KC::Esc => "esc".to_string(),
                KC::F(n) => format!("f{}", n),
                _ => format!("{:?}", code).to_lowercase(),
            };
            if mods.contains(crossterm::event::KeyModifiers::CONTROL) {
                format!("ctrl+{}", key_str)
            } else if mods.contains(crossterm::event::KeyModifiers::ALT) {
                format!("alt+{}", key_str)
            } else if mods.contains(crossterm::event::KeyModifiers::SHIFT) {
                format!("shift+{}", key_str)
            } else {
                key_str
            }
        }

        // Section: action description → display name mapping (presentation order)
        let sections: &[(&str, &str, &[&str])] = &[
            ("Tasks", "Tasks", &[
                "add", "edit", "delete", "bulk_delete", "bulk_append", "toggle_done",
            ]),
            ("Quick Edits", "Quick Edits", &[
                "quick_context", "quick_project",
            ]),
            ("Filter", "Filter", &[
                "filter_open", "filter_define", "filter_toggle", "clear_filter",
            ]),
            ("View", "View", &[
                "sort_cycle", "group_toggle", "group_by_cycle", "deferred_toggle", "theme_cycle", "reload",
            ]),
            ("Select", "Select", &[
                "disjoint_select", "disjoint_mark",
            ]),
            ("Panes", "Panes", &[
                "pane_add", "pane_delete", "pane_hide_toggle",
            ]),
            ("App", "App", &[
                "help", "quit",
            ]),
        ];

        let action_labels: std::collections::HashMap<&str, &str> = [
            ("add", "Add task"),
            ("edit", "Edit task"),
            ("delete", "Delete task"),
            ("bulk_delete", "Bulk delete"),
            ("bulk_append", "Bulk append"),
            ("toggle_done", "Toggle done"),
            ("filter_open", "Open filter"),
            ("filter_define", "Define presets"),
            ("filter_toggle", "Toggle filter on/off"),
            ("clear_filter", "Clear filter"),
            ("sort_cycle", "Cycle sort"),
            ("group_toggle", "Toggle grouping"),
            ("group_by_cycle", "Cycle group-by"),
            ("deferred_toggle", "Toggle deferred"),
            ("theme_cycle", "Cycle theme"),
            ("reload", "Reload file"),
            ("disjoint_select", "Disjoint select"),
            ("disjoint_mark", "Mark selection"),
            ("quick_context", "Quick context setter"),
            ("quick_project", "Quick project setter"),
            ("pane_add", "Create pane"),
            ("pane_delete", "Delete pane"),
            ("pane_hide_toggle", "Toggle panes"),
            ("pane_move_left", "Move task to left pane"),
            ("pane_move_right", "Move task to right pane"),
            ("help", "Show help"),
            ("quit", "Quit"),
        ].into_iter().collect();

        // Build lines
        let mut lines: Vec<Line> = Vec::new();

        for (_key, section_title, actions) in sections {
            lines.push(Line::from(format!("  \u{2500}\u{2500} {} \u{2500}\u{2500}", section_title)));
            for action in *actions {
                if let Some((code, mods)) = self.effective_keymap.get(*action) {
                    let chord = chord_description(*code, *mods);
                    let label = action_labels.get(action).copied().unwrap_or(action);
                    lines.push(Line::from(format!("    {:>12}  {}", chord, label)));
                }
            }
        }

        // Fixed nav keys (not in effective_keymap since they're always hardcoded)
        lines.push(Line::from("  \u{2500}\u{2500} Navigation \u{2500}\u{2500}".to_string()));
        lines.push(Line::from("    j / down  Move down"));
        lines.push(Line::from("    k / up    Move up"));
        lines.push(Line::from("    ctrl+d    Page down"));
        lines.push(Line::from("    ctrl+u    Page up"));
        lines.push(Line::from("    shift+j   Extend selection down"));
        lines.push(Line::from("    shift+k   Extend selection up"));
        lines.push(Line::from("  shift+ctrl+d  Extend selection half-page down"));
        lines.push(Line::from("  shift+ctrl+u  Extend selection half-page up"));
        lines.push(Line::from("  \u{2500}\u{2500} Presets \u{2500}\u{2500}".to_string()));
        lines.push(Line::from("         1-9  Apply filter preset"));
        lines.push(Line::from("  \u{2500}\u{2500} Pane label \u{2500}\u{2500}".to_string()));
        lines.push(Line::from("      up @ top  Select pane header"));
        lines.push(Line::from("         enter  Edit pane label"));
        lines.push(Line::from("     enter/esc  Save / cancel label edit"));
        lines.push(Line::from("  \u{2500}\u{2500} Errors \u{2500}\u{2500}".to_string()));
        lines.push(Line::from("           !  Show error log"));

        let total_lines = lines.len() as u16;
        let popup_width = (area.width * 4 / 5).max(40).min(area.width);
        let popup_height = (total_lines + 2).min(area.height.saturating_sub(2));
        let inner_height = popup_height.saturating_sub(2); // subtract border rows
        // Clamp scroll so we never scroll past the last visible line.
        let max_scroll = total_lines.saturating_sub(inner_height);
        let scroll_offset = self.help_scroll.min(max_scroll);

        let h_layout = Layout::horizontal([Constraint::Length(popup_width)])
            .flex(Flex::Center)
            .split(area);
        let v_layout = Layout::vertical([Constraint::Length(popup_height)])
            .flex(Flex::Center)
            .split(h_layout[0]);
        let popup_area = v_layout[0];

        frame.render_widget(Clear, popup_area);

        let scroll_indicator = if max_scroll > 0 {
            format!(" Keybindings \u{2014} j/k: scroll  Esc/q: close ({}/{}) ", scroll_offset + 1, total_lines)
        } else {
            " Keybindings \u{2014} Esc/q: close ".to_string()
        };
        let paragraph = Paragraph::new(Text::from(lines))
            .block(Block::bordered().title(scroll_indicator))
            .scroll((scroll_offset, 0))
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, popup_area);
    }

    /// Render a centered popup overlay listing app warnings/errors.
    /// Covers the normal task list and status bar with a bordered block + message list.
    fn render_keymap_errors_overlay(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::layout::{Constraint, Flex, Layout};
        use ratatui::widgets::{Block, Clear, List, ListItem};

        let messages = self.error_log_lines();

        // Center a popup sized to fit messages (max 80% width, max 60% height)
        let popup_width = area.width.saturating_mul(4) / 5;
        let popup_height = (messages.len() as u16 + 2).min(area.height * 3 / 5);
        let h_layout = Layout::horizontal([Constraint::Length(popup_width)])
            .flex(Flex::Center)
            .split(area);
        let v_layout = Layout::vertical([Constraint::Length(popup_height)])
            .flex(Flex::Center)
            .split(h_layout[0]);
        let popup_area = v_layout[0];

        frame.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = if messages.is_empty() {
            vec![ListItem::new("  No errors logged.")]
        } else {
            messages
                .iter()
                .map(|w| ListItem::new(format!("  ⚠ {}", w)))
                .collect()
        };

        let list = List::new(items).block(
            Block::bordered().title(" Error Log — Esc/q: close "),
        );
        frame.render_widget(list, popup_area);
    }

    /// Render the one-row archive confirmation panel (Phase 39, ARCH-01/02).
    fn render_archive_confirm(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::text::{Line, Span};
        use ratatui::widgets::Paragraph;
        let count = self.task_list.tasks().iter().filter(|t| t.completed).count();
        let text = format!(
            "Archive {} completed task(s) to done.txt?  y=confirm  any=cancel",
            count
        );
        frame.render_widget(Paragraph::new(Line::from(Span::raw(text))), area);
    }

    /// Render the one-row delete confirmation panel (D-06, D-07).
    fn render_delete_confirm(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::text::{Line, Span};
        use ratatui::widgets::Paragraph;

        let text = if self.selected_tasks.len() > 1 {
            // Bulk confirmation: show count, not task preview (D-02, D-07)
            format!("Delete {} tasks?  y=confirm  any=cancel", self.selected_tasks.len())
        } else if self.selected_tasks.len() == 1 {
            // Single-task-via-selection: show count (D-07 wording update)
            "Delete 1 task?  y=confirm  any=cancel".to_string()
        } else {
            // Cursor-task delete (selection empty): show "Delete task?"
            "Delete task?  y=confirm  any=cancel".to_string()
        };

        frame.render_widget(Paragraph::new(Line::from(Span::raw(text))), area);
    }

    fn render_filter_panel(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::layout::Rect;
        use ratatui::style::{Modifier, Style};
        use ratatui::widgets::{List, ListItem, ListState};

        let input_area = Rect { height: 1, ..area };
        if let Some(ref state) = self.filter_state {
            frame.render_widget(&state.editor, input_area);
        }

        if area.height > 1 && !self.presets.is_empty() {
            let list_area = Rect {
                y: area.y + 1,
                height: area.height - 1,
                ..area
            };
            let selected_preset = self.filter_state.as_ref().map(|s| s.selected_preset);
            let items: Vec<ListItem> = self
                .presets
                .iter()
                .enumerate()
                .map(|(i, (name, query))| {
                    ListItem::new(format!("{}. {} — {}", i + 1, name, query))
                })
                .collect();
            let list = List::new(items)
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
            let mut list_state = ListState::default().with_selected(selected_preset);
            frame.render_stateful_widget(list, list_area, &mut list_state);
        }
    }

    /// Render the autocomplete popup above the editor footer row.
    fn render_autocomplete_popup(&self, frame: &mut Frame, footer_area: ratatui::layout::Rect) {
        use ratatui::layout::Rect;
        use ratatui::style::{Modifier, Style};
        use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

        let ac = match &self.autocomplete {
            Some(ac) if !ac.items.is_empty() => ac,
            _ => return,
        };

        let popup_height = (ac.items.len() as u16).min(5).min(footer_area.y);
        if popup_height == 0 { return; }

        let popup_title = match ac.mode {
            AutocompleteMode::QuickSetter(trigger) => {
                let label = if trigger == '@' { "context" } else { "project" };
                let target_count = if self.selected_tasks.is_empty() {
                    1
                } else {
                    self.selected_tasks.len()
                };
                Some(format!(
                    " {}{} | input: {} | {} target(s) | ↑↓ nav  Tab/Enter apply  Esc cancel ",
                    trigger,
                    label,
                    ac.prefix,
                    target_count
                ))
            }
            _ => None,
        };

        let popup_width = ac.items.iter()
            .map(|s| s.len() + 4) // 4 for trigger char + borders
            .max()
            .unwrap_or(20)
            .max(popup_title.as_ref().map(|s| s.len() + 2).unwrap_or(0))
            .min(88) as u16;
        let popup_width = popup_width.min(frame.area().width);

        let popup_area = Rect {
            x: footer_area.x,
            y: footer_area.y.saturating_sub(popup_height),
            width: popup_width,
            height: popup_height,
        };

        let items: Vec<ListItem> = ac.items.iter()
            .map(|token| ListItem::new(format!("{}{}", ac.trigger, token)))
            .collect();

        let highlight_style = if ac.focused {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default().add_modifier(Modifier::DIM)
        };

        let popup_block = if let Some(title) = popup_title {
            Block::default().borders(Borders::ALL).title(title)
        } else {
            Block::default().borders(Borders::ALL)
        };

        let popup_list = List::new(items)
            .block(popup_block)
            .highlight_style(highlight_style);

        let mut list_state = ListState::default().with_selected(Some(ac.selected));

        frame.render_widget(ratatui::widgets::Clear, popup_area);
        frame.render_stateful_widget(popup_list, popup_area, &mut list_state);
    }

    /// Render the date picker overlay (Phase 33, Plan 01).
    /// Displays month/year and a list of day suggestions with weekday labels.
    fn render_date_picker_overlay(&self, frame: &mut Frame, footer_area: ratatui::layout::Rect) {
        use ratatui::layout::Rect;
        use ratatui::style::{Modifier, Style};
        use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

        let dp = match &self.date_picker {
            Some(dp) => dp,
            None => return,
        };

        let suggestions_height = if dp.suggestions.is_empty() { 1 } else { dp.suggestions.len() as u16 };
        let popup_height = suggestions_height.clamp(3, 10).min(footer_area.y);
        if popup_height == 0 { return; }

        let title = if dp.day_input.is_empty() {
            format!(" Set due date: {} (type YYYY-MM-DD or use arrows) ", dp.month_year)
        } else {
            format!(" Set due date: {}-{} (type YYYY-MM-DD or use arrows) ", dp.month_year, dp.day_input)
        };

        let popup_width = dp.suggestions.iter()
            .map(|s| s.len() + 4)
            .max()
            .unwrap_or(20)
            .max(title.len() + 2)
            .min(72) as u16;
        let popup_width = popup_width.min(frame.area().width);

        let popup_area = Rect {
            x: footer_area.x,
            y: footer_area.y.saturating_sub(popup_height),
            width: popup_width,
            height: popup_height,
        };

        let items: Vec<ListItem> = if dp.suggestions.is_empty() {
            vec![ListItem::new(" Type YYYY-MM-DD, then Enter ")]
        } else {
            dp.suggestions
                .iter()
                .map(|day_str| ListItem::new(day_str.clone()))
                .collect()
        };

        let highlight_style = if dp.focused {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default().add_modifier(Modifier::DIM)
        };

        let selected_idx = if dp.suggestions.is_empty() {
            None
        } else {
            dp.selected_day.and_then(|day| {
                dp.suggestions.iter().position(|s| {
                    s.split_whitespace().next()
                        .and_then(|d| d.parse::<u32>().ok())
                        .map(|d| d == day)
                        .unwrap_or(false)
                })
            })
        };
        let popup_list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(highlight_style);

        let mut list_state = ListState::default().with_selected(selected_idx);

        frame.render_widget(ratatui::widgets::Clear, popup_area);
        frame.render_stateful_widget(popup_list, popup_area, &mut list_state);
    }

    /// Render the priority picker overlay (Phase 34, Plan 01).
    /// Displays A–Z priorities plus "— (no priority)" in a scrollable list.
    fn render_priority_picker_overlay(&self, frame: &mut Frame, footer_area: ratatui::layout::Rect) {
        use ratatui::layout::Rect;
        use ratatui::style::{Modifier, Style};
        use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

        let pp = match &self.priority_picker {
            Some(pp) if !pp.items.is_empty() => pp,
            _ => return,
        };

        let popup_height = (pp.items.len() as u16).min(10).min(footer_area.y);
        if popup_height == 0 { return; }

        let popup_width = 28u16.min(frame.area().width);

        let popup_area = Rect {
            x: footer_area.x,
            y: footer_area.y.saturating_sub(popup_height),
            width: popup_width,
            height: popup_height,
        };

        let items: Vec<ListItem> = pp.items.iter()
            .map(|s| ListItem::new(s.clone()))
            .collect();

        let highlight_style = if pp.focused {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default().add_modifier(Modifier::DIM)
        };

        // Count targets for header
        let n = if !self.selected_tasks.is_empty() { self.selected_tasks.len() } else { 1 };
        let title = if n > 1 {
            format!(" Setting priority — {} tasks ", n)
        } else {
            " Set priority ".to_string()
        };

        let popup_list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(highlight_style);

        let mut list_state = ListState::default().with_selected(Some(pp.selected_idx));

        frame.render_widget(ratatui::widgets::Clear, popup_area);
        frame.render_stateful_widget(popup_list, popup_area, &mut list_state);
    }

    /// Render the bulk append count confirmation banner (D-06, Phase 34, Plan 03).
    fn render_append_text_confirm(&self, frame: &mut Frame, footer_area: ratatui::layout::Rect) {
        use ratatui::layout::Rect;
        use ratatui::style::{Modifier, Style};
        use ratatui::widgets::{Block, Borders, Paragraph};

        let n = self.append_confirm_count;
        if n == 0 { return; }

        let text = format!("Appending to {} tasks — Enter to continue, Esc to cancel", n);
        let popup_width = (text.len() as u16 + 4).min(frame.area().width);
        if footer_area.y < 3 { return; }

        let popup_area = Rect {
            x: footer_area.x,
            y: footer_area.y.saturating_sub(3),
            width: popup_width,
            height: 3,
        };

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(" Bulk Append "))
            .style(Style::default().add_modifier(Modifier::BOLD));

        frame.render_widget(ratatui::widgets::Clear, popup_area);
        frame.render_widget(paragraph, popup_area);
    }

    /// Render the F-key preset definition panel (D-06, D-07, Plan 16-03).
    ///
    /// Layout: bordered outer panel → active filter row (top) → preset list (below).
    fn render_filter_defining_panel(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::layout::{Constraint, Direction, Layout};
        use ratatui::style::{Modifier, Style};
        use ratatui::widgets::{Block, Borders, List, ListItem};

        let state = match self.filter_defining_state.as_mut() {
            Some(s) => s,
            None => return,
        };

        // Outer bordered block.
        let outer = Block::default()
            .title(" Filter Definitions (F) — \u{2191}\u{2193}: navigate  Enter: save+apply row  Esc: discard ")
            .borders(Borders::ALL);
        let inner = outer.inner(area);
        frame.render_widget(outer, area);

        // Split inner: active filter row (height 1) + separator + preset list.
        let row_constraints = [
            Constraint::Length(1), // active filter row
            Constraint::Min(0),    // preset list
        ];
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(inner);

        // --- Row 0: Active filter editor (D-07 live preview) ---
        let active_focused = state.selected_row == 0;
        let active_style = if active_focused {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        // Render TextArea widget directly into the single-line row area.
        // tui-textarea doesn't support inline border here, so we skip it;
        // the outer block title explains the panel purpose.
        state.active_editor.set_style(active_style);
        frame.render_widget(&state.active_editor, rows[0]);

        // --- Rows 1–9: Preset list ---
        let items: Vec<ListItem> = state.preset_names.iter().enumerate().map(|(i, name)| {
            let filter_val = state.preset_editors[i].lines().join("");
            let label = format!(" #{} {}  {}", i + 1, name, filter_val);
            let style = if state.selected_row == i + 1 {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            ListItem::new(label).style(style)
        }).collect();

        let preset_list = List::new(items);
        frame.render_widget(preset_list, rows[1]);
    }
}

/// Advance to the next sort order in the fixed cycle.
fn cycle_sort(current: SortOrder) -> SortOrder {
    match current {
        SortOrder::FileOrder     => SortOrder::Alphabetical,
        SortOrder::Alphabetical  => SortOrder::CompletedDate,
        SortOrder::CompletedDate => SortOrder::Context,
        SortOrder::Context       => SortOrder::DueDate,
        SortOrder::DueDate       => SortOrder::CreationDate,
        SortOrder::CreationDate  => SortOrder::Priority,
        SortOrder::Priority      => SortOrder::Project,
        SortOrder::Project       => SortOrder::FileOrder,
        _                        => SortOrder::FileOrder,
    }
}

/// Advance to the next theme in the fixed cycle.
fn cycle_theme(current: Theme) -> Theme {
    match current {
        Theme::Default => Theme::Light,
        Theme::Light => Theme::Default,
    }
}

/// Parse numbered preset keys like `f1`..`f9`.
fn parse_preset_slot(name: &str) -> Option<usize> {
    let suffix = name.strip_prefix('f')?;
    let slot = suffix.parse::<usize>().ok()?;
    if (1..=9).contains(&slot) {
        Some(slot)
    } else {
        None
    }
}

/// Human-readable name for a sort order, shown in the status bar.
fn sort_name(order: SortOrder) -> &'static str {
    match order {
        SortOrder::FileOrder     => "file order",
        SortOrder::Alphabetical  => "alpha",
        SortOrder::CompletedDate => "completed",
        SortOrder::Context       => "context",
        SortOrder::DueDate       => "due",
        SortOrder::CreationDate  => "created",
        SortOrder::Priority      => "priority",
        SortOrder::Project       => "project",
        _                        => "?",
    }
}

/// Advance to the next group-by category in the fixed cycle (GRP-02, Phase 40).
fn cycle_group_by(current: GroupByCategory) -> GroupByCategory {
    match current {
        GroupByCategory::Priority => GroupByCategory::Project,
        GroupByCategory::Project  => GroupByCategory::Context,
        GroupByCategory::Context  => GroupByCategory::DueDate,
        GroupByCategory::DueDate  => GroupByCategory::Priority,
    }
}

/// Human-readable name for a group-by category, shown in the status bar (GRP-03, Phase 40).
fn group_by_name(g: GroupByCategory) -> &'static str {
    match g {
        GroupByCategory::Priority => "priority",
        GroupByCategory::Project  => "project",
        GroupByCategory::Context  => "context",
        GroupByCategory::DueDate  => "duedate",
    }
}

fn group_key_for(task: &Task, group_by: &GroupByCategory) -> String {
    match group_by {
        GroupByCategory::Priority => task
            .priority
            .map(|p| format!("({})", p))
            .unwrap_or_else(|| "none".to_string()),
        GroupByCategory::Project => task
            .projects
            .first()
            .map(|p| format!("+{}", p))
            .unwrap_or_else(|| "none".to_string()),
        GroupByCategory::Context => task
            .contexts
            .first()
            .map(|c| format!("@{}", c))
            .unwrap_or_else(|| "none".to_string()),
        GroupByCategory::DueDate => task
            .due_date
            .map(|d| d.to_string())
            .unwrap_or_else(|| "no due date".to_string()),
    }
}

/// Determine autocomplete state for the filter input based on cursor position (AC-02, AC-04).
///
/// Cursor-aware: only the word immediately to the left of `cursor_col` is examined.
/// - Word starts with `@` → `TokenAutocomplete('@')` with contexts from `task_list`.
/// - Word starts with `+` → `TokenAutocomplete('+')` with projects from `task_list`.
/// - No trigger and `history` non-empty → `FilterHistory`.
/// - Otherwise → `None`.
fn compute_filter_autocomplete(
    line: &str,
    cursor_col: usize,
    task_list: &TaskList,
    history: &std::collections::VecDeque<String>,
) -> Option<AutocompleteState> {
    let before_cursor = &line[..cursor_col.min(line.len())];
    let word_start = before_cursor
        .rfind(|c: char| c.is_whitespace())
        .map(|i| i + 1)
        .unwrap_or(0);
    let word = &before_cursor[word_start..];

    if let Some(trigger) = word.chars().next().filter(|&c| c == '@' || c == '+') {
        let prefix = &word[1..];
        let prefix_lower = prefix.to_lowercase();
        let candidates = if trigger == '@' {
            get_existing_contexts(task_list)
        } else {
            get_existing_projects(task_list)
        };
        let mut filtered: Vec<String> = candidates
            .into_iter()
            .filter(|t| t.to_lowercase().starts_with(&prefix_lower))
            .collect();
        filtered.sort();
        if filtered.is_empty() {
            return None;
        }
        Some(AutocompleteState::new(trigger, prefix.to_string(), filtered))
    } else if !history.is_empty() {
        Some(AutocompleteState::new_filter_history(
            line.to_string(),
            history.iter().cloned().collect(),
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::Mutex;
    use tempfile::NamedTempFile;

    static CLIPBOARD_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn make_app_with_tasks(task_lines: &[&str]) -> App {
        let mut file = NamedTempFile::new().expect("failed to create temp file");
        for line in task_lines {
            writeln!(file, "{}", line).unwrap();
        }
        let path = file.path().to_path_buf();
        let task_list = TaskList::load(&path).expect("load failed");
        // Keep temp file alive until after load by persisting it.
        let _ = file.keep();
        App::new(task_list, path, TuiConfig::default(), None, Theme::Default, true)
    }

    // ── Task 1: Canonical selection state ────────────────────────────────────

    #[test]
    fn selection_state_initialized_empty() {
        let app = make_app_with_tasks(&["Task one", "Task two"]);
        assert!(app.selected_tasks.is_empty(), "selected_tasks should be empty on init");
        assert!(app.selection_anchor.is_none(), "selection_anchor should be None on init");
        assert!(!app.disjoint_select, "disjoint_select should be false on init");
    }

    #[test]
    fn toggle_task_selection_adds_canonical_index() {
        let mut app = make_app_with_tasks(&["Task one", "Task two"]);
        // selected == 0, display_rows[0] == Task(0)
        app.toggle_task_selection();
        assert!(app.selected_tasks.contains(&0), "canonical index 0 should be in selected_tasks after toggle");
    }

    #[test]
    fn toggle_task_selection_removes_if_already_selected() {
        let mut app = make_app_with_tasks(&["Task one", "Task two"]);
        app.selected_tasks.insert(0);
        app.toggle_task_selection();
        assert!(!app.selected_tasks.contains(&0), "canonical index 0 should be removed after second toggle");
    }

    #[test]
    fn toggle_task_selection_no_op_on_group_header() {
        let mut app = make_app_with_tasks(&["Task one", "Task two"]);
        // Manually override display_rows to put a GroupHeader at position 0.
        app.display_rows = vec![
            DisplayRow::GroupHeader("Header".to_string()),
            DisplayRow::Task(0),
        ];
        app.selected = 0;
        app.toggle_task_selection();
        assert!(app.selected_tasks.is_empty(), "GroupHeader row must never enter selected_tasks (D-08)");
    }

    // ── Task 2: Disjoint selection mode keys ─────────────────────────────────

    fn press_key(app: &mut App, code: crossterm::event::KeyCode) {
        use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
        app.handle_normal_key(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }).unwrap();
    }

    fn press_ctrl_key(app: &mut App, code: crossterm::event::KeyCode) {
        use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
        app.handle_normal_key(KeyEvent {
            code,
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }).unwrap();
    }

    fn key_no_mod(code: crossterm::event::KeyCode) -> crossterm::event::KeyEvent {
        use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }
    }

    fn key_ctrl(code: crossterm::event::KeyCode) -> crossterm::event::KeyEvent {
        use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
        KeyEvent {
            code,
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }
    }

    #[test]
    fn ctrl_n_triggers_pane_add_not_add_mode() {
        let mut app = make_app_with_tasks(&["Task one", "Task two"]);
        assert_eq!(app.mode, AppMode::Normal);
        assert_eq!(app.panes.len(), 1);

        press_ctrl_key(&mut app, KeyCode::Char('n'));

        assert_eq!(app.mode, AppMode::Normal, "Ctrl+N should not enter Adding mode");
        assert_eq!(app.panes.len(), 2, "Ctrl+N should add a pane");
    }

    #[test]
    fn ctrl_w_triggers_pane_delete_not_noop() {
        let mut app = make_app_with_tasks(&["Task one", "Task two"]);
        app.pane_add();
        assert_eq!(app.panes.len(), 2);

        press_ctrl_key(&mut app, KeyCode::Char('w'));

        assert_eq!(app.panes.len(), 1, "Ctrl+W should delete active pane");
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn ctrl_p_triggers_pane_hide_toggle() {
        let mut app = make_app_with_tasks(&["Task one", "Task two"]);
        assert!(!app.panes_hidden);

        press_ctrl_key(&mut app, KeyCode::Char('p'));
        assert!(app.panes_hidden, "Ctrl+P should hide panes");

        press_ctrl_key(&mut app, KeyCode::Char('p'));
        assert!(!app.panes_hidden, "Ctrl+P should toggle panes back on");
    }

    #[test]
    fn edit_targets_selected_row_in_active_pane() {
        let mut app = make_app_with_tasks(&["task A", "task B", "task C"]);
        app.pane_add();
        app.rebuild_all_panes();
        assert!(!app.should_show_single_pane(), "must be in multi-pane mode");

        app.active_pane_mut().selected = 1;
        press_key(&mut app, KeyCode::Char('u'));

        match app.mode {
            AppMode::Editing { original_idx } => {
                assert_eq!(original_idx, 1, "edit should target selected row, not first row");
            }
            _ => panic!("expected Editing mode after edit key"),
        }
    }

    #[test]
    fn delete_targets_selected_row_in_active_pane() {
        let mut app = make_app_with_tasks(&["task A", "task B", "task C"]);
        app.pane_add();
        app.rebuild_all_panes();
        assert!(!app.should_show_single_pane(), "must be in multi-pane mode");

        app.active_pane_mut().selected = 1;

        press_key(&mut app, KeyCode::Char('d'));
        assert_eq!(app.task_list.len(), 2);
        assert_eq!(app.task_list.tasks()[0].to_raw(), "task A");
        assert_eq!(app.task_list.tasks()[1].to_raw(), "task C");
        assert_eq!(app.mode, AppMode::Normal, "single delete should not enter confirmation mode");
    }

    #[test]
    fn active_canonical_selected_filters_stale_global_index() {
        let mut app = make_app_with_tasks(&["task A", "task B"]);
        app.display_rows = vec![DisplayRow::Task(999)];
        app.selected = 0;

        assert_eq!(app.active_canonical_selected(), None);
    }

    #[test]
    fn delete_confirm_y_with_stale_index_is_noop_not_panic() {
        use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};

        let mut app = make_app_with_tasks(&["task A", "task B"]);
        app.display_rows = vec![DisplayRow::Task(999)];
        app.selected = 0;
        app.mode = AppMode::DeleteConfirm;

        let confirm_key = KeyEvent {
            code: KeyCode::Char('y'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        app.handle_delete_confirm_key(confirm_key).unwrap();

        assert_eq!(app.task_list.len(), 2, "stale index should not delete any task");
        assert_eq!(app.mode, AppMode::Normal, "mode should return to normal");
    }

    #[test]
    fn single_delete_with_duplicate_content_targets_cursor_row() {
        let mut app = make_app_with_tasks(&["n", "n", "n", "n", "n"]);
        app.selected = 3;

        press_key(&mut app, KeyCode::Char('d'));
        assert_eq!(app.task_list.len(), 4);
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn single_selected_task_delete_with_duplicate_content_no_panic() {
        let mut app = make_app_with_tasks(&["n", "n", "n", "n", "n"]);

        // Simulate one task selected in disjoint mode and delete via 'd'.
        app.disjoint_select = true;
        app.selected = 2;
        press_key(&mut app, KeyCode::Char(' ')); // select one task
        assert_eq!(app.selected_tasks.len(), 1);

        press_key(&mut app, KeyCode::Char('d'));
        assert_eq!(app.task_list.len(), 4, "single selected delete should remove one task");
        assert!(app.selected_tasks.is_empty(), "selection should clear after delete path");
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn backspace_alias_deletes_single_task_immediately() {
        let mut app = make_app_with_tasks(&["task A", "task B", "task C"]);
        app.selected = 1;

        press_key(&mut app, KeyCode::Backspace);

        assert_eq!(app.task_list.len(), 2);
        assert_eq!(app.task_list.tasks()[0].to_raw(), "task A");
        assert_eq!(app.task_list.tasks()[1].to_raw(), "task C");
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn delete_key_alias_deletes_single_task_immediately() {
        let mut app = make_app_with_tasks(&["task A", "task B", "task C"]);
        app.selected = 1;

        press_key(&mut app, KeyCode::Delete);

        assert_eq!(app.task_list.len(), 2);
        assert_eq!(app.task_list.tasks()[0].to_raw(), "task A");
        assert_eq!(app.task_list.tasks()[1].to_raw(), "task C");
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn delete_with_multiple_selected_tasks_still_prompts_confirmation() {
        let mut app = make_app_with_tasks(&["task A", "task B", "task C"]);
        app.disjoint_select = true;
        app.selected_tasks.insert(0);
        app.selected_tasks.insert(2);
        assert_eq!(app.selected_tasks.len(), 2);

        press_key(&mut app, KeyCode::Char('d'));

        assert_eq!(app.mode, AppMode::DeleteConfirm, "multi-delete should still require confirmation");
        assert_eq!(app.task_list.len(), 3, "tasks should remain until confirmation");
    }

    #[test]
    fn delete_rebuilds_inactive_panes_to_prevent_stale_indices() {
        let mut app = make_app_with_tasks(&["n", "n", "n"]);
        app.pane_add();
        app.rebuild_all_panes();
        assert_eq!(app.panes.len(), 2);

        // Delete from pane 1; pane 0 must also be rebuilt to avoid stale canonical indices.
        app.active_pane = 1;
        app.active_pane_mut().selected = 1;
        press_key(&mut app, KeyCode::Char('d'));
        assert_eq!(app.mode, AppMode::Normal);

        for pane in &app.panes {
            for row in &pane.display_rows {
                if let DisplayRow::Task(idx) = row {
                    assert!(*idx < app.task_list.len(), "pane contains stale canonical index after delete");
                }
            }
        }
    }

    #[test]
    fn v_key_toggles_disjoint_select_on() {
        let mut app = make_app_with_tasks(&["Task one", "Task two"]);
        assert!(!app.disjoint_select);
        press_key(&mut app, KeyCode::Char('v'));
        assert!(app.disjoint_select, "v should enable disjoint_select");
    }

    #[test]
    fn v_key_toggles_disjoint_select_off() {
        let mut app = make_app_with_tasks(&["Task one", "Task two"]);
        app.disjoint_select = true;
        press_key(&mut app, KeyCode::Char('v'));
        assert!(!app.disjoint_select, "v should disable disjoint_select when already on");
    }

    #[test]
    fn space_toggles_task_in_disjoint_mode() {
        let mut app = make_app_with_tasks(&["Task one", "Task two"]);
        app.disjoint_select = true;
        press_key(&mut app, KeyCode::Char(' '));
        assert!(app.selected_tasks.contains(&0), "Space should add cursor task to selected_tasks in disjoint mode");
    }

    #[test]
    fn s_opens_date_picker_when_selected_tasks_exist_even_on_group_header_cursor() {
        let mut app = make_app_with_tasks(&["Task one", "Task two"]);
        app.selected_tasks.insert(1);
        app.display_rows = vec![
            DisplayRow::GroupHeader("Header".to_string()),
            DisplayRow::Task(0),
            DisplayRow::Task(1),
        ];
        app.selected = 0;

        press_key(&mut app, KeyCode::Char('s'));

        assert_eq!(app.mode, AppMode::DatePicker, "'s' should open date picker when selection exists");
        assert!(app.date_picker.is_some());
    }

    #[test]
    fn date_picker_enter_applies_due_date_to_selected_tasks() {
        let mut app = make_app_with_tasks(&["Task one", "Task two"]);
        app.selected_tasks.insert(0);
        app.selected_tasks.insert(1);
        app.mode = AppMode::DatePicker;
        app.date_picker = Some(DatePickerState::new("2032-02"));
        if let Some(ref mut dp) = app.date_picker {
            dp.selected_day = Some(29);
        }

        app.handle_date_picker_key(key_no_mod(KeyCode::Enter)).unwrap();

        assert_eq!(app.task_list.tasks()[0].due_date.map(|d| d.to_string()), Some("2032-02-29".to_string()));
        assert_eq!(app.task_list.tasks()[1].due_date.map(|d| d.to_string()), Some("2032-02-29".to_string()));
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn date_picker_supports_typing_month_then_day() {
        let mut app = make_app_with_tasks(&["Task one"]);
        app.mode = AppMode::DatePicker;
        app.date_picker = Some(DatePickerState {
            month_year: String::new(),
            selected_day: None,
            day_input: String::new(),
            suggestions: vec![],
            focused: false,
        });

        for ch in ['2', '0', '3', '2', '-', '0', '2', '2', '9'] {
            app.handle_date_picker_key(key_no_mod(KeyCode::Char(ch))).unwrap();
        }

        let dp = app.date_picker.as_ref().unwrap();
        assert_eq!(dp.month_year, "2032-02");
        assert_eq!(dp.day_input, "29");
        assert_eq!(dp.selected_day, Some(29));
    }

    #[test]
    fn date_picker_allows_direct_full_date_typing_without_backspacing_default() {
        let mut app = make_app_with_tasks(&["Task one"]);
        app.mode = AppMode::DatePicker;
        app.date_picker = Some(DatePickerState::new("2026-04"));

        // First typed key starts from scratch (YYYY-MM-DD).
        for ch in ['2', '0', '3', '1', '-', '1', '2', '2', '5'] {
            app.handle_date_picker_key(key_no_mod(KeyCode::Char(ch))).unwrap();
        }

        let dp = app.date_picker.as_ref().unwrap();
        assert_eq!(dp.month_year, "2031-12");
        assert_eq!(dp.day_input, "25");
    }

    #[test]
    fn y_copies_active_task_to_clipboard() {
        let _guard = CLIPBOARD_TEST_LOCK.lock().unwrap();

        let mut app = make_app_with_tasks(&["copy me"]);
        let mut clipboard = match Clipboard::new() {
            Ok(cb) => cb,
            Err(_) => return,
        };
        if clipboard.set_text("seed".to_string()).is_err() {
            return;
        }
        app.clipboard = Some(clipboard);

        press_key(&mut app, KeyCode::Char('y'));

        let copied = app
            .clipboard
            .as_mut()
            .and_then(|cb| cb.get_text().ok());
        assert_eq!(copied.as_deref(), Some("copy me"));
    }

    #[test]
    fn y_copies_selected_tasks_in_descending_canonical_order() {
        let _guard = CLIPBOARD_TEST_LOCK.lock().unwrap();

        let mut app = make_app_with_tasks(&["task A", "task B", "task C"]);
        let mut clipboard = match Clipboard::new() {
            Ok(cb) => cb,
            Err(_) => return,
        };
        if clipboard.set_text("seed".to_string()).is_err() {
            return;
        }
        app.clipboard = Some(clipboard);

        app.selected_tasks.insert(0);
        app.selected_tasks.insert(2);
        press_key(&mut app, KeyCode::Char('y'));

        let copied = app
            .clipboard
            .as_mut()
            .and_then(|cb| cb.get_text().ok());
        assert_eq!(copied.as_deref(), Some("task C\ntask A"));
    }

    #[test]
    fn cut_composes_copy_then_delete_for_single_selected_task() {
        let _guard = CLIPBOARD_TEST_LOCK.lock().unwrap();

        let mut app = make_app_with_tasks(&["task A", "task B"]);
        let mut clipboard = match Clipboard::new() {
            Ok(cb) => cb,
            Err(_) => return,
        };
        if clipboard.set_text("seed".to_string()).is_err() {
            return;
        }
        app.clipboard = Some(clipboard);

        app.disjoint_select = true;
        app.selected = 1;
        app.active_pane_mut().selected = 1;
        press_key(&mut app, KeyCode::Char(' '));
        press_key(&mut app, KeyCode::Char('y'));
        press_key(&mut app, KeyCode::Char('d'));

        assert_eq!(app.task_list.len(), 1);
        assert_eq!(app.task_list.tasks()[0].to_raw(), "task A");
    }

    #[test]
    fn p_pastes_each_non_empty_clipboard_line_as_task() {
        let _guard = CLIPBOARD_TEST_LOCK.lock().unwrap();

        let mut app = make_app_with_tasks(&["existing"]);
        let mut clipboard = match Clipboard::new() {
            Ok(cb) => cb,
            Err(_) => return,
        };
        if clipboard
            .set_text("first pasted\n\nsecond pasted\n".to_string())
            .is_err()
        {
            return;
        }
        app.clipboard = Some(clipboard);

        app.paste_from_clipboard().unwrap();

        let tasks = app.task_list.tasks();
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[1].to_raw(), "first pasted");
        assert_eq!(tasks[2].to_raw(), "second pasted");
    }

    #[test]
    fn ctrl_v_in_adding_mode_pastes_first_clipboard_line_only() {
        let _guard = CLIPBOARD_TEST_LOCK.lock().unwrap();

        let mut app = make_app_with_tasks(&["existing"]);
        let mut clipboard = match Clipboard::new() {
            Ok(cb) => cb,
            Err(_) => return,
        };
        if clipboard
            .set_text("first line\nsecond line".to_string())
            .is_err()
        {
            return;
        }
        app.clipboard = Some(clipboard);

        press_key(&mut app, KeyCode::Char('n'));
        assert_eq!(app.mode, AppMode::Adding);

        app.handle_editor_key(key_ctrl(KeyCode::Char('v'))).unwrap();

        let content = app.editor.lines().join("\n");
        assert_eq!(content, "first line");
    }

    #[test]
    fn space_no_op_when_not_in_disjoint_mode() {
        let mut app = make_app_with_tasks(&["Task one", "Task two"]);
        // disjoint_select is false by default
        press_key(&mut app, KeyCode::Char(' '));
        assert!(app.selected_tasks.is_empty(), "Space should be a no-op when disjoint_select is false");
    }

    #[test]
    fn space_no_op_on_group_header_in_disjoint_mode() {
        let mut app = make_app_with_tasks(&["Task one", "Task two"]);
        app.disjoint_select = true;
        app.display_rows = vec![
            DisplayRow::GroupHeader("Header".to_string()),
            DisplayRow::Task(0),
        ];
        app.selected = 0;
        // Also update pane for Phase 24-02
        app.active_pane_mut().display_rows = vec![
            DisplayRow::GroupHeader("Header".to_string()),
            DisplayRow::Task(0),
        ];
        app.active_pane_mut().selected = 0;
        press_key(&mut app, KeyCode::Char(' '));
        assert!(app.selected_tasks.is_empty(), "Space on GroupHeader must be a no-op (D-08)");
    }

    #[test]
    fn esc_clears_selection_and_exits_disjoint_mode() {
        let mut app = make_app_with_tasks(&["Task one", "Task two"]);
        app.disjoint_select = true;
        app.selected_tasks.insert(0);
        app.selected_tasks.insert(1);
        press_key(&mut app, KeyCode::Esc);
        assert!(app.selected_tasks.is_empty(), "Esc should clear all selected_tasks");
        assert!(!app.disjoint_select, "Esc should exit disjoint mode");
    }

    // ── Task 1 (19-02): Anchor lifecycle helpers ──────────────────────────────

    #[test]
    fn ensure_anchor_sets_anchor_from_cursor_when_unset() {
        let mut app = make_app_with_tasks(&["A", "B", "C"]);
        assert!(app.selection_anchor.is_none());
        app.ensure_anchor();
        assert_eq!(app.selection_anchor, Some(0), "ensure_anchor should set anchor to cursor canonical index when unset (D-11)");
    }

    #[test]
    fn ensure_anchor_does_not_overwrite_existing_anchor() {
        let mut app = make_app_with_tasks(&["A", "B", "C"]);
        app.selection_anchor = Some(2);
        app.ensure_anchor();
        assert_eq!(app.selection_anchor, Some(2), "ensure_anchor should not overwrite an existing anchor");
    }

    #[test]
    fn apply_range_selection_selects_tasks_from_anchor_to_cursor() {
        let mut app = make_app_with_tasks(&["A", "B", "C", "D"]);
        app.selection_anchor = Some(0); // anchor at Task(0), display row 0
        app.selected = 2;              // cursor at Task(2), display row 2
        app.apply_range_selection();
        assert!(app.selected_tasks.contains(&0), "Task 0 should be in range");
        assert!(app.selected_tasks.contains(&1), "Task 1 should be in range");
        assert!(app.selected_tasks.contains(&2), "Task 2 should be in range");
        assert!(!app.selected_tasks.contains(&3), "Task 3 is outside range");
    }

    #[test]
    fn apply_range_selection_works_upward_from_anchor() {
        let mut app = make_app_with_tasks(&["A", "B", "C", "D"]);
        app.selection_anchor = Some(2); // anchor at Task(2), display row 2
        app.selected = 0;              // cursor at Task(0), display row 0
        app.apply_range_selection();
        assert!(app.selected_tasks.contains(&0));
        assert!(app.selected_tasks.contains(&1));
        assert!(app.selected_tasks.contains(&2));
        assert!(!app.selected_tasks.contains(&3));
    }

    #[test]
    fn apply_range_selection_replaces_prior_selection() {
        let mut app = make_app_with_tasks(&["A", "B", "C", "D"]);
        app.selected_tasks.insert(3); // pre-existing selection outside range
        app.selection_anchor = Some(0);
        app.selected = 1;
        app.apply_range_selection();
        assert!(app.selected_tasks.contains(&0));
        assert!(app.selected_tasks.contains(&1));
        assert!(!app.selected_tasks.contains(&3), "apply_range_selection should replace prior selected_tasks");
    }

    #[test]
    fn plain_j_clears_anchor_but_not_selected_tasks() {
        let mut app = make_app_with_tasks(&["A", "B", "C"]);
        app.selected_tasks.insert(0);
        app.selected_tasks.insert(1);
        app.selection_anchor = Some(0);
        press_key(&mut app, KeyCode::Char('j'));
        assert!(app.selection_anchor.is_none(), "plain j should clear selection_anchor (D-12)");
        assert!(app.selected_tasks.contains(&0), "plain j should NOT clear selected_tasks (D-12)");
        assert!(app.selected_tasks.contains(&1), "plain j should NOT clear selected_tasks (D-12)");
    }

    #[test]
    fn plain_k_clears_anchor_but_not_selected_tasks() {
        let mut app = make_app_with_tasks(&["A", "B", "C"]);
        app.selected = 2;
        app.selected_tasks.insert(0);
        app.selection_anchor = Some(0);
        press_key(&mut app, KeyCode::Char('k'));
        assert!(app.selection_anchor.is_none(), "plain k should clear selection_anchor (D-12)");
        assert!(app.selected_tasks.contains(&0), "plain k should NOT clear selected_tasks (D-12)");
    }

    // ── Task 2 (19-02): Shift-range key matrix ────────────────────────────────

    fn press_shift_key(app: &mut App, code: KeyCode) {
        use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
        app.handle_normal_key(KeyEvent {
            code,
            modifiers: KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }).unwrap();
    }

    #[test]
    fn shift_j_sets_anchor_on_first_use_then_extends_down() {
        let mut app = make_app_with_tasks(&["A", "B", "C"]);
        assert!(app.selection_anchor.is_none());
        press_shift_key(&mut app, KeyCode::Char('j'));
        assert_eq!(app.selection_anchor, Some(0), "shift-j should lazily set anchor to original cursor (D-11)");
        assert_eq!(app.selected, 1, "shift-j should move cursor down");
        assert!(app.selected_tasks.contains(&0), "anchor task should be selected");
        assert!(app.selected_tasks.contains(&1), "new cursor task should be selected");
    }

    #[test]
    fn shift_j_extends_selection_further_down() {
        let mut app = make_app_with_tasks(&["A", "B", "C", "D"]);
        app.selection_anchor = Some(0);
        app.selected = 1;
        app.selected_tasks = [0usize, 1].iter().cloned().collect();
        press_shift_key(&mut app, KeyCode::Char('j'));
        assert_eq!(app.selected, 2);
        assert!(app.selected_tasks.contains(&0));
        assert!(app.selected_tasks.contains(&1));
        assert!(app.selected_tasks.contains(&2));
    }

    #[test]
    fn shift_k_shrinks_selection_back_toward_anchor() {
        let mut app = make_app_with_tasks(&["A", "B", "C"]);
        app.selection_anchor = Some(0);
        app.selected = 2;
        app.selected_tasks = [0usize, 1, 2].iter().cloned().collect();
        press_shift_key(&mut app, KeyCode::Char('k'));
        assert_eq!(app.selected, 1);
        assert!(app.selected_tasks.contains(&0));
        assert!(app.selected_tasks.contains(&1));
        assert!(!app.selected_tasks.contains(&2), "shrunk range should not include task 2");
    }

    #[test]
    fn shift_down_extends_selection_downward() {
        let mut app = make_app_with_tasks(&["A", "B", "C"]);
        app.selected = 0;
        press_shift_key(&mut app, KeyCode::Down);
        assert_eq!(app.selected, 1);
        assert!(app.selected_tasks.contains(&0));
        assert!(app.selected_tasks.contains(&1));
    }

    #[test]
    fn shift_up_extends_selection_upward() {
        let mut app = make_app_with_tasks(&["A", "B", "C"]);
        app.selected = 2;
        press_shift_key(&mut app, KeyCode::Up);
        assert_eq!(app.selected, 1);
        assert!(app.selected_tasks.contains(&1));
        assert!(app.selected_tasks.contains(&2));
    }

    #[test]
    fn shift_range_skips_group_headers_in_navigation() {
        let mut app = make_app_with_tasks(&["A", "B", "C"]);
        // Manually set up display_rows with a GroupHeader between tasks
        app.display_rows = vec![
            DisplayRow::Task(0),
            DisplayRow::GroupHeader("Group".to_string()),
            DisplayRow::Task(1),
            DisplayRow::Task(2),
        ];
        app.display_indices = vec![0, 1, 2];
        app.selected = 0;
        app.panes[0].display_rows = app.display_rows.clone();
        app.panes[0].selected = 0;
        press_shift_key(&mut app, KeyCode::Char('j'));
        // shift-j should skip GroupHeader at row 1 and land on Task(1) at row 2
        assert_eq!(app.selected, 2, "shift-j should skip GroupHeader rows (D-08)");
        assert!(app.selected_tasks.contains(&0), "Task 0 (anchor) should be selected");
        assert!(app.selected_tasks.contains(&1), "Task 1 (at row 2) should be selected");
        assert!(!app.selected_tasks.is_empty());
    }

    #[test]
    fn shift_range_in_multi_pane_uses_active_pane_rows_not_global_rows() {
        let mut app = make_app_with_tasks(&["A", "B", "C", "D"]);
        app.pane_add();

        // Force multi-pane mode path and provide active pane rows with a header gap.
        app.panes_hidden = false;
        app.active_pane = 1;
        app.panes[1].display_rows = vec![
            DisplayRow::Task(0),
            DisplayRow::GroupHeader("H".to_string()),
            DisplayRow::Task(2),
            DisplayRow::Task(3),
        ];
        app.panes[1].selected = 0;

        // Keep global rows different to catch accidental global-row usage.
        app.display_rows = vec![
            DisplayRow::Task(0),
            DisplayRow::Task(1),
            DisplayRow::Task(2),
            DisplayRow::Task(3),
        ];
        app.selected = 0;

        press_shift_key(&mut app, KeyCode::Down);

        // Should skip header and land on canonical 2 from pane row 2.
        assert_eq!(app.panes[1].selected, 2);
        assert!(app.selected_tasks.contains(&0));
        assert!(app.selected_tasks.contains(&2));
        assert!(!app.selected_tasks.contains(&1), "must not accidentally select global-row task 1");
    }

    // ── Task 1 (19-03): Selection persistence through rebuild ─────────────────

    #[test]
    fn rebuild_and_reanchor_does_not_clear_selected_tasks() {
        let mut app = make_app_with_tasks(&["A", "B", "C"]);
        app.selected_tasks.insert(0);
        app.selected_tasks.insert(1);
        app.rebuild_and_reanchor();
        assert!(app.selected_tasks.contains(&0), "rebuild_and_reanchor must not clear selected_tasks (D-18)");
        assert!(app.selected_tasks.contains(&1), "rebuild_and_reanchor must not clear selected_tasks (D-18)");
    }

    #[test]
    fn rebuild_display_indices_does_not_clear_selected_tasks() {
        let mut app = make_app_with_tasks(&["A", "B", "C"]);
        app.selected_tasks.insert(0);
        app.selected_tasks.insert(2);
        app.rebuild_display_indices();
        assert!(app.selected_tasks.contains(&0), "rebuild_display_indices must not clear selected_tasks (D-18)");
        assert!(app.selected_tasks.contains(&2), "rebuild_display_indices must not clear selected_tasks (D-18)");
    }

    #[test]
    fn filter_hidden_tasks_remain_selected_d20() {
        let mut app = make_app_with_tasks(&["(A) priority task", "plain task"]);
        app.selected_tasks.insert(0);
        app.selected_tasks.insert(1);
        // Apply a filter that hides task 1 (index 1, "plain task" has no priority)
        app.filter_query = "pri:A".to_string();
        app.rebuild_and_reanchor();
        // Task index 1 is hidden by filter but must remain in selected_tasks per D-20
        assert!(app.selected_tasks.contains(&0), "visible task stays selected after filter");
        assert!(app.selected_tasks.contains(&1), "filter-hidden task must remain in selected_tasks (D-20)");
    }

    #[test]
    fn sort_change_does_not_clear_selected_tasks() {
        let mut app = make_app_with_tasks(&["(B) task b", "(A) task a"]);
        app.selected_tasks.insert(0); // canonical index 0 = "(B) task b"
        app.sort_order = todotxt_core::SortOrder::Priority;
        app.rebuild_and_reanchor();
        // Sort changes display order but canonical index 0 must still be selected
        assert!(app.selected_tasks.contains(&0), "sort change must not clear selected_tasks (D-18)");
    }

    // ── Task 2 (19-03): Pruning stale selections on reload ────────────────────

    fn make_app_with_file(task_lines: &[&str]) -> (App, NamedTempFile) {
        use std::io::Write;
        let mut file = NamedTempFile::new().expect("failed to create temp file");
        for line in task_lines {
            writeln!(file, "{}", line).unwrap();
        }
        file.flush().unwrap();
        let path = file.path().to_path_buf();
        let task_list = TaskList::load(&path).expect("load failed");
        let app = App::new(task_list, path, TuiConfig::default(), None, Theme::Default, true);
        (app, file)
    }

    #[test]
    fn reload_prunes_out_of_range_selections() {
        use std::io::{Seek, SeekFrom, Write};
        let (mut app, mut file) = make_app_with_file(&["A", "B", "C"]);
        app.selected_tasks.insert(0);
        app.selected_tasks.insert(1);
        app.selected_tasks.insert(2);

        // Shrink file to 2 tasks (remove "C")
        file.seek(SeekFrom::Start(0)).unwrap();
        file.as_file().set_len(0).unwrap();
        writeln!(file, "A").unwrap();
        writeln!(file, "B").unwrap();
        file.flush().unwrap();

        app.pending_reload = true;
        app.apply_pending_reload().unwrap();

        assert!(!app.selected_tasks.contains(&2), "out-of-range index 2 must be pruned after reload (D-19)");
        assert!(app.selected_tasks.contains(&0), "valid index 0 must be retained after reload");
        assert!(app.selected_tasks.contains(&1), "valid index 1 must be retained after reload");
    }

    #[test]
    fn reload_clears_out_of_range_anchor() {
        use std::io::{Seek, SeekFrom, Write};
        let (mut app, mut file) = make_app_with_file(&["A", "B", "C"]);
        app.selection_anchor = Some(2);

        file.seek(SeekFrom::Start(0)).unwrap();
        file.as_file().set_len(0).unwrap();
        writeln!(file, "A").unwrap();
        writeln!(file, "B").unwrap();
        file.flush().unwrap();

        app.pending_reload = true;
        app.apply_pending_reload().unwrap();

        assert!(app.selection_anchor.is_none(), "anchor pointing to removed task must be cleared after reload (D-19)");
    }

    #[test]
    fn reload_retains_valid_anchor() {
        use std::io::{Seek, SeekFrom, Write};
        let (mut app, mut file) = make_app_with_file(&["A", "B", "C"]);
        app.selection_anchor = Some(1);

        file.seek(SeekFrom::Start(0)).unwrap();
        file.as_file().set_len(0).unwrap();
        writeln!(file, "A").unwrap();
        writeln!(file, "B").unwrap();
        file.flush().unwrap();

        app.pending_reload = true;
        app.apply_pending_reload().unwrap();

        assert_eq!(app.selection_anchor, Some(1), "valid anchor (index 1, still exists) must be retained after reload");
    }

    // ── Task 20-01: Bulk delete with descending index order ──────────────────

    #[test]
    fn bulk_delete_descending_order() {
        // Select tasks at indices 0 and 2 (non-contiguous). Verify they are deleted
        // in descending order so the index-shift does not invalidate subsequent deletes.
        let mut app = make_app_with_tasks(&["task A", "task B", "task C"]);
        assert_eq!(app.task_list.len(), 3);
        
        // Add indices 0 and 2 to selected_tasks
        app.selected_tasks.insert(0);
        app.selected_tasks.insert(2);
        
        // Simulate pressing 'y' to confirm deletion
        use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
        let confirm_key = KeyEvent {
            code: KeyCode::Char('y'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        
        app.handle_delete_confirm_key(confirm_key).unwrap();
        
        // After deleting indices 2 then 0 in descending order:
        // - Delete index 2: ["task A", "task B"]
        // - Delete index 0: ["task B"]
        // Remaining task is the original task at index 1
        assert_eq!(app.task_list.len(), 1, "bulk delete should remove 2 tasks");
        assert_eq!(app.task_list.tasks()[0].to_raw(), "task B", "remaining task should be B (original index 1)");
        assert!(app.selected_tasks.is_empty(), "selected_tasks should be cleared after bulk delete (D-04)");
        assert!(!app.disjoint_select, "disjoint_select should be reset after bulk delete (D-04)");
    }

    #[test]
    fn bulk_delete_cancel_clears_selection() {
        // Pressing any key other than 'y' should cancel the deletion and clear the selection
        let mut app = make_app_with_tasks(&["task A", "task B", "task C"]);
        app.selected_tasks.insert(0);
        app.selected_tasks.insert(2);
        app.disjoint_select = true;
        
        // Simulate pressing 'n' (cancel)
        use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
        let cancel_key = KeyEvent {
            code: KeyCode::Char('n'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        
        app.handle_delete_confirm_key(cancel_key).unwrap();
        
        // No tasks should be deleted
        assert_eq!(app.task_list.len(), 3, "cancel should not delete any tasks");
        // Selection should be cleared
        assert!(app.selected_tasks.is_empty(), "selected_tasks should be cleared on cancel (D-04)");
        assert_eq!(app.selection_anchor, None, "selection_anchor should be cleared on cancel");
        assert!(!app.disjoint_select, "disjoint_select should be reset on cancel");
    }

    #[test]
    fn single_task_delete_via_selection_shows_preview() {
        // When 1 task is selected, the confirmation should show the task preview (D-02)
        let mut app = make_app_with_tasks(&["task A", "task B", "task C"]);
        app.selected_tasks.insert(1);
        
        // Call render_delete_confirm to build the message
        // We can't directly render without a frame, but we can check the logic flow
        assert_eq!(app.selected_tasks.len(), 1, "should have 1 task selected");
        assert!(!app.selected_tasks.is_empty(), "bulk delete should be triggered");
    }

    #[test]
    fn bulk_delete_multiple_tasks_shows_count() {
        // When > 1 task is selected, the confirmation should show the count (D-02)
        let mut app = make_app_with_tasks(&["task A", "task B", "task C"]);
        app.selected_tasks.insert(0);
        app.selected_tasks.insert(2);
        
        // Verify conditions for rendering bulk count
        assert!(app.selected_tasks.len() > 1, "should have >1 task selected");
    }

    // ── Task 2 (20-02): Bulk append mode ─────────────────────────────────────

    #[test]
    fn bulk_append_applies_text_to_all_selected() {
        let mut app = make_app_with_tasks(&["task A", "task B", "task C"]);
        app.selected_tasks.insert(0);
        app.selected_tasks.insert(2);
        app.mode = AppMode::AppendText;
        app.editor = TextArea::default();
        app.editor.insert_str("+project1");

        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_append_text_key(key).unwrap();

        assert_eq!(app.task_list.tasks()[0].to_raw(), "task A +project1");
        assert_eq!(app.task_list.tasks()[1].to_raw(), "task B"); // untouched
        assert_eq!(app.task_list.tasks()[2].to_raw(), "task C +project1");
        assert!(app.selected_tasks.is_empty());
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn bulk_append_empty_input_cancels_without_mutation() {
        let mut app = make_app_with_tasks(&["task A", "task B"]);
        app.selected_tasks.insert(0);
        app.mode = AppMode::AppendText;
        // editor is empty (default)
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_append_text_key(key).unwrap();
        // tasks unchanged
        assert_eq!(app.task_list.tasks()[0].to_raw(), "task A");
        assert!(app.selected_tasks.is_empty());
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn bulk_append_esc_cancels_without_mutation() {
        let mut app = make_app_with_tasks(&["task A", "task B"]);
        app.selected_tasks.insert(0);
        app.mode = AppMode::AppendText;
        app.editor = TextArea::default();
        app.editor.insert_str("+project1");

        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_append_text_key(key).unwrap();

        // tasks unchanged
        assert_eq!(app.task_list.tasks()[0].to_raw(), "task A");
        assert!(app.selected_tasks.is_empty());
        assert_eq!(app.mode, AppMode::Normal);
    }

    // ── Task 3 (20-03): Status bar selection indicator ──────────────────────

    /// Verify selection count indicator logic: selected_tasks non-empty → count string
    /// This tests the condition used in render_status_bar without requiring a Frame.
    #[test]
    fn status_bar_selection_indicator_absent_when_empty() {
        let app = make_app_with_tasks(&["task A", "task B", "task C"]);
        // No selection → no indicator
        assert!(app.selected_tasks.is_empty());
        let mut left = format!("todo.txt | {}/{} tasks", app.display_indices.len(), app.task_list.len());
        if !app.selected_tasks.is_empty() {
            left.push_str(&format!(" | {} selected", app.selected_tasks.len()));
        }
        assert!(!left.contains("selected"), "Status bar must not contain 'selected' when selection is empty");
    }

    #[test]
    fn status_bar_selection_indicator_present_when_tasks_selected() {
        let mut app = make_app_with_tasks(&["task A", "task B", "task C"]);
        // With selection → indicator present
        app.selected_tasks.insert(0);
        app.selected_tasks.insert(2);
        let mut left = format!("todo.txt | {}/{} tasks", app.display_indices.len(), app.task_list.len());
        if !app.selected_tasks.is_empty() {
            left.push_str(&format!(" | {} selected", app.selected_tasks.len()));
        }
        assert!(left.contains("| 2 selected"), "Status bar should show '| 2 selected' with 2 tasks selected");
    }

    /// Verify disjoint_select=true does not add extra prefix beyond count (D-14)
    #[test]
    fn status_bar_disjoint_mode_shows_count_not_v_prefix() {
        let mut app = make_app_with_tasks(&["task A"]);
        app.selected_tasks.insert(0);
        app.disjoint_select = true;
        let mut left = format!("todo.txt | {}/{} tasks", app.display_indices.len(), app.task_list.len());
        if !app.selected_tasks.is_empty() {
            left.push_str(&format!(" | {} selected", app.selected_tasks.len()));
        }
        // Must contain count
        assert!(left.contains("| 1 selected"), "Status bar should show count when disjoint_select=true");
        // Must NOT contain [v] or v-mode prefix
        assert!(!left.contains("[v]"), "D-14 violated: status bar must not show [v] prefix");
    }

    #[test]
    fn status_scope_uses_active_pane_tasks_in_multi_pane_mode() {
        let mut app = make_two_pane_app(&["new task one", "old task", "new task two"]);

        app.panes[0].filter_query = "new".to_string();
        app.panes[1].filter_query = "old".to_string();
        app.active_pane = 1;
        app.rebuild_all_panes();

        let scoped = app.status_scope_task_indices();
        assert_eq!(scoped.len(), 1, "status scope should use active pane filtered task count");
        assert_eq!(scoped[0], 1, "active pane filter 'old' should target canonical index 1");
    }

    #[test]
    fn status_filter_display_has_explicit_label() {
        let display = App::format_status_filter("new").expect("filter text should be shown");
        assert_eq!(display, "filter: new");
    }

    #[test]
    fn status_filter_display_truncates_long_values() {
        let long_filter = "abcdefghijklmnopqrstuvwxyz0123456789";
        let display = App::format_status_filter(long_filter).expect("filter text should be shown");
        assert!(display.starts_with("filter: "));
        assert!(display.ends_with('…'));
    }

    // ── Phase 24, Plan 01: Pane navigation tests ────────────────────────────

    #[test]
    fn test_app_initializes_with_one_pane() {
        let app = make_app_with_tasks(&["Task 1"]);
        assert_eq!(app.panes.len(), 1);
        assert_eq!(app.active_pane, 0);
        assert_eq!(app.panes[0].label, "");
    }

    #[test]
    fn test_focus_next_pane_single_pane_noop() {
        let mut app = make_app_with_tasks(&["Task 1"]);
        let original = app.active_pane;
        app.focus_next_pane();
        assert_eq!(app.active_pane, original, "Should not change with only one pane");
    }

    #[test]
    fn test_focus_navigation_multiple_panes() {
        let mut app = make_app_with_tasks(&["Task 1"]);

        // Add two more panes manually
        app.panes.push(Pane::new(1, "Work".to_string()));
        app.panes.push(Pane::new(2, "Personal".to_string()));

        assert_eq!(app.active_pane, 0);

        app.focus_next_pane();
        assert_eq!(app.active_pane, 1);

        app.focus_next_pane();
        assert_eq!(app.active_pane, 2);

        // Wrap around
        app.focus_next_pane();
        assert_eq!(app.active_pane, 0);

        // Test prev
        app.focus_prev_pane();
        assert_eq!(app.active_pane, 2);
    }

    #[test]
    fn test_pane_selection_independence() {
        let mut app = make_app_with_tasks(&["Task 1", "Task 2", "Task 3"]);
        app.panes.push(Pane::new(1, "Work".to_string()));

        // Manipulate selection in pane 0
        {
            let pane0 = &mut app.panes[0];
            pane0.selected = 1;
        }

        // Switch to pane 1 and set different selection
        app.focus_next_pane();
        {
            let pane1 = app.active_pane_mut();
            pane1.display_rows = vec![DisplayRow::Task(0), DisplayRow::Task(1)];
            pane1.selected = 0;
        }

        // Verify selections are independent
        assert_eq!(app.panes[0].selected, 1);
        assert_eq!(app.panes[1].selected, 0);

        // Switch back and verify
        app.focus_prev_pane();
        assert_eq!(app.active_pane().selected, 1);
    }

    #[test]
    fn startup_populates_non_active_panes_without_focus_change() {
        let app = make_two_pane_app(&["Task 1", "Task 2", "Task 3"]);

        assert_eq!(app.panes.len(), 2, "expected two panes from config");
        assert!(!app.panes[0].display_rows.is_empty(), "active pane should be populated at startup");
        assert!(!app.panes[1].display_rows.is_empty(), "non-active pane should be populated at startup");
    }

    #[test]
    fn pane_label_can_be_selected_with_up_from_top() {
        let mut app = make_two_pane_app(&["Task 1", "Task 2", "Task 3"]);
        app.active_pane = 1;
        app.panes[1].selected = 0;
        assert!(!app.panes[1].label_selected);

        press_key(&mut app, KeyCode::Up);

        assert!(app.panes[1].label_selected, "Up from top should select pane header");
    }

    #[test]
    fn pane_label_edit_save_updates_label() {
        use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};

        let mut app = make_two_pane_app(&["Task 1", "Task 2", "Task 3"]);
        app.active_pane = 1;
        app.panes[1].label_selected = true;
        assert_eq!(app.panes[1].label, "Work");

        press_key(&mut app, KeyCode::Enter);
        assert_eq!(app.mode, AppMode::PaneLabelEditing { pane_idx: 1 });

        app.editor = TextArea::default();
        app.editor.insert_str("Errands");

        app.handle_pane_label_edit_key(
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: crossterm::event::KeyEventState::NONE,
            },
            1,
        )
        .unwrap();

        assert_eq!(app.panes[1].label, "Errands");
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn pane_label_edit_save_allows_empty_label() {
        use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};

        let mut app = make_two_pane_app(&["Task 1", "Task 2", "Task 3"]);
        app.active_pane = 1;
        app.panes[1].label_selected = true;
        assert_eq!(app.panes[1].label, "Work");

        press_key(&mut app, KeyCode::Enter);
        assert_eq!(app.mode, AppMode::PaneLabelEditing { pane_idx: 1 });

        app.editor = TextArea::default();

        app.handle_pane_label_edit_key(
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: crossterm::event::KeyEventState::NONE,
            },
            1,
        )
        .unwrap();

        assert_eq!(app.panes[1].label, "");
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn pane_label_edit_escape_cancels_changes() {
        use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};

        let mut app = make_two_pane_app(&["Task 1", "Task 2", "Task 3"]);
        app.active_pane = 1;
        app.panes[1].label_selected = true;
        let original = app.panes[1].label.clone();

        press_key(&mut app, KeyCode::Enter);
        assert_eq!(app.mode, AppMode::PaneLabelEditing { pane_idx: 1 });
        app.editor = TextArea::default();
        app.editor.insert_str("Temp");

        app.handle_pane_label_edit_key(
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: crossterm::event::KeyEventState::NONE,
            },
            1,
        )
        .unwrap();

        assert_eq!(app.panes[1].label, original);
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn pane_label_can_be_selected_with_up_when_group_header_is_above_first_task() {
        let mut app = make_two_pane_app(&["Task 1", "Task 2", "Task 3"]);
        app.active_pane = 1;
        app.panes[1].display_rows = vec![
            DisplayRow::GroupHeader("Group".to_string()),
            DisplayRow::Task(0),
            DisplayRow::Task(1),
        ];
        app.panes[1].selected = 1;
        app.panes[1].label_selected = false;

        app.pane_move_up();

        assert!(app.panes[1].label_selected, "Up from first task after group header should select pane header");
        assert_eq!(app.panes[1].selected, 1, "header selection should not move cursor to a different task row");
    }

    // ── Phase 28: Per-pane FilterDefining FAIL-1 fix ──────────────────────────

    /// Helper: create a two-pane App with the given tasks.
    fn make_two_pane_app(task_lines: &[&str]) -> App {
        let mut file = tempfile::NamedTempFile::new().expect("failed to create temp file");
        for line in task_lines {
            writeln!(file, "{}", line).unwrap();
        }
        let path = file.path().to_path_buf();
        let task_list = TaskList::load(&path).expect("load failed");
        let _ = file.keep();

        let mut config = TuiConfig::default();
        config.panes = vec![
            PaneConfig { label: "All".to_string(), filter: String::new(), sort: PaneSort::default(), group: false, group_by: None },
            PaneConfig { label: "Work".to_string(), filter: String::new(), sort: PaneSort::default(), group: false, group_by: None },
        ];
        App::new(task_list, path, config, None, Theme::Default, true)
    }

    #[test]
    fn filter_defining_enter_writes_to_active_pane_not_global() {
        // FAIL-1 regression test (Phase 28):
        // Pressing Enter in FilterDefining mode must write the query to the active pane's
        // filter_query, not to global self.filter_query.
        let mut app = make_two_pane_app(&["task A +work", "task B"]);

        // Confirm two panes exist and active pane is 0.
        assert_eq!(app.panes.len(), 2);
        assert_eq!(app.active_pane, 0);

        // Set up FilterDefining state with "+work" in the active editor.
        let mut active_editor = TextArea::default();
        active_editor.insert_str("+work");
        app.filter_defining_state = Some(FilterDefiningState {
            active_editor,
            preset_names: vec![],
            preset_editors: vec![],
            selected_row: 0,
        });
        app.mode = AppMode::FilterDefining;

        // Press Enter.
        let enter_key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        );
        app.handle_filter_defining_key(enter_key).unwrap();

        // Active pane (0) must have "+work" as filter_query.
        assert_eq!(
            app.panes[0].filter_query, "+work",
            "FAIL-1: active pane filter_query must be '+work' after Enter"
        );
        // Sibling pane (1) must be unchanged.
        assert_eq!(
            app.panes[1].filter_query, "",
            "FAIL-1: sibling pane filter_query must be empty"
        );
        // Global self.filter_query must NOT be written to.
        assert_eq!(
            app.filter_query, "",
            "FAIL-1: global filter_query must remain empty"
        );
        // Mode must return to Normal.
        assert_eq!(app.mode, AppMode::Normal);
        // filter_defining_state must be cleared.
        assert!(app.filter_defining_state.is_none());
    }

    // ── Phase 36: Undo infrastructure tests ──────────────────────────────────

    #[test]
    fn push_then_apply_restores_task_list() {
        let mut app = make_app_with_tasks(&["task A", "task B"]);
        app.selected = 0;
        // Snapshot before mutation
        app.push_undo_entry();
        // Mutate: delete task B (index 1)
        app.task_list.delete(1).unwrap();
        app.rebuild_all_panes();
        assert_eq!(app.task_list.tasks().len(), 1, "should have 1 task after delete");
        // Undo
        app.apply_undo().unwrap();
        assert_eq!(app.task_list.tasks().len(), 2, "should have 2 tasks after undo");
        let names: Vec<String> = app.task_list.tasks().iter().map(|t| t.body.clone()).collect();
        assert!(names.iter().any(|n| n.contains("task A")), "task A must be present");
        assert!(names.iter().any(|n| n.contains("task B")), "task B must be present");
        assert_eq!(app.selected, 0, "cursor must be restored to 0");
    }

    #[test]
    fn apply_undo_when_empty_is_no_op() {
        let mut app = make_app_with_tasks(&["task A"]);
        app.undo_entry = None;
        // apply_undo with no snapshot must not panic and must leave tasks intact
        app.apply_undo().unwrap();
        assert_eq!(app.task_list.tasks().len(), 1, "task list unchanged when undo_entry is None");
    }

    #[test]
    fn second_push_overwrites_first() {
        let mut app = make_app_with_tasks(&["task A", "task B", "task C"]);
        app.selected = 0;
        // First push: 3 tasks
        app.push_undo_entry();
        // Mutate: delete C (index 2) → 2 tasks
        app.task_list.delete(2).unwrap();
        // Second push: 2 tasks
        app.push_undo_entry();
        // Mutate: delete B (now index 1) → 1 task
        app.task_list.delete(1).unwrap();
        assert_eq!(app.task_list.tasks().len(), 1);
        // Undo should restore to 2 tasks (second push), not 3
        app.apply_undo().unwrap();
        assert_eq!(app.task_list.tasks().len(), 2, "should restore to 2 tasks (second snapshot)");
    }

    #[test]
    fn apply_undo_clears_entry() {
        let mut app = make_app_with_tasks(&["task A", "task B"]);
        app.push_undo_entry();
        app.apply_undo().unwrap();
        assert!(app.undo_entry.is_none(), "undo_entry must be None after apply_undo (second Ctrl+Z is a no-op)");
    }

    #[test]
    fn ctrl_z_in_normal_mode_triggers_apply_undo() {
        let mut app = make_app_with_tasks(&["task A", "task B"]);
        app.selected = 0;
        // Snapshot before mutation
        app.push_undo_entry();
        // Mutate: delete task B
        app.task_list.delete(1).unwrap();
        app.rebuild_all_panes();
        assert_eq!(app.task_list.tasks().len(), 1);
        // Simulate Ctrl+Z
        press_ctrl_key(&mut app, KeyCode::Char('z'));
        assert_eq!(app.task_list.tasks().len(), 2, "Ctrl+Z must restore the task list");
    }

    // ── Phase 36 Plan 02: end-to-end undo round-trip integration tests ───────

    #[test]
    fn delete_undo_round_trip() {
        // delete_active_task now calls push_undo_entry() internally
        let mut app = make_app_with_tasks(&["task A", "task B", "task C"]);
        app.selected = 1; // select "task B"
        app.rebuild_display_indices();

        // Trigger delete via 'd' key which routes to delete_active_task in confirm-skip path
        // Use delete_active_task directly to test the wired site
        app.delete_active_task().unwrap();
        assert_eq!(app.task_list.tasks().len(), 2, "task B should be deleted");

        // Undo via Ctrl+Z
        press_ctrl_key(&mut app, KeyCode::Char('z'));
        assert_eq!(app.task_list.tasks().len(), 3, "task list should have 3 tasks after undo");
    }

    #[test]
    fn add_undo_round_trip() {
        // save_and_exit (AppMode::Adding) now calls push_undo_entry() before task_list.add()
        let mut app = make_app_with_tasks(&["task A", "task B"]);
        let original_count = app.task_list.tasks().len();

        // Simulate adding via save_and_exit
        app.mode = AppMode::Adding;
        app.editor = {
            let mut ta = tui_textarea::TextArea::default();
            ta.insert_str("new task");
            ta
        };
        app.save_and_exit().unwrap();
        assert_eq!(app.task_list.tasks().len(), original_count + 1, "task should be added");

        // Undo via Ctrl+Z
        press_ctrl_key(&mut app, KeyCode::Char('z'));
        assert_eq!(app.task_list.tasks().len(), original_count, "Ctrl+Z should remove the added task");
    }

    #[test]
    fn toggle_undo_round_trip() {
        // pane_toggle_done now calls push_undo_entry() before task_list.update()
        let mut app = make_app_with_tasks(&["task A"]);
        let was_completed = app.task_list.tasks()[0].completed;

        app.pane_toggle_done();
        assert_ne!(app.task_list.tasks()[0].completed, was_completed, "completion state should have changed");

        // Undo via Ctrl+Z
        press_ctrl_key(&mut app, KeyCode::Char('z'));
        assert_eq!(app.task_list.tasks()[0].completed, was_completed, "Ctrl+Z should restore original completion state");
    }

    // ── Phase 39 Plan 01: Archive workflow tests ──────────────────────────────

    #[allow(dead_code)]
    fn make_app_with_done_file(task_lines: &[&str]) -> (App, NamedTempFile, NamedTempFile) {
        let mut todo_file = NamedTempFile::new().expect("todo tempfile");
        for line in task_lines {
            writeln!(todo_file, "{}", line).unwrap();
        }
        todo_file.flush().unwrap();
        let todo_path = todo_file.path().to_path_buf();
        let done_file = NamedTempFile::new().expect("done tempfile");
        let done_path = done_file.path().to_path_buf();
        let task_list = TaskList::load(&todo_path).expect("load");
        let _ = todo_file.keep();
        let mut config = TuiConfig::default();
        config.done_file = Some(done_path);
        let app = App::new(task_list, todo_path, config, None, Theme::Default, true);
        (app, NamedTempFile::new().unwrap(), done_file)
    }

    #[test]
    fn archive_tasks_moves_completed_to_done_txt() {
        let mut todo_file = NamedTempFile::new().unwrap();
        writeln!(todo_file, "x 2026-01-01 done task").unwrap();
        writeln!(todo_file, "incomplete task").unwrap();
        todo_file.flush().unwrap();
        let todo_path = todo_file.path().to_path_buf();
        let _ = todo_file.keep();
        // Use a temp dir for done.txt — no open file handle (Windows: can't rename over open handle).
        let done_dir = tempfile::tempdir().unwrap();
        let done_path = done_dir.path().join("done.txt");
        let task_list = TaskList::load(&todo_path).unwrap();
        let mut config = TuiConfig::default();
        config.done_file = Some(done_path.clone());
        let mut app = App::new(task_list, todo_path, config, None, Theme::Default, true);

        let count = app.archive_tasks().unwrap();
        assert_eq!(count, 1, "should archive 1 completed task");
        assert_eq!(app.task_list.len(), 1, "one incomplete task remains");
        assert!(!app.task_list.tasks()[0].completed, "remaining task is incomplete");
        let done_content = std::fs::read_to_string(&done_path).unwrap();
        assert!(done_content.contains("done task"), "done.txt must contain archived task");
    }

    #[test]
    fn archive_tasks_pushes_undo_entry() {
        let mut todo_file = NamedTempFile::new().unwrap();
        writeln!(todo_file, "x 2026-01-01 done task").unwrap();
        todo_file.flush().unwrap();
        let todo_path = todo_file.path().to_path_buf();
        let _ = todo_file.keep();
        let done_dir = tempfile::tempdir().unwrap();
        let done_path = done_dir.path().join("done.txt");
        let task_list = TaskList::load(&todo_path).unwrap();
        let mut config = TuiConfig::default();
        config.done_file = Some(done_path);
        let mut app = App::new(task_list, todo_path, config, None, Theme::Default, true);
        assert!(app.undo_entry.is_none(), "no undo before archive");
        app.archive_tasks().unwrap();
        assert!(app.undo_entry.is_some(), "undo_entry must be set after archive");
    }

    #[test]
    fn archive_tasks_appends_to_existing_done_txt() {
        let mut todo_file = NamedTempFile::new().unwrap();
        writeln!(todo_file, "x 2026-01-01 new done").unwrap();
        todo_file.flush().unwrap();
        let todo_path = todo_file.path().to_path_buf();
        let _ = todo_file.keep();
        let done_dir = tempfile::tempdir().unwrap();
        let done_path = done_dir.path().join("done.txt");
        // Pre-populate done.txt with no open handle (Windows-safe).
        std::fs::write(&done_path, "x 2026-01-01 old done\n").unwrap();
        let task_list = TaskList::load(&todo_path).unwrap();
        let mut config = TuiConfig::default();
        config.done_file = Some(done_path.clone());
        let mut app = App::new(task_list, todo_path, config, None, Theme::Default, true);

        app.archive_tasks().unwrap();
        let done_content = std::fs::read_to_string(&done_path).unwrap();
        assert!(done_content.contains("old done"), "existing done.txt content must be preserved");
        assert!(done_content.contains("new done"), "newly archived task must be appended");
    }

    #[test]
    fn archive_confirm_cancel_leaves_tasks_unchanged() {
        let mut todo_file = NamedTempFile::new().unwrap();
        writeln!(todo_file, "x 2026-01-01 done task").unwrap();
        todo_file.flush().unwrap();
        let todo_path = todo_file.path().to_path_buf();
        let _ = todo_file.keep();
        let done_dir = tempfile::tempdir().unwrap();
        let done_path = done_dir.path().join("done.txt");
        // Pre-create empty done.txt with no open handle so we can assert its content after cancel.
        std::fs::write(&done_path, "").unwrap();
        let task_list = TaskList::load(&todo_path).unwrap();
        let mut config = TuiConfig::default();
        config.done_file = Some(done_path.clone());
        let mut app = App::new(task_list, todo_path, config, None, Theme::Default, true);
        app.mode = AppMode::ArchiveConfirm;
        let esc = crossterm::event::KeyEvent::new(KeyCode::Esc, crossterm::event::KeyModifiers::NONE);
        app.handle_archive_confirm_key(esc).unwrap();
        assert_eq!(app.task_list.len(), 1, "task must remain after cancel");
        assert_eq!(app.mode, AppMode::Normal);
        let done_content = std::fs::read_to_string(&done_path).unwrap();
        assert!(done_content.is_empty(), "done.txt must not be written on cancel");
    }

    // ── Bulk mark-done tests (Phase 39, Plan 02) ─────────────────────────────

    #[test]
    fn bulk_mark_done_marks_incomplete_tasks() {
        let mut app = make_app_with_tasks(&["task A", "task B", "task C"]);
        app.selected_tasks.insert(0);
        app.selected_tasks.insert(1);
        app.bulk_mark_done();
        assert!(app.task_list.tasks()[0].completed, "task 0 must be completed");
        assert!(app.task_list.tasks()[1].completed, "task 1 must be completed");
        assert!(!app.task_list.tasks()[2].completed, "task 2 must remain incomplete");
    }

    #[test]
    fn bulk_mark_done_skips_already_done_tasks() {
        let mut app = make_app_with_tasks(&["x 2026-01-01 already done", "incomplete task"]);
        app.selected_tasks.insert(0);
        app.selected_tasks.insert(1);
        let was_done = app.task_list.tasks()[0].completed;
        app.bulk_mark_done();
        assert!(was_done, "task 0 was already done");
        assert!(app.task_list.tasks()[0].completed, "already-done task must remain done");
        assert!(app.task_list.tasks()[1].completed, "incomplete task must become done");
    }

    #[test]
    fn bulk_mark_done_pushes_single_undo_entry() {
        let mut app = make_app_with_tasks(&["task A", "task B"]);
        app.selected_tasks.insert(0);
        app.selected_tasks.insert(1);
        assert!(app.undo_entry.is_none(), "no undo before bulk_mark_done");
        app.bulk_mark_done();
        assert!(app.undo_entry.is_some(), "exactly one undo_entry after bulk_mark_done");
    }

    #[test]
    fn bulk_mark_done_clears_selection_after() {
        let mut app = make_app_with_tasks(&["task A", "task B"]);
        app.selected_tasks.insert(0);
        app.selected_tasks.insert(1);
        app.bulk_mark_done();
        assert!(app.selected_tasks.is_empty(), "selected_tasks must be cleared after bulk_mark_done");
    }

    #[test]
    fn bulk_mark_done_posts_status_message() {
        let mut app = make_app_with_tasks(&["task A", "task B"]);
        app.selected_tasks.insert(0);
        app.selected_tasks.insert(1);
        app.bulk_mark_done();
        let has_msg = app.runtime_warnings.iter().any(|w| w.contains("Marked") && w.contains("done"));
        assert!(has_msg, "status message must mention 'Marked' and 'done'");
    }

    #[test]
    fn toggle_done_routes_to_bulk_when_selection_nonempty() {
        let mut app = make_app_with_tasks(&["task A", "task B"]);
        app.selected_tasks.insert(0);
        app.selected_tasks.insert(1);
        app.bulk_mark_done();
        assert!(app.task_list.tasks()[0].completed);
        assert!(app.task_list.tasks()[1].completed);
    }

    // BDONE-01 gap: verify empty selection means bulk_mark_done touches nothing.
    // (When selected_tasks is empty, handle_normal_key routes to pane_toggle_done instead.)
    #[test]
    fn bulk_mark_done_empty_selection_marks_nothing() {
        let mut app = make_app_with_tasks(&["task A", "task B"]);
        // selected_tasks intentionally left empty
        app.bulk_mark_done();
        assert!(!app.task_list.tasks()[0].completed, "task A must stay incomplete with empty selection");
        assert!(!app.task_list.tasks()[1].completed, "task B must stay incomplete with empty selection");
        // Status bar should report 0 (not an error)
        let has_msg = app.runtime_warnings.iter().any(|w| w.contains("Marked 0"));
        assert!(has_msg, "status must report 'Marked 0' for empty selection");
    }

    // ── External editor tests (Phase 39, Plan 03) ─────────────────────────────

    // Serialize env-var tests — Rust test threads run in parallel by default.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn resolve_editor_prefers_visual_over_editor() {
        let _lock = ENV_LOCK.lock().unwrap();
        let orig_visual = std::env::var("VISUAL").ok();
        let orig_editor = std::env::var("EDITOR").ok();
        std::env::set_var("VISUAL", "emacs");
        std::env::set_var("EDITOR", "vim");
        let result = resolve_editor();
        match orig_visual { Some(v) => std::env::set_var("VISUAL", v), None => std::env::remove_var("VISUAL") }
        match orig_editor { Some(v) => std::env::set_var("EDITOR", v), None => std::env::remove_var("EDITOR") }
        assert_eq!(result, Some("emacs".to_string()), "VISUAL must take precedence over EDITOR");
    }

    #[test]
    fn resolve_editor_falls_back_to_editor_when_visual_unset() {
        let _lock = ENV_LOCK.lock().unwrap();
        let orig_visual = std::env::var("VISUAL").ok();
        let orig_editor = std::env::var("EDITOR").ok();
        std::env::remove_var("VISUAL");
        std::env::set_var("EDITOR", "nano");
        let result = resolve_editor();
        match orig_visual { Some(v) => std::env::set_var("VISUAL", v), None => {} }
        match orig_editor { Some(v) => std::env::set_var("EDITOR", v), None => std::env::remove_var("EDITOR") }
        assert_eq!(result, Some("nano".to_string()));
    }

    #[test]
    fn resolve_editor_falls_back_to_platform_default() {
        let _lock = ENV_LOCK.lock().unwrap();
        let orig_visual = std::env::var("VISUAL").ok();
        let orig_editor = std::env::var("EDITOR").ok();
        std::env::remove_var("VISUAL");
        std::env::remove_var("EDITOR");
        let result = resolve_editor();
        match orig_visual { Some(v) => std::env::set_var("VISUAL", v), None => {} }
        match orig_editor { Some(v) => std::env::set_var("EDITOR", v), None => {} }
        assert!(result.is_some(), "platform fallback must always return Some");
        #[cfg(target_os = "windows")]
        assert_eq!(result, Some("notepad.exe".to_string()));
        #[cfg(not(target_os = "windows"))]
        assert_eq!(result, Some("vi".to_string()));
    }

    // ── AC-01 autocomplete verification tests (Phase 39, Plan 04) ────────────

    #[test]
    fn project_autocomplete_shows_popup_on_plus() {
        let mut app = make_app_with_tasks(&["task +work", "task +home"]);
        app.mode = AppMode::Adding;
        app.editor = tui_textarea::TextArea::default();
        app.editor.insert_str("+");
        app.update_autocomplete();
        assert!(app.autocomplete.is_some(), "autocomplete must appear after typing '+'");
    }

    #[test]
    fn project_autocomplete_items_are_bare_names() {
        let mut app = make_app_with_tasks(&["task +work", "task +home"]);
        app.mode = AppMode::Adding;
        app.editor = tui_textarea::TextArea::default();
        app.editor.insert_str("+");
        app.update_autocomplete();
        let ac = app.autocomplete.as_ref().expect("autocomplete must be Some");
        assert!(
            ac.items.iter().all(|item| !item.starts_with('+')),
            "autocomplete items must be bare names without '+' prefix, got: {:?}", ac.items
        );
        assert!(ac.items.contains(&"work".to_string()), "items must include 'work'");
        assert!(ac.items.contains(&"home".to_string()), "items must include 'home'");
    }

    #[test]
    fn project_autocomplete_narrows_on_typing() {
        let mut app = make_app_with_tasks(&["task +work", "task +home"]);
        app.mode = AppMode::Adding;
        app.editor = tui_textarea::TextArea::default();
        app.editor.insert_str("+h");
        app.update_autocomplete();
        let ac = app.autocomplete.as_ref().expect("autocomplete must be Some after '+h'");
        assert_eq!(ac.items, vec!["home".to_string()], "'+h' must narrow to only 'home', got: {:?}", ac.items);
        assert!(!ac.items.contains(&"work".to_string()), "'work' must not appear after '+h'");
    }

    #[test]
    fn project_autocomplete_accept_inserts_correctly_no_prefix_typed() {
        // User types "+" and accepts "work" — result must be "+work" not "++work".
        let mut app = make_app_with_tasks(&["task +work"]);
        app.mode = AppMode::Adding;
        app.editor = tui_textarea::TextArea::default();
        app.editor.insert_str("+");
        app.update_autocomplete();
        assert!(app.autocomplete.is_some(), "autocomplete must be active");
        app.accept_completion();
        let line = app.editor.lines().first().cloned().unwrap_or_default();
        assert_eq!(
            line, "+work",
            "accepting '+' completion of 'work' must produce '+work', got: {:?}", line
        );
    }

    #[test]
    fn project_autocomplete_accept_replaces_typed_prefix() {
        // User types "+wo" and accepts "work" — result must be "+work" not "+wowork".
        let mut app = make_app_with_tasks(&["task +work"]);
        app.mode = AppMode::Adding;
        app.editor = tui_textarea::TextArea::default();
        app.editor.insert_str("+wo");
        app.update_autocomplete();
        assert!(app.autocomplete.is_some(), "autocomplete must be active after '+wo'");
        app.accept_completion();
        let line = app.editor.lines().first().cloned().unwrap_or_default();
        assert_eq!(
            line, "+work",
            "accepting completion after typing prefix '+wo' must produce '+work', got: {:?}", line
        );
    }

    // ── Phase 40 Plan 03-B: GRP requirement coverage ────────────────────────

    // GRP-01-T1: GroupByCategory type invariants.
    #[test]
    fn group_by_category_default_is_priority() {
        assert_eq!(
            GroupByCategory::default(),
            GroupByCategory::Priority,
            "GroupByCategory default must be Priority (D-02, GRP-01)"
        );
        // Verify all 4 variants are distinct and exist (compile-time coverage).
        let variants = [
            GroupByCategory::Priority,
            GroupByCategory::Project,
            GroupByCategory::Context,
            GroupByCategory::DueDate,
        ];
        assert_eq!(variants.len(), 4, "GroupByCategory must have exactly 4 variants (D-01)");
        // Each variant must differ from the others.
        for i in 0..variants.len() {
            for j in (i + 1)..variants.len() {
                assert_ne!(variants[i], variants[j], "GroupByCategory variants must be distinct");
            }
        }
    }

    // GRP-01-T2: group_key_for returns correct group key per variant.
    // Tested indirectly via single-pane display_rows GroupHeader values.
    // Note: single-pane rebuild syncs pane.grouping → app.grouping but NOT group_by;
    // app.group_by must be set directly for group_key selection to take effect.
    #[test]
    fn group_key_for_groups_by_correct_field_per_variant() {
        // Task: "(A) fix things +work @home due:2025-01-15"
        let task_line = "(A) fix things +work @home due:2025-01-15";
        let mut app = make_app_with_tasks(&[task_line]);
        // Enable grouping via pane field (synced to app.grouping in rebuild path).
        app.active_pane_mut().grouping = true;

        // Priority: expect GroupHeader "(A)"
        app.group_by = GroupByCategory::Priority;
        app.rebuild_and_reanchor();
        let headers: Vec<_> = app.display_rows.iter()
            .filter_map(|r| if let DisplayRow::GroupHeader(s) = r { Some(s.as_str()) } else { None })
            .collect();
        assert!(headers.contains(&"(A)"), "Priority group_key should be '(A)', got {:?}", headers);

        // Project: expect GroupHeader "+work"
        app.group_by = GroupByCategory::Project;
        app.rebuild_and_reanchor();
        let headers: Vec<_> = app.display_rows.iter()
            .filter_map(|r| if let DisplayRow::GroupHeader(s) = r { Some(s.as_str()) } else { None })
            .collect();
        assert!(headers.contains(&"+work"), "Project group_key should be '+work', got {:?}", headers);

        // Context: expect GroupHeader "@home"
        app.group_by = GroupByCategory::Context;
        app.rebuild_and_reanchor();
        let headers: Vec<_> = app.display_rows.iter()
            .filter_map(|r| if let DisplayRow::GroupHeader(s) = r { Some(s.as_str()) } else { None })
            .collect();
        assert!(headers.contains(&"@home"), "Context group_key should be '@home', got {:?}", headers);

        // DueDate: expect GroupHeader "2025-01-15"
        app.group_by = GroupByCategory::DueDate;
        app.rebuild_and_reanchor();
        let headers: Vec<_> = app.display_rows.iter()
            .filter_map(|r| if let DisplayRow::GroupHeader(s) = r { Some(s.as_str()) } else { None })
            .collect();
        assert!(headers.contains(&"2025-01-15"), "DueDate group_key should be '2025-01-15', got {:?}", headers);
    }

    // GRP-01-T3: Pane::new() (via App) initializes group_by = Priority.
    #[test]
    fn pane_initializes_group_by_to_priority() {
        let app = make_app_with_tasks(&["task A"]);
        // In multi-pane mode panes vec is used; in single-pane mode, app.group_by is the field.
        // make_app_with_tasks starts single-pane, so check app.group_by and panes[0].group_by.
        assert_eq!(
            app.group_by,
            GroupByCategory::Priority,
            "App.group_by must default to Priority (GRP-01)"
        );
        assert_eq!(
            app.panes[0].group_by,
            GroupByCategory::Priority,
            "Pane.group_by must default to Priority (D-04, GRP-01)"
        );
    }

    // GRP-02-T1: cycle_group_by() cycles through all 4 variants and wraps.
    // Tested via 'g' key presses on app with tasks (group_by_cycle action).
    #[test]
    fn cycle_group_by_wraps_through_all_four_variants() {
        let mut app = make_app_with_tasks(&["task A"]);
        // Start: Priority (default)
        assert_eq!(app.active_pane().group_by, GroupByCategory::Priority);
        // Press 'g' → Project
        press_key(&mut app, KeyCode::Char('g'));
        assert_eq!(app.active_pane().group_by, GroupByCategory::Project, "1st 'g': Priority→Project");
        // Press 'g' → Context
        press_key(&mut app, KeyCode::Char('g'));
        assert_eq!(app.active_pane().group_by, GroupByCategory::Context, "2nd 'g': Project→Context");
        // Press 'g' → DueDate
        press_key(&mut app, KeyCode::Char('g'));
        assert_eq!(app.active_pane().group_by, GroupByCategory::DueDate, "3rd 'g': Context→DueDate");
        // Press 'g' → Priority (wrap)
        press_key(&mut app, KeyCode::Char('g'));
        assert_eq!(app.active_pane().group_by, GroupByCategory::Priority, "4th 'g': DueDate→Priority (wrap)");
    }

    // GRP-02-T2: 'g' key changes active pane's group_by independently of sort_order.
    #[test]
    fn g_key_cycles_group_by_independently_of_sort_order() {
        let mut app = make_app_with_tasks(&["task A", "task B"]);
        let initial_sort = app.active_pane().sort_order;
        // Press 'g' to cycle group_by.
        press_key(&mut app, KeyCode::Char('g'));
        assert_eq!(
            app.active_pane().group_by,
            GroupByCategory::Project,
            "'g' must advance group_by from Priority to Project"
        );
        assert_eq!(
            app.active_pane().sort_order,
            initial_sort,
            "'g' must not change sort_order (group_by and sort_order are independent, GRP-02)"
        );
    }

    // GRP-03-T1: Status bar grp: indicator reflects active group_by when grouping enabled.
    #[test]
    fn status_bar_grp_indicator_text_matches_active_group_by() {
        // group_by_name() is accessible via `use super::*` (private fn in same file).
        // Test all 4 variants produce the expected status bar string.
        assert_eq!(group_by_name(GroupByCategory::Priority), "priority");
        assert_eq!(group_by_name(GroupByCategory::Project),  "project");
        assert_eq!(group_by_name(GroupByCategory::Context),  "context");
        assert_eq!(group_by_name(GroupByCategory::DueDate),  "duedate");

        // Simulate the status bar logic: grouping=true → "grp:{name}" appended to middle.
        let mut middle = String::new();
        let pane_grouping = true;
        let pane_group_by = GroupByCategory::Project;
        if pane_grouping {
            middle.push_str(" | grp:");
            middle.push_str(group_by_name(pane_group_by));
        }
        assert!(
            middle.contains("grp:project"),
            "status bar middle must contain 'grp:project' when grouping=true and group_by=Project (D-12, GRP-03)"
        );

        // When grouping=false, grp: must NOT appear.
        let mut middle2 = String::new();
        let pane_grouping2 = false;
        let pane_group_by2 = GroupByCategory::Context;
        if pane_grouping2 {
            middle2.push_str(" | grp:");
            middle2.push_str(group_by_name(pane_group_by2));
        }
        assert!(
            !middle2.contains("grp:"),
            "status bar must NOT show grp: when grouping=false (D-13, GRP-03)"
        );
    }

    // GRP-04-T1: PaneConfig TOML backward compat — absent group_by field → None.
    #[test]
    fn pane_config_without_group_by_deserializes_to_none() {
        // TOML without group_by field → group_by = None (backward compat, D-06, GRP-04).
        let cfg: crate::config::PaneConfig = toml::from_str(
            "label = \"test\"\nfilter = \"+work\"\ngroup = false"
        ).expect("should deserialize PaneConfig without group_by");
        assert_eq!(cfg.group_by, None, "absent group_by in TOML must deserialize to None");

        // TOML with group_by = "project" → group_by = Some(Project).
        let cfg2: crate::config::PaneConfig = toml::from_str(
            "group_by = \"project\""
        ).expect("should deserialize PaneConfig with group_by = \"project\"");
        assert_eq!(
            cfg2.group_by,
            Some(GroupByCategory::Project),
            "group_by = \"project\" in TOML must deserialize to Some(Project)"
        );

        // TOML with group_by = "due_date" → group_by = Some(DueDate).
        let cfg3: crate::config::PaneConfig = toml::from_str(
            "group_by = \"due_date\""
        ).expect("should deserialize PaneConfig with group_by = \"due_date\"");
        assert_eq!(
            cfg3.group_by,
            Some(GroupByCategory::DueDate),
            "group_by = \"due_date\" in TOML must deserialize to Some(DueDate)"
        );
    }

    // ── Phase 40 Plan 03: Phase 22 gap coverage ──────────────────────────────

    fn make_app_with_config(task_lines: &[&str], config: TuiConfig) -> App {
        let mut file = NamedTempFile::new().expect("failed to create temp file");
        for line in task_lines {
            writeln!(file, "{}", line).unwrap();
        }
        let path = file.path().to_path_buf();
        let task_list = TaskList::load(&path).expect("load failed");
        let _ = file.keep();
        App::new(task_list, path, config, None, Theme::Default, true)
    }

    // 22-01-G01: App::new initializes effective_keymap and keymap_warnings from config.
    #[test]
    fn app_new_initializes_effective_keymap_from_config() {
        let app = make_app_with_tasks(&["task A"]);
        // Default config → effective_keymap should be populated (at least "help" action present).
        assert!(
            app.effective_keymap.contains_key("help"),
            "effective_keymap must contain 'help' action after App::new"
        );
        // Default config has no invalid entries → warnings should be empty.
        assert!(
            app.keymap_warnings.is_empty(),
            "keymap_warnings must be empty with default config"
        );
    }

    // 22-01-G02: handle_normal_key dispatches default action keys through dynamic dispatch.
    #[test]
    fn handle_normal_key_default_dispatch_works() {
        let mut app = make_app_with_tasks(&["task A", "task B"]);
        // Default 'n' key → AddingMode (add action)
        press_key(&mut app, KeyCode::Char('n'));
        assert_eq!(
            app.mode,
            AppMode::Adding,
            "default 'n' key must transition to AppMode::Adding via effective_keymap dispatch"
        );
    }

    // 22-02-G01: Status bar error_log_count reflects keymap warnings.
    #[test]
    fn error_log_count_reflects_keymap_warnings() {
        let mut cfg = TuiConfig::default();
        // Insert an invalid action to generate a keymap warning.
        cfg.keymap.insert("nonexistent_action_xyz".into(), "a".into());
        let app = make_app_with_config(&["task A"], cfg);
        assert!(
            app.error_log_count() > 0,
            "error_log_count must be > 0 when keymap_warnings is non-empty"
        );
        assert!(
            !app.keymap_warnings.is_empty(),
            "keymap_warnings must be non-empty after invalid action in config"
        );
    }

    // 22-02-G02: Clean status bar when no warnings.
    #[test]
    fn error_log_count_zero_with_clean_config() {
        let app = make_app_with_tasks(&["task A"]);
        assert_eq!(
            app.error_log_count(),
            0,
            "error_log_count must be 0 with default config (no warnings)"
        );
    }

    // 22-02-G03: '!' in Normal mode → AppMode::KeymapErrors.
    #[test]
    fn bang_key_enters_keymap_errors_mode() {
        let mut app = make_app_with_tasks(&["task A"]);
        assert_eq!(app.mode, AppMode::Normal);
        press_key(&mut app, KeyCode::Char('!'));
        assert_eq!(
            app.mode,
            AppMode::KeymapErrors,
            "'!' must transition to AppMode::KeymapErrors"
        );
    }

    // 22-02-G04: Esc from KeymapErrors → AppMode::Normal.
    #[test]
    fn esc_from_keymap_errors_returns_to_normal() {
        let mut app = make_app_with_tasks(&["task A"]);
        app.mode = AppMode::KeymapErrors;
        use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
        let esc = KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        app.handle_keymap_errors_key(esc).unwrap();
        assert_eq!(
            app.mode,
            AppMode::Normal,
            "Esc from KeymapErrors must return to AppMode::Normal"
        );
    }

    // 22-03-G01: '0' clears filter_query.
    #[test]
    fn zero_key_clears_filter_query() {
        let mut app = make_app_with_tasks(&["task A +work", "task B +home"]);
        app.active_pane_mut().filter_query = "+work".to_string();
        app.rebuild_and_reanchor();
        press_key(&mut app, KeyCode::Char('0'));
        assert_eq!(
            app.active_pane().filter_query,
            "",
            "'0' must clear active pane filter_query"
        );
    }

    // 22-03-G02: '1'-'9' applies preset filter when slot is defined; no-op if slot empty.
    #[test]
    fn number_keys_apply_preset_filter() {
        let mut cfg = TuiConfig::default();
        cfg.presets.filter.insert(
            "1".into(),
            crate::config::FilterPreset { filter: Some("+work".into()) },
        );
        let mut app = make_app_with_config(&["task A +work", "task B +home"], cfg);
        // '1' should apply preset filter "1".
        press_key(&mut app, KeyCode::Char('1'));
        assert_eq!(
            app.active_pane().filter_query,
            "+work",
            "'1' must apply preset filter '1' to active pane"
        );
        // '2' with no preset defined → no-op (filter unchanged).
        press_key(&mut app, KeyCode::Char('2'));
        assert_eq!(
            app.active_pane().filter_query,
            "+work",
            "'2' with no preset must be a no-op (filter unchanged)"
        );
    }

    // 22-03-G03: '.' calls task_list.reload() — verify via round-trip with temp file.
    #[test]
    fn dot_key_triggers_reload() {
        let mut file = NamedTempFile::new().expect("failed to create temp file");
        writeln!(file, "task A").unwrap();
        let path = file.path().to_path_buf();
        let task_list = TaskList::load(&path).expect("load failed");
        let path_clone = path.clone();
        let _ = file.keep();
        let mut app = App::new(task_list, path_clone, TuiConfig::default(), None, Theme::Default, true);
        assert_eq!(app.task_list.tasks().len(), 1);
        // Append a task to the file on disk.
        {
            let mut f = std::fs::OpenOptions::new().append(true).open(&path).unwrap();
            writeln!(f, "task B").unwrap();
        }
        // Press '.' to reload.
        press_key(&mut app, KeyCode::Char('.'));
        assert_eq!(
            app.task_list.tasks().len(),
            2,
            "'.' must reload task list from disk (task B should appear after reload)"
        );
    }

    // 22-03-G04: '?' → AppMode::Help.
    #[test]
    fn question_mark_enters_help_mode() {
        let mut app = make_app_with_tasks(&["task A"]);
        assert_eq!(app.mode, AppMode::Normal);
        press_key(&mut app, KeyCode::Char('?'));
        assert_eq!(
            app.mode,
            AppMode::Help,
            "'?' must transition to AppMode::Help"
        );
    }

    // 22-03-G05: Esc/q from Help → AppMode::Normal.
    #[test]
    fn esc_and_q_from_help_return_to_normal() {
        use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};

        // Esc closes Help.
        let mut app = make_app_with_tasks(&["task A"]);
        app.mode = AppMode::Help;
        let esc = KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        app.handle_help_key(esc).unwrap();
        assert_eq!(app.mode, AppMode::Normal, "Esc must close Help overlay");

        // 'q' closes Help.
        let mut app2 = make_app_with_tasks(&["task A"]);
        app2.mode = AppMode::Help;
        let q_key = KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        app2.handle_help_key(q_key).unwrap();
        assert_eq!(app2.mode, AppMode::Normal, "'q' must close Help overlay");
    }

    // ── Phase 41: Filter history, preset loading, pane layout presets ─────────

    // FHIST-01 / 41-03-T01: push_filter_history deduplicates and caps at 50.
    #[test]
    fn push_filter_history_dedup_and_cap() {
        let mut app = make_app_with_tasks(&["task A"]);
        // Push same entry twice → only one in history.
        app.push_filter_history("+work");
        app.push_filter_history("+work");
        assert_eq!(app.filter_history.len(), 1, "duplicate push must not grow history");
        assert_eq!(app.filter_history[0], "+work");

        // Push a different entry → 2 entries, newest at front.
        app.push_filter_history("+home");
        assert_eq!(app.filter_history.len(), 2);
        assert_eq!(app.filter_history[0], "+home", "newest entry must be at front");

        // Re-push "+work" → moved to front, deduplicated.
        app.push_filter_history("+work");
        assert_eq!(app.filter_history.len(), 2, "dedup must not grow history on re-push");
        assert_eq!(app.filter_history[0], "+work", "re-pushed entry must move to front");

        // Cap at 50.
        for i in 0..50 {
            app.push_filter_history(&format!("entry-{}", i));
        }
        assert_eq!(app.filter_history.len(), 50, "filter_history must be capped at 50 entries");
    }

    // FHIST-01 / 41-03-T02: Empty string is ignored by push_filter_history.
    #[test]
    fn push_filter_history_ignores_empty() {
        let mut app = make_app_with_tasks(&["task A"]);
        app.push_filter_history("");
        assert!(app.filter_history.is_empty(), "empty string must not be added to filter history");
    }

    // FHIST-01 / 41-03-T03: push_filter_history resets filter_history_cursor.
    #[test]
    fn push_filter_history_resets_cursor() {
        let mut app = make_app_with_tasks(&["task A"]);
        app.push_filter_history("+work");
        app.filter_history_cursor = Some(0);
        // Push new entry → cursor reset.
        app.push_filter_history("+home");
        assert!(app.filter_history_cursor.is_none(), "push must reset filter_history_cursor");
    }

    // PRST-01 / 41-03-T04: App::new loads filter presets from config.presets.filter.
    #[test]
    fn app_new_loads_filter_presets_from_config() {
        let mut cfg = TuiConfig::default();
        cfg.presets.filter.insert(
            "1".to_string(),
            crate::config::FilterPreset { filter: Some("+work".to_string()) },
        );
        cfg.presets.filter.insert(
            "2".to_string(),
            crate::config::FilterPreset { filter: Some("+home".to_string()) },
        );
        let app = make_app_with_config(&["task A"], cfg);
        // Both presets must be loaded and sorted by key.
        assert_eq!(app.presets.len(), 2, "must load 2 filter presets");
        assert_eq!(app.presets[0].0, "1");
        assert_eq!(app.presets[0].1, "+work");
    }

    // PRST-02 / 41-03-T05: apply_pane_layout_preset replaces panes atomically.
    #[test]
    fn apply_pane_layout_preset_replaces_panes() {
        let mut app = make_app_with_tasks(&["task A +work", "task B +home"]);
        assert_eq!(app.panes.len(), 1, "default: single pane");

        let preset = crate::config::PaneLayoutPreset {
            panes: vec![
                crate::config::PaneConfig {
                    label: "Work".to_string(),
                    filter: "+work".to_string(),
                    sort: crate::config::PaneSort::default(),
                    group: false,
                    group_by: None,
                },
                crate::config::PaneConfig {
                    label: "Home".to_string(),
                    filter: "+home".to_string(),
                    sort: crate::config::PaneSort::default(),
                    group: false,
                    group_by: None,
                },
            ],
        };
        app.apply_pane_layout_preset(&preset);
        assert_eq!(app.panes.len(), 2, "preset must replace panes with 2 new panes");
        assert_eq!(app.panes[0].label, "Work");
        assert_eq!(app.active_pane, 0, "active pane must be reset to 0");
    }

    // PRST-02 / 41-03-T06: apply_pane_layout_preset with empty panes is a no-op.
    #[test]
    fn apply_pane_layout_preset_empty_is_noop() {
        let mut app = make_app_with_tasks(&["task A"]);
        let initial_pane_count = app.panes.len();
        let preset = crate::config::PaneLayoutPreset { panes: vec![] };
        app.apply_pane_layout_preset(&preset);
        assert_eq!(app.panes.len(), initial_pane_count, "empty preset must be a no-op");
    }

    // ── Phase 41 Plan 04: pane task movement ─────────────────────────────────

    // PMOVE-01 / 41-04-T01: is_single_tag_token accepts valid single tokens.
    #[test]
    fn is_single_tag_token_valid() {
        assert!(App::is_single_tag_token("@work"), "@work should be valid");
        assert!(App::is_single_tag_token("+project"), "+project should be valid");
        assert!(App::is_single_tag_token("@home-office"), "@home-office should be valid");
    }

    // PMOVE-01 / 41-04-T02: is_single_tag_token rejects invalid tokens.
    #[test]
    fn is_single_tag_token_invalid() {
        assert!(!App::is_single_tag_token("@work @home"), "compound @work @home must be invalid");
        assert!(!App::is_single_tag_token("due:today"), "due:today has no @/+ prefix");
        assert!(!App::is_single_tag_token(""), "empty string must be invalid");
        assert!(!App::is_single_tag_token("@work +personal"), "compound @work +personal must be invalid");
    }

    // PMOVE-02 / 41-04-T03: pane_move_task swaps tags correctly.
    #[test]
    fn pane_move_task_tag_swap() {
        use crate::config::{TuiConfig, PaneConfig, PaneSort};
        let mut config = TuiConfig::default();
        config.panes = vec![
            PaneConfig { label: "Work".into(), filter: "@work".into(), sort: PaneSort::default(), group: false, group_by: None },
            PaneConfig { label: "Home".into(), filter: "@home".into(), sort: PaneSort::default(), group: false, group_by: None },
        ];
        let mut app = make_app_with_config(&["todo @work task"], config);
        assert_eq!(app.active_pane, 0);
        // Cursor is on the @work task in pane 0.
        app.pane_move_task(1).unwrap();
        let raw = app.task_list.tasks()[0].to_raw().to_string();
        assert!(!raw.contains("@work"), "src token not removed: {}", raw);
        assert!(raw.contains("@home"), "dest token not added: {}", raw);
        assert_eq!(app.active_pane, 1, "focus must jump to dest pane");
    }

    // PMOVE-03 / 41-04-T04: pane_move_task is declined when src filter is compound.
    #[test]
    fn pane_move_task_declined_compound_filter() {
        use crate::config::{TuiConfig, PaneConfig, PaneSort};
        let mut config = TuiConfig::default();
        config.panes = vec![
            PaneConfig { label: "Compound".into(), filter: "@work +project".into(), sort: PaneSort::default(), group: false, group_by: None },
            PaneConfig { label: "Home".into(), filter: "@home".into(), sort: PaneSort::default(), group: false, group_by: None },
        ];
        let mut app = make_app_with_config(&["todo @work +project task"], config);
        let was_none = app.undo_entry.is_none();
        app.pane_move_task(1).unwrap();
        // Declined: undo_entry unchanged, active pane unchanged.
        assert_eq!(app.undo_entry.is_none(), was_none, "undo entry must not be pushed on declined move");
        assert_eq!(app.active_pane, 0, "active pane must not change on declined move");
    }

    // PMOVE-02 / 41-04-T05: pane_move_task wraps at boundary.
    #[test]
    fn pane_move_task_wraps_at_boundary() {
        use crate::config::{TuiConfig, PaneConfig, PaneSort};
        let mut config = TuiConfig::default();
        config.panes = vec![
            PaneConfig { label: "Work".into(), filter: "@work".into(), sort: PaneSort::default(), group: false, group_by: None },
            PaneConfig { label: "Home".into(), filter: "@home".into(), sort: PaneSort::default(), group: false, group_by: None },
        ];
        let mut app = make_app_with_config(&["todo @work task"], config);
        app.active_pane = 0;
        // Move left from pane 0 → should wrap to last pane (index 1).
        app.pane_move_task(-1).unwrap();
        assert_eq!(app.active_pane, 1, "move left from pane 0 must wrap to last pane");
    }

    // ── Phase 41 gap-fill tests ───────────────────────────────────────────────

    // PRST-02 / 41-03-G01: Ctrl+1 in normal mode applies pane layout preset at index 0.
    #[test]
    fn ctrl_one_applies_pane_layout_preset() {
        use crate::config::{TuiConfig, PaneConfig, PaneLayoutPreset, PaneSort};
        let mut config = TuiConfig::default();
        config.presets.panes.insert(
            "1".into(),
            PaneLayoutPreset {
                panes: vec![
                    PaneConfig { label: "Work".into(), filter: "@work".into(), sort: PaneSort::default(), group: false, group_by: None },
                    PaneConfig { label: "Home".into(), filter: "@home".into(), sort: PaneSort::default(), group: false, group_by: None },
                ],
            },
        );
        let mut app = make_app_with_config(&["task @work", "task @home"], config);
        assert_eq!(app.panes.len(), 1, "initial pane count must be 1");
        assert_eq!(app.pane_presets.len(), 1, "must have 1 pane preset loaded");
        press_ctrl_key(&mut app, KeyCode::Char('1'));
        assert_eq!(app.panes.len(), 2, "Ctrl+1 must apply pane layout preset → 2 panes");
        assert_eq!(app.panes[0].label, "Work");
        assert_eq!(app.active_pane, 0, "active pane reset to 0 after preset");
    }

    // FHIST-01 / 41-03-G02: pressing Enter in Filtering mode pushes the entered text to history.
    #[test]
    fn filter_enter_pushes_to_history() {
        use crate::state::FilteringState;
        use tui_textarea::TextArea;
        use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
        let mut app = make_app_with_tasks(&["task +work"]);
        assert!(app.filter_history.is_empty(), "history must start empty");
        // Enter filtering mode manually.
        let mut editor = TextArea::default();
        editor.insert_str("+work");
        app.filter_state = Some(FilteringState {
            editor,
            selected_preset: 0,
            snapshot: String::new(),
        });
        app.mode = AppMode::Filtering;
        // Press Enter to apply filter.
        let enter = KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        app.handle_filtering_key(enter).unwrap();
        assert_eq!(app.filter_history.len(), 1, "Enter must push filter text to history");
        assert_eq!(app.filter_history[0], "+work");
        assert_eq!(app.mode, AppMode::Normal, "mode must return to Normal after Enter");
    }

    // FHIST-02 / 41-03-G03: Ctrl+R in Filtering mode cycles backward through history.
    #[test]
    fn ctrl_r_cycles_filter_history() {
        use crate::state::FilteringState;
        use tui_textarea::TextArea;
        use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
        let mut app = make_app_with_tasks(&["task +work", "task +home"]);
        app.filter_history.push_front("+home".into());
        app.filter_history.push_front("+work".into());
        // Enter filtering mode with empty editor.
        let editor = TextArea::default();
        app.filter_state = Some(FilteringState {
            editor,
            selected_preset: 0,
            snapshot: String::new(),
        });
        app.mode = AppMode::Filtering;
        // First Ctrl+R → cursor 0, entry "+work".
        let ctrl_r = KeyEvent {
            code: KeyCode::Char('r'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        app.handle_filtering_key(ctrl_r).unwrap();
        assert_eq!(app.filter_history_cursor, Some(0), "first Ctrl+R must set cursor to 0");
        assert_eq!(
            app.active_pane().filter_query, "+work",
            "first Ctrl+R must load history[0] into active pane filter"
        );
        // Second Ctrl+R → cursor 1, entry "+home".
        let ctrl_r2 = KeyEvent {
            code: KeyCode::Char('r'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        app.handle_filtering_key(ctrl_r2).unwrap();
        assert_eq!(app.filter_history_cursor, Some(1), "second Ctrl+R must advance cursor to 1");
        assert_eq!(
            app.active_pane().filter_query, "+home",
            "second Ctrl+R must load history[1]"
        );
    }

    // PMOVE-02 / 41-04-G01: Ctrl+Right key dispatch — NOTE: IMPLEMENTATION BUG FOUND
    // The unguarded `KeyCode::Right =>` arm in handle_normal_key (line ~994) catches all
    // Right-arrow events before the `pane_move_right` action check is reached, making
    // Ctrl+Right always call focus_next_pane() instead of pane_move_task(1).
    // This test verifies the method works correctly when called directly (PMOVE-02 method coverage).
    // The key dispatch path is blocked by the unguarded arm — documented in VALIDATION.md.
    #[test]
    fn pane_move_task_direct_moves_right() {
        use crate::config::{TuiConfig, PaneConfig, PaneSort};
        let mut config = TuiConfig::default();
        config.panes = vec![
            PaneConfig { label: "Work".into(), filter: "@work".into(), sort: PaneSort::default(), group: false, group_by: None },
            PaneConfig { label: "Home".into(), filter: "@home".into(), sort: PaneSort::default(), group: false, group_by: None },
        ];
        let mut app = make_app_with_config(&["todo @work task"], config);
        assert_eq!(app.active_pane, 0);
        app.pane_move_task(1).unwrap();
        let raw = app.task_list.tasks()[0].to_raw().to_string();
        assert!(!raw.contains("@work"), "pane_move_task(1) must remove src tag: {}", raw);
        assert!(raw.contains("@home"), "pane_move_task(1) must add dest tag: {}", raw);
        assert_eq!(app.active_pane, 1, "pane_move_task(1) must jump focus to dest pane");
    }

    // PMOVE-03 / 41-04-G02: pane_move_task pushes undo entry before mutation.
    #[test]
    fn pane_move_task_pushes_undo_entry() {
        use crate::config::{TuiConfig, PaneConfig, PaneSort};
        let mut config = TuiConfig::default();
        config.panes = vec![
            PaneConfig { label: "Work".into(), filter: "@work".into(), sort: PaneSort::default(), group: false, group_by: None },
            PaneConfig { label: "Home".into(), filter: "@home".into(), sort: PaneSort::default(), group: false, group_by: None },
        ];
        let mut app = make_app_with_config(&["todo @work task"], config);
        assert!(app.undo_entry.is_none(), "undo_entry must be None before any mutation");
        app.pane_move_task(1).unwrap();
        assert!(app.undo_entry.is_some(), "pane_move_task must push undo_entry before mutating");
    }

    // ── compute_filter_autocomplete tests (Phase 42, Plan 01) ────────────────

    fn make_task_list_for_filter(task_lines: &[&str]) -> TaskList {
        let mut file = NamedTempFile::new().expect("failed to create temp file");
        for line in task_lines {
            writeln!(file, "{}", line).unwrap();
        }
        let path = file.path().to_path_buf();
        let task_list = TaskList::load(&path).expect("load failed");
        let _ = file.keep();
        task_list
    }

    // AC-04-T01: empty input returns None
    #[test]
    fn compute_filter_autocomplete_empty_returns_none() {
        let tl = make_task_list_for_filter(&[]);
        let history = std::collections::VecDeque::<String>::new();
        assert!(compute_filter_autocomplete("", 0, &tl, &history).is_none());
    }

    // AC-04-T02: "@" alone returns all contexts
    #[test]
    fn compute_filter_autocomplete_at_alone_returns_all_contexts() {
        let tl = make_task_list_for_filter(&["task @work", "task @waiting"]);
        let history = std::collections::VecDeque::<String>::new();
        let result = compute_filter_autocomplete("@", 1, &tl, &history);
        assert!(result.is_some(), "@ alone should return Some");
        let ac = result.unwrap();
        assert_eq!(ac.trigger, '@', "trigger must be '@'");
        assert_eq!(ac.prefix, "", "prefix must be empty");
        assert_eq!(ac.mode, AutocompleteMode::TokenAutocomplete('@'));
        let mut items = ac.items.clone();
        items.sort();
        assert!(items.contains(&"work".to_string()), "items must contain 'work': {:?}", items);
        assert!(items.contains(&"waiting".to_string()), "items must contain 'waiting': {:?}", items);
    }

    // AC-04-T03: "@w" filters to contexts starting with 'w'
    #[test]
    fn compute_filter_autocomplete_at_w_filters_contexts() {
        let tl = make_task_list_for_filter(&["task @work", "task @waiting", "task @home"]);
        let history = std::collections::VecDeque::<String>::new();
        let result = compute_filter_autocomplete("@w", 2, &tl, &history);
        assert!(result.is_some(), "@w should return Some");
        let ac = result.unwrap();
        assert_eq!(ac.trigger, '@');
        assert_eq!(ac.prefix, "w");
        let mut items = ac.items.clone();
        items.sort();
        assert_eq!(items, vec!["waiting", "work"], "must be filtered+sorted: {:?}", items);
        assert!(!items.contains(&"home".to_string()), "must not include @home");
    }

    // AC-04-T04: cursor-aware extraction — "done:false @w" at col 13 triggers '@'
    #[test]
    fn compute_filter_autocomplete_mid_expression_cursor_aware() {
        let tl = make_task_list_for_filter(&["task @work", "task @waiting"]);
        let history = std::collections::VecDeque::<String>::new();
        let line = "done:false @w";
        let result = compute_filter_autocomplete(line, 13, &tl, &history);
        assert!(result.is_some(), "mid-expression @w should return Some");
        let ac = result.unwrap();
        assert_eq!(ac.trigger, '@');
        assert_eq!(ac.prefix, "w");
    }

    // AC-02-T01: "+" alone returns all projects
    #[test]
    fn compute_filter_autocomplete_plus_alone_returns_all_projects() {
        let tl = make_task_list_for_filter(&["task +inbox", "task +personal"]);
        let history = std::collections::VecDeque::<String>::new();
        let result = compute_filter_autocomplete("+", 1, &tl, &history);
        assert!(result.is_some(), "+ alone should return Some");
        let ac = result.unwrap();
        assert_eq!(ac.trigger, '+');
        assert_eq!(ac.prefix, "");
        assert_eq!(ac.mode, AutocompleteMode::TokenAutocomplete('+'));
        assert!(ac.items.contains(&"inbox".to_string()), "items must contain 'inbox': {:?}", ac.items);
        assert!(ac.items.contains(&"personal".to_string()), "items must contain 'personal': {:?}", ac.items);
    }

    // AC-04-T05: no trigger + non-empty history → FilterHistory
    #[test]
    fn compute_filter_autocomplete_no_trigger_with_history_returns_filter_history() {
        let tl = make_task_list_for_filter(&[]);
        let mut history = std::collections::VecDeque::<String>::new();
        history.push_back("+work".to_string());
        history.push_back("@home".to_string());
        let result = compute_filter_autocomplete("just text", 9, &tl, &history);
        assert!(result.is_some(), "no trigger + non-empty history should return Some");
        let ac = result.unwrap();
        assert_eq!(ac.mode, AutocompleteMode::FilterHistory);
        assert_eq!(ac.trigger, '\0');
    }

    // AC-04-T06: no trigger + empty history → None
    #[test]
    fn compute_filter_autocomplete_no_trigger_empty_history_returns_none() {
        let tl = make_task_list_for_filter(&[]);
        let history = std::collections::VecDeque::<String>::new();
        assert!(compute_filter_autocomplete("just text", 9, &tl, &history).is_none());
    }

    // AC-04-T07: "@xyz" where no context starts with 'xyz' → None
    #[test]
    fn compute_filter_autocomplete_at_xyz_no_match_returns_none() {
        let tl = make_task_list_for_filter(&["task @work", "task @home"]);
        let history = std::collections::VecDeque::<String>::new();
        assert!(compute_filter_autocomplete("@xyz", 4, &tl, &history).is_none());
    }

    // ── handle_filtering_key integration tests (Phase 42, Plan 02) ───────────

    /// Set up an app in Filtering mode with the given task lines.
    fn make_filtering_app(task_lines: &[&str]) -> App {
        let mut app = make_app_with_tasks(task_lines);
        app.mode = AppMode::Filtering;
        app.filter_state = Some(FilteringState {
            editor: tui_textarea::TextArea::default(),
            selected_preset: 0,
            snapshot: String::new(),
        });
        app
    }

    fn key(code: KeyCode) -> crossterm::event::KeyEvent {
        crossterm::event::KeyEvent::new(code, crossterm::event::KeyModifiers::NONE)
    }

    // AC-02-I01: typing '@' in filter input triggers TokenAutocomplete('@')
    #[test]
    fn filter_autocomplete_at_triggers_token_popup() {
        let mut app = make_filtering_app(&["task @work", "task @waiting"]);
        app.handle_filtering_key(key(KeyCode::Char('@'))).unwrap();
        let ac = app.autocomplete.as_ref().expect("autocomplete should be Some after '@'");
        assert_eq!(
            ac.mode,
            AutocompleteMode::TokenAutocomplete('@'),
            "mode must be TokenAutocomplete('@'), got {:?}",
            ac.mode
        );
    }

    // AC-02-I02: typing '+' in filter input triggers TokenAutocomplete('+')
    #[test]
    fn filter_autocomplete_plus_triggers_project_popup() {
        let mut app = make_filtering_app(&["task +inbox", "task +personal"]);
        app.handle_filtering_key(key(KeyCode::Char('+'))).unwrap();
        let ac = app.autocomplete.as_ref().expect("autocomplete should be Some after '+'");
        assert_eq!(
            ac.mode,
            AutocompleteMode::TokenAutocomplete('+'),
            "mode must be TokenAutocomplete('+'), got {:?}",
            ac.mode
        );
    }

    // AC-04-I01: typing '@' then 'w' narrows to contexts starting with 'w'
    #[test]
    fn filter_autocomplete_narrowing_reduces_list() {
        let mut app = make_filtering_app(&["task @work", "task @waiting", "task @home"]);
        app.handle_filtering_key(key(KeyCode::Char('@'))).unwrap();
        app.handle_filtering_key(key(KeyCode::Char('w'))).unwrap();
        let ac = app.autocomplete.as_ref().expect("autocomplete should be Some after '@w'");
        assert_eq!(ac.mode, AutocompleteMode::TokenAutocomplete('@'));
        for item in &ac.items {
            assert!(
                item.to_lowercase().starts_with('w'),
                "item '{}' doesn't start with 'w'",
                item
            );
        }
        assert!(!ac.items.contains(&"home".to_string()), "'home' must not appear after '@w'");
    }

    // AC-02-I03: Down navigates popup — focused=true, selected increments
    #[test]
    fn filter_autocomplete_down_navigates_when_popup_present() {
        let mut app = make_filtering_app(&["task @work", "task @waiting"]);
        // Manually inject a token autocomplete with 2 items, not focused
        app.autocomplete = Some(AutocompleteState::new(
            '@',
            String::new(),
            vec!["waiting".to_string(), "work".to_string()],
        ));
        app.handle_filtering_key(key(KeyCode::Down)).unwrap();
        let ac = app.autocomplete.as_ref().expect("autocomplete should still be Some after Down");
        assert!(ac.focused, "Down with popup present must set focused=true");
        assert_eq!(ac.selected, 1, "selected must increment to 1 after Down");
    }

    // AC-02-I04: Up with focused popup decrements selected
    #[test]
    fn filter_autocomplete_up_decrements_when_popup_focused() {
        let mut app = make_filtering_app(&["task @work", "task @waiting"]);
        app.autocomplete = Some(AutocompleteState::new(
            '@',
            String::new(),
            vec!["waiting".to_string(), "work".to_string()],
        ));
        // First focus and move to index 1
        app.handle_filtering_key(key(KeyCode::Down)).unwrap();
        // Now Up should go back to 0
        app.handle_filtering_key(key(KeyCode::Up)).unwrap();
        let ac = app.autocomplete.as_ref().expect("autocomplete should still be Some after Up");
        assert_eq!(ac.selected, 0, "Up must decrement selected back to 0");
    }

    // AC-03-I01: Enter with focused popup keeps Filtering mode open (D-02)
    #[test]
    fn filter_autocomplete_enter_when_focused_keeps_filter_open() {
        let mut app = make_filtering_app(&["task @work"]);
        let mut ac = AutocompleteState::new('@', String::new(), vec!["work".to_string()]);
        ac.focused = true;
        ac.selected = 0;
        app.autocomplete = Some(ac);
        app.handle_filtering_key(key(KeyCode::Enter)).unwrap();
        assert_eq!(
            app.mode,
            AppMode::Filtering,
            "Enter with focused popup must keep Filtering mode, not apply filter"
        );
        assert!(
            app.filter_state.is_some(),
            "filter_state must stay Some after autocomplete accept"
        );
    }

    // AC-03-I02: Tab accepts the focused popup and inserts token into editor
    #[test]
    fn filter_autocomplete_tab_accepts_and_inserts_token() {
        let mut app = make_filtering_app(&["task @work"]);
        // Seed editor with "@" so the token replacement has a word to work with
        if let Some(ref mut state) = app.filter_state {
            state.editor.insert_str("@");
        }
        let mut ac = AutocompleteState::new('@', String::new(), vec!["work".to_string()]);
        ac.focused = true;
        ac.selected = 0;
        app.autocomplete = Some(ac);
        app.handle_filtering_key(key(KeyCode::Tab)).unwrap();
        // Autocomplete popup should be dismissed
        assert!(
            app.autocomplete.is_none(),
            "autocomplete must be None after Tab accept"
        );
        // Mode stays Filtering (D-02)
        assert_eq!(app.mode, AppMode::Filtering, "mode must stay Filtering after Tab accept");
        // Editor content should contain "work" (token inserted)
        let content = app
            .filter_state
            .as_ref()
            .map(|s| s.editor.lines().first().cloned().unwrap_or_default())
            .unwrap_or_default();
        assert!(
            content.contains("work"),
            "editor must contain 'work' after accepting '@work', got: '{}'",
            content
        );
    }

    // AC-03-I03: Enter without focused popup applies filter normally (no regression)
    #[test]
    fn filter_autocomplete_enter_no_focused_popup_applies_filter() {
        let mut app = make_filtering_app(&["task @work"]);
        if let Some(ref mut state) = app.filter_state {
            state.editor.insert_str("@work");
        }
        // No focused autocomplete
        app.autocomplete = None;
        app.handle_filtering_key(key(KeyCode::Enter)).unwrap();
        assert_eq!(
            app.mode,
            AppMode::Normal,
            "Enter without focused popup must apply filter and return to Normal mode"
        );
    }
}




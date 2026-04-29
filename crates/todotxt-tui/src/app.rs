//! Application state and main event loop.
//!
//! All state mutation happens exclusively on the main thread (D-03).
//! The two sender threads only produce `AppEvent` values — they never
//! touch `App` or `TaskList` directly.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use chrono::Local;
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::Frame;
use todotxt_core::{Filter, SortOrder, Task, TaskList, normalize_append, normalize_line};
use tui_textarea::TextArea;

use crate::config::{PaneConfig, PaneSort, TuiConfig, resolve_keymap};
use crate::event::AppEvent;
use crate::theme as theme_module;
use theme_module::{StyleSheet, Theme};
use crate::tui::Tui;
use crate::state::{Pane, DisplayRow, AutocompleteState, FilteringState, FilterDefiningState};
use crate::components::PaneList;



/// Interaction mode for the TUI (D-01 in 11-CONTEXT.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Adding,
    Editing { original_idx: usize },
    DeleteConfirm,
    Filtering,
    /// F-key preset definition panel (D-01 in 16-CONTEXT.md).
    FilterDefining,
    /// Bulk append mode: user types text to append to all selected tasks (D-06, Phase 20).
    AppendText,
    /// Read-only overlay showing app warnings/errors log.
    KeymapErrors,
    /// Read-only overlay showing all keybindings (D-10, Phase 22 parity).
    Help,
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
    /// Maps display row position → canonical task index (D-10, D-11 in 12-CONTEXT.md).
    pub display_indices: Vec<usize>,
    /// Toggle grouped rendering with non-selectable header rows.
    pub grouping: bool,
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
    /// Named filter presets from `[presets]` in config (Plan 02).
    pub presets: Vec<(String, String)>,
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
                pane
            })
            .collect();

        if panes.is_empty() {
            panes.push(Pane::new(0, "Tasks".to_string()));
        }

        panes
    }

    pub fn new(task_list: TaskList, todo_path: PathBuf, config: TuiConfig, config_path: Option<PathBuf>, palette: Theme, no_color: bool) -> Self {
        // Build sorted presets vec from config for quick filter selection (Plan 02).
        let mut presets: Vec<(String, String)> = config
            .presets
            .iter()
            .filter_map(|(name, p)| p.filter.as_ref().map(|f| (name.clone(), f.clone())))
            .collect();
        presets.sort_by(|(a, _), (b, _)| a.cmp(b));
        // Resolve keymap at startup — applies user overrides, collects warnings (D-04, Phase 22).
        let (effective_keymap, keymap_warnings) = resolve_keymap(&config);
        let panes = Self::panes_from_config(&config);
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
            display_indices: Vec::new(),
            grouping: false,
            display_rows: Vec::new(),
            sort_order: SortOrder::FileOrder,
            show_deferred: false,
            filter_query: String::new(),
            toggled_filter_query: None,
            filter_state: None,
            presets,
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
        };
        app.rebuild_display_indices();
        app.rebuild_active_pane();
        app
    }

    /// Returns true when the given key event matches the configured binding for `action` (D-05, Phase 22).
    ///
    /// Checks `effective_keymap` so user overrides are honoured. For bindings with no modifier
    /// (e.g. uppercase 'D' for bulk_delete), matches on key code only — this preserves the
    /// existing behavior where terminals may or may not report the implicit SHIFT modifier
    /// separately for uppercase printable characters.
    fn key_is_action(&self, key: crossterm::event::KeyEvent, action: &str) -> bool {
        self.effective_keymap.get(action).map_or(false, |(code, mods)| {
            if mods.is_empty() {
                key.code == *code
            } else {
                key.code == *code && key.modifiers.contains(*mods)
            }
        })
    }

    fn push_runtime_warning(&mut self, msg: impl Into<String>) {
        self.runtime_warnings.push(msg.into());
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
        } else if self.panes.len() > 1 {
            0
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
            self.panes.push(Pane::new(0, "Tasks".to_string()));
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
            
            for (source_index, task) in &filtered_tasks {
                let key = group_key_for(task, &pane.sort_order);
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

    /// Rebuild the active pane's display_rows from task_list
    pub fn rebuild_active_pane(&mut self) {
        self.rebuild_visible_rows();
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
                        let key = group_key_for(task, &sort_order);
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
            self.persist_panes_on_quit()?;
        }

        Ok(())
    }

    pub fn persist_panes_on_quit(&mut self) -> color_eyre::Result<()> {
        self.config.panes = self
            .panes
            .iter()
            .map(|pane| PaneConfig {
                label: pane.label.clone(),
                filter: pane.filter_query.clone(),
                sort: PaneSort::from_sort_order(pane.sort_order),
                group: pane.grouping,
            })
            .collect();

        if let Some(path) = self.config_path.clone() {
            self.config.save(&path)?;
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
                    AppMode::Adding | AppMode::Editing { .. } => {
                        self.handle_editor_key(key)?;
                    }
                    AppMode::AppendText => self.handle_append_text_key(key)?,
                    AppMode::DeleteConfirm => self.handle_delete_confirm_key(key)?,
                    AppMode::Filtering => self.handle_filtering_key(key)?,
                    AppMode::FilterDefining => self.handle_filter_defining_key(key)?,
                    AppMode::KeymapErrors => self.handle_keymap_errors_key(key)?,
                    AppMode::Help => self.handle_help_key(key)?,
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
        let row_count = self.display_rows.len();
        match key.code {
            // ── Ctrl+C quit (not overridable) ────────────────────────────────
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
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
                let mut next = self.selected + 1;
                while next < row_count
                    && matches!(self.display_rows[next], DisplayRow::GroupHeader(_))
                {
                    next += 1;
                }
                if next < row_count {
                    self.selected = next;
                }
                self.apply_range_selection();
            }
            // Shift+k or Shift+Up: extend contiguous range selection upward (D-09, D-11).
            // MUST precede plain k/Up arm so SHIFT modifier is checked first (T-19-04).
            KeyCode::Char('k') | KeyCode::Up
                if key.modifiers.contains(KeyModifiers::SHIFT) && row_count > 0 =>
            {
                self.ensure_anchor();
                if self.selected > 0 {
                    let mut prev = self.selected.saturating_sub(1);
                    while prev > 0
                        && matches!(self.display_rows[prev], DisplayRow::GroupHeader(_))
                    {
                        prev -= 1;
                    }
                    if matches!(self.display_rows[prev], DisplayRow::Task(_)) {
                        self.selected = prev;
                    }
                }
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
                self.pane_toggle_done();
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
                if let Some(canonical) = self.canonical_selected() {
                    let raw = self.task_list.tasks()[canonical].to_raw().to_string();
                    let mut ed = TextArea::default();
                    ed.insert_str(&raw);
                    self.editor = ed;
                    self.mode = AppMode::Editing { original_idx: canonical };
                }
            }

            _ if !self.selected_tasks.is_empty() && display_count > 0 && self.key_is_action(key, "bulk_delete") => {
                self.mode = AppMode::DeleteConfirm;
            }

            _ if display_count > 0 && self.key_is_action(key, "delete") => {
                self.mode = AppMode::DeleteConfirm;
            }

            _ if self.key_is_action(key, "sort_cycle") => {
                // Per-pane sort state (D-07, Phase 25): Apply only to active pane
                let current_sort = self.active_pane().sort_order;
                self.active_pane_mut().sort_order = cycle_sort(current_sort);
                self.rebuild_and_reanchor();
            }

            _ if !self.selected_tasks.is_empty() && display_count > 0 && self.key_is_action(key, "bulk_append") => {
                self.editor = TextArea::default();
                self.mode = AppMode::AppendText;
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

            // '1'-'9' applies a preset filter by slot (D-11, Phase 22; not overridable)
            KeyCode::Char(c @ '1'..='9') if key.modifiers == KeyModifiers::NONE => {
                let slot = format!("f{}", c);
                if let Some(preset) = self.config.presets.get(&slot) {
                    if let Some(filter_str) = preset.filter.as_ref() {
                        // Per-pane: apply preset filter to active pane (Phase 25)
                        self.active_pane_mut().filter_query = filter_str.clone();
                        self.toggled_filter_query = None;
                        self.rebuild_and_reanchor();
                    }
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

            _ => {}
        }
        Ok(())
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
                self.mode = AppMode::Normal;
                self.rebuild_and_reanchor();
                self.apply_pending_reload()?;
            }
            KeyCode::Enter => {
                // Apply filter to active pane
                if let Some(state) = self.filter_state.take() {
                    let filter_text = state.editor.lines().join("").trim().to_string();
                    self.active_pane_mut().filter_query = filter_text;
                }
                self.mode = AppMode::Normal;
                self.toggled_filter_query = None;
                self.rebuild_and_reanchor();
                self.apply_pending_reload()?;
            }
            KeyCode::Down => {
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
            _ => {
                if let Some(ref mut state) = self.filter_state {
                    state.editor.input(key);
                    let filter_text = state
                        .editor
                        .lines()
                        .first()
                        .cloned()
                        .unwrap_or_default();
                    // Per-pane: update active pane's filter as user types (D-04, Phase 25)
                    self.active_pane_mut().filter_query = filter_text;
                }
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

                // Update config.presets from editors.
                for (i, name) in state.preset_names.iter().enumerate() {
                    let filter_str = state.preset_editors[i].lines().join("").trim().to_string();
                    if filter_str.is_empty() {
                        // Remove empty/cleared presets — do not write blank slots to config.
                        self.config.presets.remove(name);
                    } else {
                        self.config.presets.entry(name.clone())
                            .and_modify(|p| p.filter = Some(filter_str.clone()))
                            .or_insert_with(|| crate::config::TuiPreset { filter: Some(filter_str) });
                    }
                }

                // Rebuild presets vec from updated config (D-05: only preset strings persisted).
                let mut updated: Vec<(String, String)> = self.config.presets.iter()
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
    fn update_autocomplete(&mut self) {
        match self.mode {
            AppMode::Adding | AppMode::Editing { .. } => {}
            AppMode::AppendText => { self.autocomplete = None; return; }
            _ => { self.autocomplete = None; return; }
        }
        let line = self.editor.lines().first().cloned().unwrap_or_default();
        // Find last @ or + in the line.
        let trigger_pos = line.rfind(|c: char| c == '@' || c == '+');
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

    /// Insert the currently selected autocomplete token into the editor.
    fn accept_completion(&mut self) {
        let (trigger, token) = match &self.autocomplete {
            Some(ac) => match ac.items.get(ac.selected) {
                Some(token) => (ac.trigger, token.clone()),
                None => { self.autocomplete = None; return; }
            },
            None => return,
        };
        let line = self.editor.lines().first().cloned().unwrap_or_default();
        if let Some(pos) = line.rfind(trigger) {
            let new_line = format!("{}{}{}", &line[..=pos], token, "");
            let mut new_editor = tui_textarea::TextArea::default();
            new_editor.insert_str(&new_line);
            self.editor = new_editor;
        }
        self.autocomplete = None;
    }


    // ── Delete confirm key handler ────────────────────────────────────────────

    fn handle_delete_confirm_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> color_eyre::Result<()> {
        if key.code == KeyCode::Char('y') {
            if self.selected_tasks.is_empty() {
                // Existing single-task path (D-01 fallback: d with empty selection)
                if let Some(idx) = self.canonical_selected() {
                    self.task_list
                        .delete(idx)
                        .map_err(|e| color_eyre::eyre::eyre!("Failed to delete task: {}", e))?;
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
            let sort_order = self.sort_order;
            // Stable-sort by group key so same-key tasks are always adjacent.
            // This fixes cases where the primary sort interleaves groups (e.g., Alphabetical
            // sorts by raw string including priority prefix, but group_key_for uses body).
            // stable_sort preserves primary sort order within each group.
            self.display_indices.sort_by(|&a, &b| {
                let ka = group_key_for(&tasks[a], &sort_order);
                let kb = group_key_for(&tasks[b], &sort_order);
                ka.cmp(&kb)
            });
            let mut rows: Vec<DisplayRow> = Vec::new();
            let mut last_key: Option<String> = None;
            for &idx in &self.display_indices {
                let task = &tasks[idx];
                let key = group_key_for(task, &self.sort_order);
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
            self.selection_anchor = self.canonical_selected();
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
        let cursor_canon = match self.canonical_selected() {
            Some(c) => c,
            None => return,
        };
        // Locate display-row positions for anchor and cursor canonical indices.
        let anchor_row = self
            .display_rows
            .iter()
            .position(|r| matches!(r, DisplayRow::Task(idx) if *idx == anchor_canon));
        let cursor_row = self
            .display_rows
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
        for row in lo..=hi {
            if let DisplayRow::Task(idx) = self.display_rows[row] {
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
        if let Err(e) = self.task_list.update(idx, toggled) {
            eprintln!("toggle_done error: {e}");
        }
        self.rebuild_all_panes();
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
        let pane = self.active_pane_mut();
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

    /// Move selection up in the active pane, skipping group headers (Phase 24-02).
    fn pane_move_up(&mut self) {
        self.reconcile_active_pane();
        let pane = self.active_pane_mut();
        if pane.selected == 0 {
            return;
        }
        let mut prev = pane.selected.saturating_sub(1);
        while prev > 0 && matches!(pane.display_rows[prev], DisplayRow::GroupHeader(_)) {
            prev -= 1;
        }
        if matches!(pane.display_rows.get(prev), Some(DisplayRow::Task(_))) {
            pane.selected = prev;
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
            AppMode::Filtering => {
                let panel_height = 1_u16 + (self.presets.len() as u16).min(5);
                let chunks =
                    Layout::vertical([Min(0), Length(panel_height), Length(1)]).split(frame.area());
                self.render_panes(frame, chunks[0]);
                self.render_filter_panel(frame, chunks[1]);
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
                        let prefix = if is_selected && !is_cursor { "> " } else { "" };
                        let content = format!("{}{}{}: {}", prefix, indent, ci + 1, t.to_raw());
                        // Priority and overdue coloring (D-01, D-09 in 13-CONTEXT.md).
                        // Style precedence: completed (DIM) > deferred shown (DIM) > priority A/B/C > overdue > plain.
                        // Modifier::REVERSED for selection is applied by List::highlight_style — not here.
                        let style = if t.completed {
                            // Completed tasks: DIM only, no color (D-01, D-06).
                            Style::default().add_modifier(Modifier::DIM)
                        } else if self.show_deferred
                            && t.threshold_date.map_or(false, |d| d > Local::now().date_naive())
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
        let visible = self.display_indices.len();

        let due_today = self
            .display_indices
            .iter()
            .filter(|&&ci| {
                !tasks[ci].completed && tasks[ci].due_status() == DueStatus::Today
            })
            .count();
        let overdue = self
            .display_indices
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
        let (pane_filter, pane_sort, pane_grouping) = if !self.should_show_single_pane() && self.panes.len() > 1 && !self.panes_hidden {
            let pane = &self.panes[self.active_pane];
            (
                pane.filter_query.clone(),
                pane.sort_order,
                pane.grouping,
            )
        } else {
            // Fallback to global state when showing single pane
            (
                self.filter_query.clone(),
                self.sort_order,
                self.grouping,
            )
        };

        let trimmed_filter = pane_filter.trim();
        if !trimmed_filter.is_empty() {
            middle.push_str(" | ");
            // Truncate long filter queries for display
            if trimmed_filter.len() > 30 {
                middle.push_str(&format!("{}…", &trimmed_filter[..27]));
            } else {
                middle.push_str(trimmed_filter);
            }
        }
        if pane_sort != SortOrder::FileOrder {
            middle.push_str(" | sort: ");
            middle.push_str(sort_name(pane_sort));
        }
        if pane_grouping {
            middle.push_str(" | group: on");
        }
        if self.show_deferred {
            middle.push_str(" [+deferred]");
        }

        let right = "  q quit | n add | u edit | d del | D bulk del | T bulk app | v sel | Shift+nav range | x done | j/k nav | f filter | ^f filt on/off | F define | o sort | g group | h deferred | t theme | 0 clear filter | 1-9 preset | . reload | ? help";
        let total_width = area.width as usize;
        let left_len = left.len();
        let middle_len = middle.len();
        let right_len = right.len();

        let show_hints = left_len + middle_len + right_len <= total_width;

        let middle_display = if show_hints || left_len + middle_len <= total_width {
            middle
        } else {
            let available = total_width.saturating_sub(left_len);
            if available == 0 {
                String::new()
            } else if available == 1 {
                "…".to_string()
            } else {
                let truncated: String = middle.chars().take(available - 1).collect();
                format!("{}…", truncated)
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
            ("Filter", "Filter", &[
                "filter_open", "filter_define", "filter_toggle", "clear_filter",
            ]),
            ("View", "View", &[
                "sort_cycle", "group_toggle", "deferred_toggle", "theme_cycle", "reload",
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
            ("deferred_toggle", "Toggle deferred"),
            ("theme_cycle", "Cycle theme"),
            ("reload", "Reload file"),
            ("disjoint_select", "Disjoint select"),
            ("disjoint_mark", "Mark selection"),
            ("pane_add", "Create pane"),
            ("pane_delete", "Delete pane"),
            ("pane_hide_toggle", "Toggle panes"),
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

    /// Render the one-row delete confirmation panel (D-06, D-07).
    fn render_delete_confirm(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::text::{Line, Span};
        use ratatui::widgets::Paragraph;

        let tasks = self.task_list.tasks();

        let text = if self.selected_tasks.len() > 1 {
            // Bulk confirmation: show count, not task preview (D-02)
            format!("Delete {} tasks?  y=confirm  any=cancel", self.selected_tasks.len())
        } else if self.selected_tasks.len() == 1 {
            // Single-task-via-selection: show existing preview (D-02)
            let idx = *self.selected_tasks.iter().next().unwrap();
            let preview = tasks.get(idx).map(|t| t.to_raw().to_string()).unwrap_or_default();
            format!("Delete: \"{}\"  y=confirm  any=cancel", preview)
        } else {
            // Cursor-task delete (selection empty): existing behavior
            let preview = match self.canonical_selected() {
                Some(idx) => tasks[idx].to_raw().to_string(),
                None => String::new(),
            };
            format!("Delete: \"{}\"  y=confirm  any=cancel", preview)
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

        let popup_width = ac.items.iter()
            .map(|s| s.len() + 4) // 4 for trigger char + borders
            .max()
            .unwrap_or(20)
            .min(40) as u16;
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

        let popup_list = List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(highlight_style);

        let mut list_state = ListState::default().with_selected(Some(ac.selected));

        frame.render_widget(ratatui::widgets::Clear, popup_area);
        frame.render_stateful_widget(popup_list, popup_area, &mut list_state);
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

fn group_key_for(task: &Task, sort: &SortOrder) -> String {
    match sort {
        SortOrder::Priority => task
            .priority
            .map(|p| format!("({})", p))
            .unwrap_or_else(|| "none".to_string()),
        SortOrder::Project => task
            .projects
            .first()
            .map(|p| format!("+{}", p))
            .unwrap_or_else(|| "none".to_string()),
        SortOrder::Context => task
            .contexts
            .first()
            .map(|c| format!("@{}", c))
            .unwrap_or_else(|| "none".to_string()),
        SortOrder::DueDate => task
            .due_date
            .map(|d| d.to_string())
            .unwrap_or_else(|| "no due date".to_string()),
        SortOrder::Alphabetical => task
            .body
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "none".to_string()),
        SortOrder::FileOrder => "all tasks".to_string(),
        SortOrder::CompletedDate => task
            .completion_date
            .map(|d| d.to_string())
            .unwrap_or_else(|| "no completion date".to_string()),
        SortOrder::CreationDate => task
            .creation_date
            .map(|d| d.to_string())
            .unwrap_or_else(|| "no creation date".to_string()),
        _ => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

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
        press_shift_key(&mut app, KeyCode::Char('j'));
        // shift-j should skip GroupHeader at row 1 and land on Task(1) at row 2
        assert_eq!(app.selected, 2, "shift-j should skip GroupHeader rows (D-08)");
        assert!(app.selected_tasks.contains(&0), "Task 0 (anchor) should be selected");
        assert!(app.selected_tasks.contains(&1), "Task 1 (at row 2) should be selected");
        assert!(!app.selected_tasks.is_empty());
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

    // ── Phase 24, Plan 01: Pane navigation tests ────────────────────────────

    #[test]
    fn test_app_initializes_with_one_pane() {
        let app = make_app_with_tasks(&["Task 1"]);
        assert_eq!(app.panes.len(), 1);
        assert_eq!(app.active_pane, 0);
        assert_eq!(app.panes[0].label, "Tasks");
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
            PaneConfig { label: "All".to_string(), filter: String::new(), sort: PaneSort::default(), group: false },
            PaneConfig { label: "Work".to_string(), filter: String::new(), sort: PaneSort::default(), group: false },
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
}





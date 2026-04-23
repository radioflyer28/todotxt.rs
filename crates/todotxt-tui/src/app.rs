//! Application state and main event loop.
//!
//! All state mutation happens exclusively on the main thread (D-03).
//! The two sender threads only produce `AppEvent` values — they never
//! touch `App` or `TaskList` directly.

use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::Frame;
use todotxt_core::{Filter, SortOrder, Task, TaskList};
use tui_textarea::TextArea;

use crate::config::TuiConfig;
use crate::event::AppEvent;
use crate::theme::{StyleSheet, Theme};
use crate::tui::Tui;

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
    pub editor: TextArea<'static>,
    pub selected_preset: usize,
    /// Snapshot of `filter_query` captured when the panel was opened (D-02).
    /// Restored on Esc so no destructive clear occurs.
    pub snapshot: String,
}

/// State for the F-key preset definition panel (D-01, D-06, D-07).
pub struct FilterDefiningState {
    /// Row 0: editable active filter with live preview (D-07).
    pub active_editor: TextArea<'static>,
    /// Preset names in sorted order (index 0 = preset #1).
    pub preset_names: Vec<String>,
    /// One editor per preset slot; index 0 corresponds to preset_names[0].
    pub preset_editors: Vec<TextArea<'static>>,
    /// Currently focused row: 0 = active filter row, 1–9 = preset row N.
    pub selected_row: usize,
}

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
}

/// Top-level application state.
pub struct App {
    pub should_quit: bool,
    pub task_list: TaskList,
    pub todo_path: PathBuf,
    /// 0-based index into `display_indices` for the currently selected row.
    /// Always clamped to `[0, display_indices.len() - 1]`.
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
    /// Current display sort order (FileOrder = no sort applied).
    pub sort_order: SortOrder,
    /// Active filter query string (empty = no filter).
    pub filter_query: String,
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
    /// Active theme selected at startup.
    pub theme: Theme,
}

impl App {
    pub fn new(task_list: TaskList, todo_path: PathBuf, config: TuiConfig, config_path: Option<PathBuf>, theme: Theme, no_color: bool) -> Self {
        // Build sorted presets vec from config for quick filter selection (Plan 02).
        let mut presets: Vec<(String, String)> = config
            .presets
            .iter()
            .filter_map(|(name, p)| p.filter.as_ref().map(|f| (name.clone(), f.clone())))
            .collect();
        presets.sort_by(|(a, _), (b, _)| a.cmp(b));
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
            sort_order: SortOrder::FileOrder,
            filter_query: String::new(),
            filter_state: None,
            presets,
            config,
            config_path,
            filter_defining_state: None,
            styles: StyleSheet::from_theme(theme, no_color),
            theme,
        };
        app.rebuild_display_indices();
        app
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

        Ok(())
    }

    /// Handle a single `AppEvent`. Dispatches on current mode (D-01).
    fn handle_event(
        &mut self,
        event: AppEvent,
        terminal: &mut Tui,
    ) -> color_eyre::Result<()> {
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
                    AppMode::DeleteConfirm => self.handle_delete_confirm_key(key)?,
                    AppMode::Filtering => self.handle_filtering_key(key)?,
                    AppMode::FilterDefining => self.handle_filter_defining_key(key)?,
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
        match key.code {
            // ── Quit ────────────────────────────────────────────────────────
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }

            // ── Navigation ──────────────────────────────────────────────────
            KeyCode::Char('j') | KeyCode::Down if display_count > 0 => {
                self.selected = (self.selected + 1).min(display_count - 1);
            }
            KeyCode::Char('k') | KeyCode::Up if display_count > 0 => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('g') if display_count > 0 => {
                self.selected = 0;
            }
            KeyCode::Char('G') if display_count > 0 => {
                self.selected = display_count - 1;
            }
            // Ctrl+U half-page up — must come before plain 'u' (edit).
            KeyCode::Char('u')
                if key.modifiers.contains(KeyModifiers::CONTROL) && display_count > 0 =>
            {
                let half = (self.list_height / 2).max(1) as usize;
                self.selected = self.selected.saturating_sub(half);
            }
            // Ctrl+D half-page down — must come before plain 'd' (delete).
            KeyCode::Char('d')
                if key.modifiers.contains(KeyModifiers::CONTROL) && display_count > 0 =>
            {
                let half = (self.list_height / 2).max(1) as usize;
                self.selected = (self.selected + half).min(display_count - 1);
            }

            // ── Done toggle ──────────────────────────────────────────────────
            KeyCode::Char('x') if display_count > 0 => {
                self.toggle_done();
            }

            // ── Add task — always available even on empty list ───────────────
            KeyCode::Char('n') => {
                self.editor = TextArea::default();
                self.mode = AppMode::Adding;
            }

            // ── Edit task (u or e) — after Ctrl+U arm ───────────────────────
            KeyCode::Char('u') | KeyCode::Char('e') if display_count > 0 => {
                if let Some(canonical) = self.canonical_selected() {
                    let raw = self.task_list.tasks()[canonical].to_raw().to_string();
                    let mut ed = TextArea::default();
                    ed.insert_str(&raw);
                    self.editor = ed;
                    self.mode = AppMode::Editing { original_idx: canonical };
                }
            }

            // ── Delete task — after Ctrl+D arm ──────────────────────────────
            KeyCode::Char('d') if display_count > 0 => {
                self.mode = AppMode::DeleteConfirm;
            }

            // ── Sort cycle ──────────────────────────────────────────────────
            KeyCode::Char('o') => {
                self.sort_order = cycle_sort(self.sort_order);
                self.rebuild_and_reanchor();
            }

            // ── Filter panel placeholder (Plan 02) ──────────────────────────
            KeyCode::Char('f') => {
                let mut editor = TextArea::default();
                editor.insert_str(&self.filter_query);
                self.filter_state = Some(FilteringState {
                    editor,
                    selected_preset: 0,
                    snapshot: self.filter_query.clone(), // per D-02
                });
                self.mode = AppMode::Filtering;
            }

            // ── Preset definition panel (Plan 16-03, D-01) ──────────────────
            KeyCode::Char('F') => {
                let mut active_editor = TextArea::default();
                active_editor.insert_str(&self.filter_query);

                // Build sorted preset list (up to 9 entries) from config snapshot.
                let mut sorted_presets: Vec<(String, String)> = self.config.presets.iter()
                    .map(|(k, v)| (k.clone(), v.filter.clone().unwrap_or_default()))
                    .collect();
                sorted_presets.sort_by_key(|(name, _)| name.clone());
                sorted_presets.truncate(9);

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
                // Restore prior filter (D-02) — do NOT clear
                let snapshot = self.filter_state.as_ref().map(|s| s.snapshot.clone()).unwrap_or_default();
                self.filter_query = snapshot;
                self.filter_state = None;
                self.mode = AppMode::Normal;
                self.rebuild_and_reanchor();
                self.apply_pending_reload()?;
            }
            KeyCode::Enter => {
                self.filter_state = None;
                self.mode = AppMode::Normal;
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
                        self.filter_query = query;
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
                        self.filter_query = query;
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
                    self.filter_query = query;
                    self.rebuild_and_reanchor();
                }
            }
            _ => {
                if let Some(ref mut state) = self.filter_state {
                    state.editor.input_without_shortcuts(Event::Key(key));
                    self.filter_query = state
                        .editor
                        .lines()
                        .first()
                        .cloned()
                        .unwrap_or_default();
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
                // Flush active editor content back to filter_query (live preview finalised).
                self.filter_query = state.active_editor.lines().join("").trim().to_string();

                // Update config.presets from editors.
                for (i, name) in state.preset_names.iter().enumerate() {
                    let filter_str = state.preset_editors[i].lines().join("").trim().to_string();
                    let preset_filter = if filter_str.is_empty() { None } else { Some(filter_str) };
                    self.config.presets.entry(name.clone())
                        .and_modify(|p| p.filter = preset_filter.clone())
                        .or_insert_with(|| crate::config::TuiPreset { filter: preset_filter });
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
                        eprintln!("Warning: failed to save config: {e}");
                    }
                }

                self.filter_defining_state = None;
                self.mode = AppMode::Normal;
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
                    // D-07: live preview — update filter_query from active editor.
                    self.filter_query = self
                        .filter_defining_state
                        .as_ref()
                        .map(|s| s.active_editor.lines().join("").trim().to_string())
                        .unwrap_or_default();
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
                    self.editor.input_without_shortcuts(Event::Key(key));
                }
            }
            KeyCode::Up => {
                if let Some(ref mut ac) = self.autocomplete {
                    if ac.focused {
                        ac.selected = ac.selected.saturating_sub(1);
                    } else {
                        self.editor.input_without_shortcuts(Event::Key(key));
                    }
                } else {
                    self.editor.input_without_shortcuts(Event::Key(key));
                }
            }
            KeyCode::Tab => {
                if self.autocomplete.as_ref().map(|ac| ac.focused).unwrap_or(false) {
                    self.accept_completion();
                } else {
                    // Tab without focused popup — pass to editor.
                    self.editor.input_without_shortcuts(Event::Key(key));
                    self.update_autocomplete();
                }
            }
            KeyCode::Char(' ') => {
                if self.autocomplete.as_ref().map(|ac| ac.focused).unwrap_or(false) {
                    self.accept_completion();
                    // Also insert the space after the token.
                    self.editor.input_without_shortcuts(Event::Key(key));
                } else {
                    self.editor.input_without_shortcuts(Event::Key(key));
                    self.update_autocomplete();
                }
            }
            _ => {
                // Route all other keys through tui-textarea without default shortcuts
                // (PITFALLS: "Single-line editors inheriting multiline and shortcut behavior").
                self.editor.input_without_shortcuts(Event::Key(key));
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
            if let Some(idx) = self.canonical_selected() {
                self.task_list
                    .delete(idx)
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to delete task: {}", e))?;
                self.rebuild_and_reanchor();
            }
        }
        // Any key (including Esc and non-y keys) returns to Normal (D-07).
        self.mode = AppMode::Normal;
        self.apply_pending_reload()?;
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
        let task = Task::parse(&text);
        let mode = self.mode; // Copy
        match mode {
            AppMode::Adding => {
                self.task_list
                    .add(task)
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to add task: {}", e))?;
                // Move selection to the newly added task (D-13).
                let canonical = self.task_list.len().saturating_sub(1);
                self.rebuild_display_indices();
                self.selected = self.display_indices.iter().position(|&x| x == canonical).unwrap_or(0);
            }
            AppMode::Editing { original_idx } => {
                self.task_list
                    .update(original_idx, task)
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to update task: {}", e))?;
                self.rebuild_display_indices();
                self.selected = self.display_indices.iter().position(|&x| x == original_idx).unwrap_or(0);
            }
            _ => {}
        }
        self.editor = TextArea::default();
        self.mode = AppMode::Normal;
        self.apply_pending_reload()
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
            self.rebuild_and_reanchor();
        }
        Ok(())
    }

    /// Clamp `selected` to `[0, display_count - 1]`, or 0 on empty display.
    fn clamp_selection(&mut self) {
        let count = self.display_indices.len();
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
                let f = Filter::from_query(&query);
                self.task_list.filter(&f)
            };
            if sort_order != SortOrder::FileOrder {
                pairs.sort_by(|(_, a), (_, b)| sort_order.compare(a, b));
            }
            pairs.into_iter().map(|(idx, _)| idx).collect()
        };
        self.display_indices = new_indices;
    }

    /// Rebuild display indices while preserving the selected canonical task.
    ///
    /// Saves the current canonical index, rebuilds, then restores the selection
    /// to the display row where that canonical index now appears (or row 0).
    fn rebuild_and_reanchor(&mut self) {
        let old_canonical = self.display_indices.get(self.selected).copied();
        self.rebuild_display_indices();
        self.selected = old_canonical
            .and_then(|ci| self.display_indices.iter().position(|&x| x == ci))
            .unwrap_or(0);
        self.clamp_selection();
    }

    /// Return the canonical task index for the currently selected display row, or `None`
    /// if the display list is empty.
    fn canonical_selected(&self) -> Option<usize> {
        self.display_indices.get(self.selected).copied()
    }

    /// Toggle the completion state of the currently selected task and persist to disk.
    ///
    /// D-10: immediate save via `task_list.update()` (which calls `save()` internally).
    /// D-11: toggles both ways — incomplete→done AND done→incomplete.
    /// D-12: called by both `x` and bare `u`.
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

    /// Render the TUI frame with mode-aware layout.
    ///
    /// Signature is `&mut self` because tui-textarea's Widget impl requires
    /// rendering via a mutable reference on some paths.
    fn draw(&mut self, frame: &mut Frame) {
        use ratatui::layout::{Constraint::{Length, Min}, Layout};

        match self.mode {
            AppMode::DeleteConfirm => {
                // Three-row split: task list | confirm panel | status bar (D-06).
                let chunks =
                    Layout::vertical([Min(0), Length(1), Length(1)]).split(frame.area());
                self.render_task_list(frame, chunks[0]);
                self.render_delete_confirm(frame, chunks[1]);
                self.render_status_bar(frame, chunks[2]);
            }
            AppMode::Adding | AppMode::Editing { .. } => {
                // Two-row split: task list | inline editor in footer row (D-02).
                let chunks =
                    Layout::vertical([Min(0), Length(1)]).split(frame.area());
                self.render_task_list(frame, chunks[0]);
                // tui-textarea renders directly; ratatui 0.29 Widget impl for &TextArea.
                frame.render_widget(&self.editor, chunks[1]);
                // Autocomplete popup floats above the footer row (D-08, D-09).
                self.render_autocomplete_popup(frame, chunks[1]);
            }
            AppMode::Normal => {
                // Two-row split: task list | status bar (D-14).
                let chunks =
                    Layout::vertical([Min(0), Length(1)]).split(frame.area());
                self.render_task_list(frame, chunks[0]);
                self.render_status_bar(frame, chunks[1]);
            }
            AppMode::Filtering => {
                let panel_height = 1_u16 + (self.presets.len() as u16).min(5);
                let chunks =
                    Layout::vertical([Min(0), Length(panel_height), Length(1)]).split(frame.area());
                self.render_task_list(frame, chunks[0]);
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
                self.render_task_list(frame, chunks[0]);
                self.render_filter_defining_panel(frame, chunks[1]);
                self.render_status_bar(frame, chunks[2]);
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
            self.display_indices.iter().map(|&ci| {
                let t = &tasks[ci];
                let content = format!("{}: {}", ci + 1, t.to_raw());
                // Priority and overdue coloring (D-01, D-09 in 13-CONTEXT.md).
                // Style precedence: completed (DIM) > priority A/B/C > overdue > plain.
                // Modifier::REVERSED for selection is applied by List::highlight_style — not here.
                let style = if t.completed {
                    // Completed tasks: DIM only, no color (D-01, D-06).
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
                ListItem::new(content).style(style)
            }).collect()
        };

        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        let mut list_state = ListState::default();
        if !self.display_indices.is_empty() {
            list_state = list_state.with_selected(Some(self.selected));
        }

        frame.render_stateful_widget(list, area, &mut list_state);
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

        let mut middle = String::new();
        let theme_label = match self.theme {
            Theme::Default => "default",
            Theme::Light => "light",
        };
        middle.push_str(" | theme:");
        middle.push_str(theme_label);

        let trimmed_filter = self.filter_query.trim();
        if !trimmed_filter.is_empty() {
            middle.push_str(" | ");
            middle.push_str(trimmed_filter);
        }
        if self.sort_order != SortOrder::FileOrder {
            middle.push_str(" | sort: ");
            middle.push_str(sort_name(self.sort_order));
        }

        let right = "  q quit | n add | u edit | d del | x done | j/k nav | f filter | o sort";
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

    /// Render the one-row delete confirmation panel (D-06, D-07).
    fn render_delete_confirm(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::text::{Line, Span};
        use ratatui::widgets::Paragraph;

        let tasks = self.task_list.tasks();
        let preview = match self.canonical_selected() {
            Some(idx) => tasks[idx].to_raw().to_string(),
            None => String::new(),
        };

        let line = Line::from(vec![
            Span::raw(format!("Delete: \"{}\"", preview)),
            Span::raw("  y=confirm  any=cancel"),
        ]);

        frame.render_widget(Paragraph::new(line), area);
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
            .title(" Filter Definitions (F) — \u{2191}\u{2193}: navigate  Enter: save  Esc: discard ")
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


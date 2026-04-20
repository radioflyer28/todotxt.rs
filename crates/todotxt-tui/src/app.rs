//! Application state and main event loop.
//!
//! All state mutation happens exclusively on the main thread (D-03).
//! The two sender threads only produce `AppEvent` values — they never
//! touch `App` or `TaskList` directly.

use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::Frame;
use todotxt_core::{Task, TaskList};
use tui_textarea::TextArea;

use crate::event::AppEvent;
use crate::tui::Tui;

/// Interaction mode for the TUI (D-01 in 11-CONTEXT.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Adding,
    Editing { original_idx: usize },
    DeleteConfirm,
}

/// Top-level application state.
pub struct App {
    pub should_quit: bool,
    pub task_list: TaskList,
    pub todo_path: PathBuf,
    /// 0-based index into `task_list.tasks()` for the currently selected row.
    /// Always clamped to `[0, task_count - 1]`.
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
}

impl App {
    pub fn new(task_list: TaskList, todo_path: PathBuf) -> Self {
        App {
            should_quit: false,
            task_list,
            todo_path,
            selected: 0,
            list_height: 0,
            mode: AppMode::Normal,
            editor: TextArea::default(),
            pending_reload: false,
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
                    self.clamp_selection();
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
        let task_count = self.task_list.len();
        match key.code {
            // ── Quit ────────────────────────────────────────────────────────
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }

            // ── Navigation ──────────────────────────────────────────────────
            KeyCode::Char('j') | KeyCode::Down if task_count > 0 => {
                self.selected = (self.selected + 1).min(task_count - 1);
            }
            KeyCode::Char('k') | KeyCode::Up if task_count > 0 => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('g') if task_count > 0 => {
                self.selected = 0;
            }
            KeyCode::Char('G') if task_count > 0 => {
                self.selected = task_count - 1;
            }
            // Ctrl+U half-page up — must come before plain 'u' (edit).
            KeyCode::Char('u')
                if key.modifiers.contains(KeyModifiers::CONTROL) && task_count > 0 =>
            {
                let half = (self.list_height / 2).max(1) as usize;
                self.selected = self.selected.saturating_sub(half);
            }
            // Ctrl+D half-page down — must come before plain 'd' (delete).
            KeyCode::Char('d')
                if key.modifiers.contains(KeyModifiers::CONTROL) && task_count > 0 =>
            {
                let half = (self.list_height / 2).max(1) as usize;
                self.selected = (self.selected + half).min(task_count - 1);
            }

            // ── Done toggle ──────────────────────────────────────────────────
            KeyCode::Char('x') if task_count > 0 => {
                self.toggle_done();
            }

            // ── Add task — always available even on empty list ───────────────
            KeyCode::Char('n') => {
                self.editor = TextArea::default();
                self.mode = AppMode::Adding;
            }

            // ── Edit task (u or e) — after Ctrl+U arm ───────────────────────
            KeyCode::Char('u') | KeyCode::Char('e') if task_count > 0 => {
                let raw = self.task_list.tasks()[self.selected].to_raw().to_string();
                let mut ed = TextArea::default();
                ed.insert_str(&raw);
                self.editor = ed;
                self.mode = AppMode::Editing { original_idx: self.selected };
            }

            // ── Delete task — after Ctrl+D arm ──────────────────────────────
            KeyCode::Char('d') if task_count > 0 => {
                self.mode = AppMode::DeleteConfirm;
            }

            _ => {}
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
                self.exit_edit_mode()?;
            }
            KeyCode::Enter => {
                self.save_and_exit()?;
            }
            _ => {
                // Route all other keys through tui-textarea without default shortcuts
                // (PITFALLS: "Single-line editors inheriting multiline and shortcut behavior").
                self.editor.input_without_shortcuts(Event::Key(key));
            }
        }
        Ok(())
    }

    // ── Delete confirm key handler ────────────────────────────────────────────

    fn handle_delete_confirm_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> color_eyre::Result<()> {
        if key.code == KeyCode::Char('y') {
            let idx = self.selected;
            self.task_list
                .delete(idx)
                .map_err(|e| color_eyre::eyre::eyre!("Failed to delete task: {}", e))?;
            self.clamp_selection();
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
                self.selected = self.task_list.len().saturating_sub(1);
            }
            AppMode::Editing { original_idx } => {
                self.task_list
                    .update(original_idx, task)
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to update task: {}", e))?;
                self.selected = original_idx;
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
            self.clamp_selection();
        }
        Ok(())
    }

    /// Clamp `selected` to `[0, task_count - 1]`, or 0 on empty list.
    fn clamp_selection(&mut self) {
        let count = self.task_list.len();
        if count == 0 {
            self.selected = 0;
        } else {
            self.selected = self.selected.min(count - 1);
        }
    }

    /// Toggle the completion state of the currently selected task and persist to disk.
    ///
    /// D-10: immediate save via `task_list.update()` (which calls `save()` internally).
    /// D-11: toggles both ways — incomplete→done AND done→incomplete.
    /// D-12: called by both `x` and bare `u`.
    fn toggle_done(&mut self) {
        let count = self.task_list.len();
        if count == 0 {
            return;
        }
        let idx = self.selected;
        let task = self.task_list.tasks()[idx].clone();
        let was_completed = task.completed;
        let toggled = task.with_completed(!was_completed);
        // update() calls save() internally — single atomic disk write (temp rename).
        if let Err(e) = self.task_list.update(idx, toggled) {
            // Non-fatal: write to stderr (terminal restore guard is active).
            eprintln!("toggle_done error: {e}");
        }
        // Clamp in case the task list shrank after the write.
        let new_count = self.task_list.len();
        if new_count > 0 {
            self.selected = self.selected.min(new_count - 1);
        } else {
            self.selected = 0;
        }
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
            }
            AppMode::Normal => {
                // Two-row split: task list | status bar (D-14).
                let chunks =
                    Layout::vertical([Min(0), Length(1)]).split(frame.area());
                self.render_task_list(frame, chunks[0]);
                self.render_status_bar(frame, chunks[1]);
            }
        }
    }

    /// Render the task list with selection highlight.
    fn render_task_list(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::style::{Modifier, Style};
        use ratatui::widgets::{List, ListItem, ListState};

        let tasks = self.task_list.tasks();

        let items: Vec<ListItem> = if tasks.is_empty() {
            vec![ListItem::new("(no tasks)")]
        } else {
            tasks
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    let content = format!("{}: {}", i + 1, t.to_raw());
                    let style = if t.completed {
                        Style::default().add_modifier(Modifier::DIM)
                    } else {
                        Style::default()
                    };
                    ListItem::new(content).style(style)
                })
                .collect()
        };

        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        let mut list_state = ListState::default();
        if !tasks.is_empty() {
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
        let visible = total;
        let due_today = tasks
            .iter()
            .filter(|t| !t.completed && t.due_status() == DueStatus::Today)
            .count();
        let overdue = tasks
            .iter()
            .filter(|t| !t.completed && t.due_status() == DueStatus::Overdue)
            .count();

        let file_name = self
            .todo_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("todo.txt");

        let left = format!(
            "{} | {} tasks | {} visible | {} due today | {} overdue",
            file_name, total, visible, due_today, overdue
        );
        let right = "q quit | n add | u edit | d del | x done | j/k nav";

        let status_line = Line::from(vec![
            Span::raw(left),
            Span::raw("  "),
            Span::raw(right),
        ]);

        frame.render_widget(Paragraph::new(status_line), area);
    }

    /// Render the one-row delete confirmation panel (D-06, D-07).
    fn render_delete_confirm(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::text::{Line, Span};
        use ratatui::widgets::Paragraph;

        let tasks = self.task_list.tasks();
        let preview = if tasks.is_empty() {
            String::new()
        } else {
            tasks[self.selected].to_raw().to_string()
        };

        let line = Line::from(vec![
            Span::raw(format!("Delete: \"{}\"", preview)),
            Span::raw("  y=confirm  any=cancel"),
        ]);

        frame.render_widget(Paragraph::new(line), area);
    }
}


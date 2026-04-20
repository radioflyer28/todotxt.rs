//! Application state and main event loop.
//!
//! All state mutation happens exclusively on the main thread (D-03).
//! The two sender threads only produce `AppEvent` values — they never
//! touch `App` or `TaskList` directly.

use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::Frame;
use todotxt_core::TaskList;
use tui_textarea::TextArea;

use crate::event::AppEvent;
use crate::tui::Tui;

/// Interaction mode for the TUI (D-01 in 11-CONTEXT.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
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
    #[allow(dead_code)]
    pub mode: AppMode,
    /// Single-line text editor used in Adding and Editing modes (D-03).
    #[allow(dead_code)]
    pub editor: TextArea<'static>,
    /// When true, a `FileChanged` event arrived while not in Normal mode and
    /// will be applied on the next Normal-mode entry (D-10).
    #[allow(dead_code)]
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

    /// Handle a single `AppEvent`. All state mutation is here (D-03).
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

                let task_count = self.task_list.len();

                match key.code {
                    // ── Quit ────────────────────────────────────────────────
                    KeyCode::Char('q') => {
                        self.should_quit = true;
                        return Ok(());
                    }
                    KeyCode::Char('c')
                        if key.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        self.should_quit = true;
                        return Ok(());
                    }

                    // ── Navigation (D-08, D-09) — only meaningful on non-empty lists ──
                    // Move down 1
                    KeyCode::Char('j') | KeyCode::Down if task_count > 0 => {
                        self.selected = (self.selected + 1).min(task_count - 1);
                    }
                    // Move up 1
                    KeyCode::Char('k') | KeyCode::Up if task_count > 0 => {
                        self.selected = self.selected.saturating_sub(1);
                    }
                    // Jump to first (g)
                    KeyCode::Char('g') if task_count > 0 => {
                        self.selected = 0;
                    }
                    // Jump to last (G)
                    KeyCode::Char('G') if task_count > 0 => {
                        self.selected = task_count - 1;
                    }
                    // Half-page down (Ctrl+d)
                    KeyCode::Char('d')
                        if key.modifiers.contains(KeyModifiers::CONTROL)
                            && task_count > 0 =>
                    {
                        let half = (self.list_height / 2).max(1) as usize;
                        self.selected = (self.selected + half).min(task_count - 1);
                    }
                    // Half-page up (Ctrl+u)
                    KeyCode::Char('u')
                        if key.modifiers.contains(KeyModifiers::CONTROL)
                            && task_count > 0 =>
                    {
                        let half = (self.list_height / 2).max(1) as usize;
                        self.selected = self.selected.saturating_sub(half);
                    }

                    // ── Done / undo (D-10, D-11, D-12) ──────────────────────
                    // Toggle done (x)
                    KeyCode::Char('x') if task_count > 0 => {
                        self.toggle_done();
                    }
                    // u → edit selected task (Phase 11); not wired yet.

                    _ => {}
                }
            }
            AppEvent::FileChanged => {
                // Reload task list on external file change.
                self.task_list.reload().map_err(|e| {
                    color_eyre::eyre::eyre!(
                        "Failed to reload {}: {}",
                        self.todo_path.display(),
                        e
                    )
                })?;
                // Clamp selected so it stays in bounds after reload (D-07).
                let task_count = self.task_list.len();
                if task_count > 0 {
                    self.selected = self.selected.min(task_count - 1);
                } else {
                    self.selected = 0;
                }
            }
            AppEvent::Resize(_, rows) => {
                // Update list_height on resize; subtract 1 for the status bar row (D-14).
                self.list_height = rows.saturating_sub(1);
            }
            AppEvent::Error(_msg) => {
                // Non-fatal: swallow silently.
            }
        }

        // Redraw after every event (including resize and file change).
        terminal.draw(|f| self.draw(f))?;
        Ok(())
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

    /// Render the task list using ratatui `List` + `ListState` (D-04, D-05, D-06).
    ///
    /// Layout (D-14): list area occupies all rows except the 1-row status bar footer.
    /// Row format (D-01): "{1-based line number}: {raw task text}"
    /// Completed tasks: `Modifier::DIM` (D-03).
    /// Selected row: `Modifier::REVERSED` (D-06).
    fn draw(&self, frame: &mut Frame) {
        use ratatui::layout::{Constraint::{Length, Min}, Layout};
        use ratatui::style::{Modifier, Style};
        use ratatui::text::{Line, Span};
        use ratatui::widgets::{List, ListItem, ListState, Paragraph};
        use todotxt_core::DueStatus;

        // ── Layout split (D-14) ───────────────────────────────────────────────
        let chunks = Layout::vertical([Min(0), Length(1)]).split(frame.area());
        let list_area = chunks[0];
        let status_area = chunks[1];

        // ── Task list (D-01 through D-06) ─────────────────────────────────────
        let tasks = self.task_list.tasks();

        let items: Vec<ListItem> = if tasks.is_empty() {
            vec![ListItem::new("(no tasks)")]
        } else {
            tasks
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    // D-01: line number = source file line (1-based), not display index.
                    let content = format!("{}: {}", i + 1, t.to_raw());
                    // D-03: completed tasks rendered dimmed.
                    let style = if t.completed {
                        Style::default().add_modifier(Modifier::DIM)
                    } else {
                        Style::default()
                    };
                    ListItem::new(content).style(style)
                })
                .collect()
        };

        // D-06: default reversed-colors highlight for selected row.
        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        // D-05: ListState built fresh each draw; not stored on App.
        let mut list_state = ListState::default();
        if !tasks.is_empty() {
            list_state = list_state.with_selected(Some(self.selected));
        }

        // frame.area() is correct for ratatui 0.30 (frame.size() is deprecated).
        frame.render_stateful_widget(list, list_area, &mut list_state);

        // ── Status bar (D-14 through D-17) ────────────────────────────────────
        let total = tasks.len();
        // "visible" = total in Phase 10 (no filtering yet; Phase 11 adds filters).
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

        // D-15: left segment — file info and counts.
        let left = format!(
            "{} | {} tasks | {} visible | {} due today | {} overdue",
            file_name, total, visible, due_today, overdue
        );
        // D-15: right segment — key hints.
        let right = "q quit | x done | u edit | j/k nav";

        // D-16: simple Span approach — agent discretion for layout.
        // D-17: monochrome in Phase 10; Phase 13 adds theme colors.
        let status_line = Line::from(vec![
            Span::raw(left),
            Span::raw("  "),
            Span::raw(right),
        ]);

        frame.render_widget(Paragraph::new(status_line), status_area);
    }
}

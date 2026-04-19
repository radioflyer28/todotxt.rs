//! Application state and main event loop.
//!
//! All state mutation happens exclusively on the main thread (D-03).
//! The two sender threads only produce `AppEvent` values — they never
//! touch `App` or `TaskList` directly.

use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::text::{Line, Text};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use todotxt_core::TaskList;

use crate::event::AppEvent;
use crate::tui::Tui;

/// Top-level application state for Phase 9.
///
/// Phase 10 will expand this with selection, mode, filter, status bar, etc.
pub struct App {
    pub should_quit: bool,
    pub task_list: TaskList,
    pub todo_path: PathBuf,
}

impl App {
    pub fn new(task_list: TaskList, todo_path: PathBuf) -> Self {
        App {
            should_quit: false,
            task_list,
            todo_path,
        }
    }

    /// Main event loop. Blocks on `rx.recv()` — no polling (D-02).
    pub fn run(
        &mut self,
        terminal: &mut Tui,
        rx: Receiver<AppEvent>,
    ) -> color_eyre::Result<()> {
        // Initial draw before waiting for the first event.
        terminal.draw(|f| self.draw(f))?;

        loop {
            match rx.recv() {
                Ok(event) => {
                    self.handle_event(event, terminal)?;
                }
                Err(_) => {
                    // All senders dropped (both threads exited). Exit cleanly.
                    break;
                }
            }

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
                if key.code == KeyCode::Char('q')
                    || (key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL))
                {
                    self.should_quit = true;
                    return Ok(());
                }
            }
            AppEvent::FileChanged => {
                // D-11: Reload task list and redraw to prove auto-refresh.
                self.task_list.reload().map_err(|e| {
                    color_eyre::eyre::eyre!(
                        "Failed to reload {}: {}",
                        self.todo_path.display(),
                        e
                    )
                })?;
            }
            AppEvent::Resize(_, _) => {
                // Ratatui handles resize automatically on the next draw.
                // No state change needed.
            }
            AppEvent::Error(_msg) => {
                // Non-fatal: swallow silently for Phase 9.
            }
        }

        // Redraw after every event (including resize and file change).
        terminal.draw(|f| self.draw(f))?;
        Ok(())
    }

    /// Render the task list as plain-text lines (D-10).
    ///
    /// No colors, no cursor highlight, no selection — Phase 10 owns those.
    /// One line per task: "{line_number}: {raw_text}".
    fn draw(&self, frame: &mut Frame) {
        let tasks = self.task_list.tasks();

        let lines: Vec<Line> = if tasks.is_empty() {
            vec![Line::raw("(no tasks)")]
        } else {
            tasks
                .iter()
                .enumerate()
                .map(|(i, t)| Line::raw(format!("{}: {}", i + 1, t.to_raw())))
                .collect()
        };

        let text = Text::from(lines);
        let widget = Paragraph::new(text);
        // frame.area() is correct for ratatui 0.30 (frame.size() is deprecated).
        frame.render_widget(widget, frame.area());
    }
}

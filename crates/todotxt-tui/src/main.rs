#![deny(warnings)]

mod app;
mod config;
mod event;
mod tui;

use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

use color_eyre::eyre::eyre;
use crossterm::event::{read, Event};
use todotxt_core::{FileWatcher, TaskList};

use app::App;
use config::TuiConfig;
use event::AppEvent;
use tui::TerminalGuard;

fn main() -> color_eyre::Result<()> {
    // D-08: Install color-eyre FIRST.
    // The panic hook it registers restores the terminal before printing the panic message.
    color_eyre::install()?;

    // D-07: Resolve config path with portable mode support.
    let platform_path = TuiConfig::default_path()
        .ok_or_else(|| eyre!("Cannot determine config directory"))?;
    let config_path = TuiConfig::resolve_path(&platform_path);
    let config = TuiConfig::load(&config_path)?;

    let todo_path = config.todo_file.ok_or_else(|| {
        eyre!(
            "todo_file is not set in config.toml ({}).\nHint: add:  todo_file = \"/path/to/todo.txt\"",
            config_path.display()
        )
    })?;

    if !todo_path.exists() {
        return Err(eyre!("todo.txt not found at: {}", todo_path.display()));
    }

    // Load initial task list.
    let task_list = TaskList::load(&todo_path)
        .map_err(|e| eyre!("Failed to load {}: {}", todo_path.display(), e))?;

    // D-02: Single mpsc channel; both sender threads share a clone of the sender.
    let (tx, rx) = mpsc::channel::<AppEvent>();

    // Thread 1: Crossterm keyboard/resize events.
    // Uses blocking `crossterm::event::read()` — no tokio required (D-01).
    let tx_input = tx.clone();
    thread::spawn(move || {
        loop {
            match read() {
                Ok(Event::Key(k)) => {
                    if tx_input.send(AppEvent::Key(k)).is_err() {
                        break; // Receiver dropped — main loop has exited.
                    }
                }
                Ok(Event::Resize(cols, rows)) => {
                    if tx_input.send(AppEvent::Resize(cols, rows)).is_err() {
                        break;
                    }
                }
                Ok(_) => {
                    // Focus, paste, and other events: silently ignored for Phase 9.
                }
                Err(e) => {
                    let _ = tx_input.send(AppEvent::Error(e.to_string()));
                    break;
                }
            }
        }
    });

    // Thread 2: FileWatcher callback (callback-based, sync — no tokio needed).
    // The callback only sends a message; it never touches app state (D-03).
    let tx_watch = tx.clone();
    let _watcher = FileWatcher::new(
        &todo_path,
        Arc::new(move || {
            let _ = tx_watch.send(AppEvent::FileChanged);
        }),
    )
    .map_err(|e| eyre!("FileWatcher failed to start: {}", e))?;

    // D-09: RAII terminal guard — Drop restores terminal on all exit paths.
    let mut guard = TerminalGuard::new()?;

    // Run the event loop.
    let mut app = App::new(task_list, todo_path);
    app.run(&mut guard.terminal, rx)?;

    // Guard drops here → disable_raw_mode + LeaveAlternateScreen.
    Ok(())
}



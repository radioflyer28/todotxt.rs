#![deny(warnings)]

mod app;
mod components;
mod config;
mod event;
mod state;
mod theme;
mod tui;

use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

use clap::Parser;
use color_eyre::eyre::eyre;
use crossterm::event::{read, Event};
use todotxt_core::{FileWatcher, TaskList};

use app::App;
use config::{resolve_startup_paths, CliPathOverrides, TuiConfig};
use event::AppEvent;
use tui::TerminalGuard;

#[derive(Debug, Parser)]
#[command(name = "todotxt-tui", version)]
struct Args {
    #[arg(short = 't', long = "todo", value_name = "PATH")]
    todo: Option<std::path::PathBuf>,

    #[arg(short = 'a', long = "archive", value_name = "PATH")]
    archive: Option<std::path::PathBuf>,

    #[arg(short = 'c', long = "config", value_name = "PATH")]
    config: Option<std::path::PathBuf>,
}

fn main() -> color_eyre::Result<()> {
    // D-08: Install color-eyre FIRST.
    // The panic hook it registers restores the terminal before printing the panic message.
    color_eyre::install()?;

    let args = Args::parse();

    // D-07: Resolve config path with portable mode support.
    let platform_path = TuiConfig::default_path()
        .ok_or_else(|| eyre!("Cannot determine config directory"))?;
    let config_path = if let Some(explicit) = args.config.as_ref() {
        explicit.clone()
    } else {
        TuiConfig::resolve_path(&platform_path)
    };
    let mut config = TuiConfig::load(&config_path)?;

    // Phase 43 (PRSV-01/02): Load view state sidecar; if present and non-empty,
    // override config.panes so the last session's pane layout is restored.
    let state_path = config::state_file_path(&config_path);
    if let Some(state) = config::TuiStateFile::load(&state_path) {
        if !state.panes.is_empty() {
            config.panes = state.panes;
        }
    }

    let overrides = CliPathOverrides {
        todo: args.todo.clone(),
        archive: args.archive.clone(),
    };
    let resolved_paths = resolve_startup_paths(&config, &overrides)?;
    let todo_path = resolved_paths.todo_path;

    config.todo_file = Some(todo_path.clone());
    config.done_file = Some(resolved_paths.archive_path);

    if !todo_path.exists() {
        return Err(eyre!("todo.txt not found at: {}", todo_path.display()));
    }

    std::fs::File::open(&todo_path)
        .map_err(|e| eyre!("todo.txt is not readable at {}: {}", todo_path.display(), e))?;

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

    // D-07 (13-CONTEXT.md): Check NO_COLOR once at startup — never per-frame.
    // Per https://no-color.org/: presence of the variable (any value) disables color.
    let no_color = std::env::var("NO_COLOR").is_ok();

    // D-03, D-05 (13-CONTEXT.md): Parse theme name from config; unknown → Default.
    let theme = crate::theme::Theme::from_str(&config.tui.theme);

    // Run the event loop.
    let mut app = App::new(task_list, todo_path, config, Some(config_path), theme, no_color);
    app.run(&mut guard.terminal, rx)?;

    // Guard drops here → disable_raw_mode + LeaveAlternateScreen.
    Ok(())
}



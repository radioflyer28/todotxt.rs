use crate::{config::Config, output::Renderer, CliError};
use chrono::Local;
use std::path::Path;
use todotxt_core::{Task, TaskList};

/// Add a new task to todo.txt.
///
/// - `force_date`: `--date` flag — prepend creation date regardless of config
/// - `no_date`: `--no-date` flag — suppress creation date even if config says true
pub fn run(
    todo_path: &Path,
    text: &str,
    force_date: bool,
    no_date: bool,
    cfg: &Config,
    renderer: &Renderer,
) -> Result<(), CliError> {
    if text.trim().is_empty() {
        return Err(CliError::Other(anyhow::anyhow!("task text cannot be empty")));
    }

    // Determine whether to prepend today's creation date (D-01).
    // --date forces it; --no-date suppresses it; otherwise follow config.
    let use_date = (cfg.auto_creation_date || force_date) && !no_date;

    let raw = if use_date {
        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        format!("{today} {text}")
    } else {
        text.to_string()
    };

    let task = Task::parse(&raw);
    let mut list = TaskList::load(todo_path)?;
    list.add(task)?;

    // Read back from list to get the actual stored task (round-trip safe).
    let idx = list.tasks().len() - 1;
    let added_task = &list.tasks()[idx];
    renderer.print_write_result(&format!("Added task #{}.", idx + 1), idx, added_task);

    Ok(())
}

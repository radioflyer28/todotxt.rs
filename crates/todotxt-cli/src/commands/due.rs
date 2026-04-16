use crate::{output::Renderer, CliError};
use std::path::Path;

/// Set due date on a task (`due`).
pub fn run_due(
    _todo_path: &Path,
    _id: usize,
    _date: &str,
    _renderer: &Renderer,
) -> Result<(), CliError> {
    // TODO: Implement due date setting logic
    // - Parse date string (today, tomorrow, weekday name, YYYY-MM-DD)
    // - Load TaskList
    // - Validate ID
    // - Update task with new due date
    // - Save TaskList
    // - Print result

    todo!("run_due not yet implemented")
}

/// Move task's due date forward by N days (`postpone`).
pub fn run_postpone(
    _todo_path: &Path,
    _id: usize,
    _days: u32,
    _renderer: &Renderer,
) -> Result<(), CliError> {
    // TODO: Implement postpone logic
    // - Load TaskList
    // - Validate ID
    // - Check that task has existing due date (error if not)
    // - Calculate new due date (current + days)
    // - Update task
    // - Save TaskList
    // - Print result

    todo!("run_postpone not yet implemented")
}

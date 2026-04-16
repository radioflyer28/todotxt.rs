use crate::{output::Renderer, CliError};
use std::path::Path;

/// Delete all completed tasks from todo.txt (`del-done`).
pub fn run_del_done(_todo_path: &Path, _renderer: &Renderer) -> Result<(), CliError> {
    // TODO: Implement del-done logic
    // - Load TaskList from todo.txt
    // - Filter out completed tasks
    // - Count how many were deleted
    // - Write filtered list back to todo.txt
    // - Print "{count} completed tasks deleted" to stderr
    // - Exit 0 (even if 0 tasks deleted)

    todo!("run_del_done not yet implemented")
}

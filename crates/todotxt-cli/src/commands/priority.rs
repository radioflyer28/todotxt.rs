use crate::{output::Renderer, CliError};
use std::path::Path;

/// Set priority (A-Z) for one or more tasks (`pri`).
pub fn run_pri(
    _todo_path: &Path,
    ids: &[usize],
    _priority: char,
    _renderer: &Renderer,
) -> Result<(), CliError> {
    if ids.is_empty() {
        return Err(CliError::Other(anyhow::anyhow!(
            "at least one task ID required"
        )));
    }

    // TODO: Implement priority setting logic
    // - Validate priority is A-Z
    // - Load TaskList
    // - Validate all IDs before mutating
    // - Sort descending and dedup IDs
    // - Update each task with new priority
    // - Save TaskList
    // - Print result for each task

    todo!("run_pri not yet implemented")
}

/// Remove priority from one or more tasks (`depri`).
pub fn run_depri(
    _todo_path: &Path,
    ids: &[usize],
    _renderer: &Renderer,
) -> Result<(), CliError> {
    if ids.is_empty() {
        return Err(CliError::Other(anyhow::anyhow!(
            "at least one task ID required"
        )));
    }

    // TODO: Implement priority removal logic
    // - Load TaskList
    // - Validate all IDs before mutating
    // - Sort descending and dedup IDs
    // - Update each task to remove priority
    // - Save TaskList
    // - Print result for each task

    todo!("run_depri not yet implemented")
}

use crate::{output::Renderer, CliError};
use std::path::Path;
use todotxt_core::TaskList;

pub fn run(todo_path: &Path, id: usize, renderer: &Renderer) -> Result<(), CliError> {
    if id == 0 {
        return Err(CliError::NotFound(format!(
            "task ID {} not found (IDs start at 1)",
            id
        )));
    }
    let list = TaskList::load(todo_path)?;
    let idx = id - 1; // convert 1-based display ID to 0-based index
    let task = list.tasks().get(idx).ok_or_else(|| {
        CliError::NotFound(format!(
            "task {} not found (list has {} tasks)",
            id,
            list.tasks().len()
        ))
    })?;
    renderer.print_task(idx, task); // D-13: raw line
    Ok(())
}

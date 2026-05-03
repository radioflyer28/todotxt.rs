use crate::{output::Renderer, CliError};
use std::path::Path;
use todotxt_core::{normalize_append, Task, TaskList};

/// Append text to the end of a task (`append` command).
pub fn run(todo_path: &Path, id: usize, text: &str, normalize: bool, renderer: &Renderer) -> Result<(), CliError> {
    let idx = validate_id(id)?;
    let mut list = TaskList::load(todo_path)?;

    if idx >= list.len() {
        return Err(CliError::NotFound(format!(
            "task {} not found (list has {} tasks)",
            id,
            list.len()
        )));
    }

    let task = list.tasks()[idx].clone();
    // D-09/D-10: --normalize flag calls shared normalize_append from todotxt-core.
    // Without --normalize, raw concat behavior is unchanged (non-breaking).
    let updated = if normalize {
        normalize_append(&task, text)
    } else {
        Task::parse(&format!("{} {}", task.to_raw(), text))
    };
    list.update(idx, updated.clone())?;
    renderer.print_write_result(&format!("Appended to task #{}.", id), idx, &updated);

    Ok(())
}

fn validate_id(id: usize) -> Result<usize, CliError> {
    if id == 0 {
        return Err(CliError::NotFound(
            "task ID 0 is invalid (IDs start at 1)".to_string(),
        ));
    }
    Ok(id - 1)
}

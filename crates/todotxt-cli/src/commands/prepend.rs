use crate::{output::Renderer, CliError};
use std::path::Path;
use todotxt_core::TaskList;

/// Prepend text before a task's body (`prepend` command).
pub fn run(todo_path: &Path, id: usize, text: &str, renderer: &Renderer) -> Result<(), CliError> {
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
    let updated = task.with_text_prepended(text);
    list.update(idx, updated.clone())?;
    renderer.print_write_result(&format!("Prepended to task #{}.", id), idx, &updated);

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

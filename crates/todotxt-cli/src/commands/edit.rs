use crate::{output::Renderer, CliError};
use std::path::Path;
use todotxt_core::{Task, TaskList};

/// Replace a task's entire text with `new_text` (`edit` command).
pub fn run(
    todo_path: &Path,
    id: usize,
    new_text: &str,
    renderer: &Renderer,
) -> Result<(), CliError> {
    if new_text.trim().is_empty() {
        return Err(CliError::Other(anyhow::anyhow!(
            "replacement text cannot be empty"
        )));
    }

    let idx = validate_id(id)?;
    let mut list = TaskList::load(todo_path)?;

    if idx >= list.len() {
        return Err(CliError::NotFound(format!(
            "task {} not found (list has {} tasks)",
            id,
            list.len()
        )));
    }

    let updated = Task::parse(new_text);
    list.update(idx, updated.clone())?;
    renderer.print_write_result(&format!("Edited task #{}.", id), idx, &updated);

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

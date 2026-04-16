use crate::{output::Renderer, CliError};
use std::path::Path;
use todotxt_core::TaskList;

/// Delete one or more tasks by 1-based ID.
pub fn run(todo_path: &Path, ids: &[usize], renderer: &Renderer) -> Result<(), CliError> {
    if ids.is_empty() {
        return Err(CliError::Other(anyhow::anyhow!(
            "at least one task ID required"
        )));
    }

    let mut list = TaskList::load(todo_path)?;

    // Validate all IDs before deleting any task.
    let mut indices: Vec<usize> = Vec::with_capacity(ids.len());
    for &id in ids {
        let idx = validate_id(id, list.len())?;
        indices.push(idx);
    }

    indices.sort_unstable_by(|a, b| b.cmp(a));
    indices.dedup();

    for idx in indices {
        let task = list.tasks()[idx].clone();
        renderer.print_write_result(&format!("Deleted task #{}.", idx + 1), idx, &task);
        list.delete(idx)?;
    }

    Ok(())
}

fn validate_id(id: usize, list_len: usize) -> Result<usize, CliError> {
    if id == 0 {
        return Err(CliError::NotFound(
            "task ID 0 is invalid (IDs start at 1)".to_string(),
        ));
    }

    let idx = id - 1;
    if idx >= list_len {
        return Err(CliError::NotFound(format!(
            "task {} not found (list has {} tasks)",
            id, list_len
        )));
    }

    Ok(idx)
}

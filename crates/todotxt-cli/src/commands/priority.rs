use crate::{output::Renderer, CliError};
use std::path::Path;
use todotxt_core::TaskList;

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

/// Set priority (A-Z) for one or more tasks (`pri`).
pub fn run_pri(
    todo_path: &Path,
    ids: &[usize],
    priority: char,
    renderer: &Renderer,
) -> Result<(), CliError> {
    if ids.is_empty() {
        return Err(CliError::Other(anyhow::anyhow!(
            "at least one task ID required"
        )));
    }

    let priority_upper = priority.to_ascii_uppercase();
    if !priority_upper.is_ascii_alphabetic() {
        return Err(CliError::Other(anyhow::anyhow!(
            "invalid priority '{}': must be A-Z",
            priority
        )));
    }

    let mut list = TaskList::load(todo_path)?;

    // Validate ALL IDs before mutating (fail-fast per D-01)
    let mut indices: Vec<usize> = Vec::with_capacity(ids.len());
    for &id in ids {
        let idx = validate_id(id, list.len())?;
        indices.push(idx);
    }

    indices.sort_unstable_by(|a, b| b.cmp(a));
    indices.dedup();

    for idx in indices {
        let task = list.tasks()[idx].clone();
        let updated = task.with_priority(Some(priority_upper));
        list.update(idx, updated.clone())?;
        renderer.print_write_result(
            &format!("Priority set to ({}) for task #{}.", priority_upper, idx + 1),
            idx,
            &updated,
        );
    }

    list.save()?;
    Ok(())
}

/// Remove priority from one or more tasks (`depri`).
pub fn run_depri(
    todo_path: &Path,
    ids: &[usize],
    renderer: &Renderer,
) -> Result<(), CliError> {
    if ids.is_empty() {
        return Err(CliError::Other(anyhow::anyhow!(
            "at least one task ID required"
        )));
    }

    let mut list = TaskList::load(todo_path)?;

    // Validate ALL IDs before mutating (fail-fast per D-01)
    let mut indices: Vec<usize> = Vec::with_capacity(ids.len());
    for &id in ids {
        let idx = validate_id(id, list.len())?;
        indices.push(idx);
    }

    indices.sort_unstable_by(|a, b| b.cmp(a));
    indices.dedup();

    for idx in indices {
        let task = list.tasks()[idx].clone();
        if task.priority.is_none() {
            eprintln!("info: task {} has no priority, skipping.", idx + 1);
            continue;
        }
        let updated = task.with_priority(None);
        list.update(idx, updated.clone())?;
        renderer.print_write_result(
            &format!("Removed priority from task #{}.", idx + 1),
            idx,
            &updated,
        );
    }

    list.save()?;
    Ok(())
}

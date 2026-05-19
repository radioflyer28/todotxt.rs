use crate::{output::Renderer, CliError};
use chrono::Local;
use std::path::Path;
use todotxt_core::TaskList;

/// Mark one or more tasks complete (`do`).
pub fn run_do(todo_path: &Path, ids: &[usize], renderer: &Renderer) -> Result<(), CliError> {
    if ids.is_empty() {
        return Err(CliError::Other(anyhow::anyhow!(
            "at least one task ID required"
        )));
    }

    let mut list = TaskList::load(todo_path)?;

    let mut indices: Vec<usize> = Vec::with_capacity(ids.len());
    for &id in ids {
        let idx = validate_id(id, list.len())?;
        indices.push(idx);
    }

    indices.sort_unstable_by(|a, b| b.cmp(a));
    indices.dedup();

    let completion_date = Local::now().date_naive();
    let mut tasks = list.tasks().to_vec();
    let mut completed_outputs = Vec::new();
    let mut generated = Vec::new();

    for idx in indices {
        let task = tasks[idx].clone();
        if task.completed {
            eprintln!("info: task {} is already completed, skipping.", idx + 1);
            continue;
        }

        if let Some(next_task) = task.next_recurring_occurrence(completion_date) {
            generated.push(next_task);
        }
        let updated = task.with_completed(true);
        tasks[idx] = updated.clone();
        completed_outputs.push((idx, updated));
    }

    if !completed_outputs.is_empty() || !generated.is_empty() {
        tasks.extend(generated);
        list.replace_all(tasks)?;
    }

    for (idx, updated) in completed_outputs {
        renderer.print_write_result(&format!("Completed task #{}.", idx + 1), idx, &updated);
    }

    Ok(())
}

/// Unmark one or more completed tasks (`undo`).
pub fn run_undo(todo_path: &Path, ids: &[usize], renderer: &Renderer) -> Result<(), CliError> {
    if ids.is_empty() {
        return Err(CliError::Other(anyhow::anyhow!(
            "at least one task ID required"
        )));
    }

    let mut list = TaskList::load(todo_path)?;

    let mut indices: Vec<usize> = Vec::with_capacity(ids.len());
    for &id in ids {
        let idx = validate_id(id, list.len())?;
        indices.push(idx);
    }

    indices.sort_unstable_by(|a, b| b.cmp(a));
    indices.dedup();

    for idx in indices {
        let task = list.tasks()[idx].clone();
        if !task.completed {
            eprintln!("info: task {} is already incomplete, skipping.", idx + 1);
            continue;
        }

        let updated = task.with_completed(false);
        list.update(idx, updated.clone())?;
        renderer.print_write_result(&format!("Undid task #{}.", idx + 1), idx, &updated);
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

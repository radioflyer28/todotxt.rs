use crate::{date::parse_date_input, output::Renderer, CliError};
use chrono::{Duration, Local};
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

/// Set due date on a task (`due`).
pub fn run_due(
    todo_path: &Path,
    id: usize,
    date: &str,
    renderer: &Renderer,
) -> Result<(), CliError> {
    let today = Local::now().date_naive();
    let due_date = parse_date_input(date, today)
        .map_err(|e| CliError::Other(anyhow::anyhow!("{}", e)))?;

    let mut list = TaskList::load(todo_path)?;
    let idx = validate_id(id, list.len())?;

    let task = list.tasks()[idx].clone();
    let updated = task.with_due_date(Some(due_date));
    list.update(idx, updated.clone())?;
    list.save()?;

    renderer.print_write_result(
        &format!("Set due date to {} on task #{}.", due_date.format("%Y-%m-%d"), id),
        idx,
        &updated,
    );

    Ok(())
}

/// Move task's due date forward by N days (`postpone`).
pub fn run_postpone(
    todo_path: &Path,
    id: usize,
    days: u32,
    renderer: &Renderer,
) -> Result<(), CliError> {
    let mut list = TaskList::load(todo_path)?;
    let idx = validate_id(id, list.len())?;

    let task = list.tasks()[idx].clone();
    let current_due = task.due_date.ok_or_else(|| {
        CliError::Other(anyhow::anyhow!(
            "task {} has no due date to postpone",
            id
        ))
    })?;

    let new_due = current_due + Duration::days(days as i64);
    let updated = task.with_due_date(Some(new_due));
    list.update(idx, updated.clone())?;
    list.save()?;

    renderer.print_write_result(
        &format!(
            "Postponed task #{} by {} day(s) to {}.",
            id,
            days,
            new_due.format("%Y-%m-%d")
        ),
        idx,
        &updated,
    );

    Ok(())
}

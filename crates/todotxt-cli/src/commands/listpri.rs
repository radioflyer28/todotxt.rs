use crate::{cli::ListpriArgs, config::Config, output::Renderer, CliError};
use std::path::Path;
use todotxt_core::{Filter, TaskList};

/// Parse a priority spec: single letter "A" or range "A-C".
/// Returns an inclusive range (start_char, end_char).
fn parse_priority_range(spec: &str) -> Result<(char, char), CliError> {
    let spec = spec.trim().to_uppercase();
    if spec.len() == 1 {
        let c = spec.chars().next().unwrap();
        if c.is_ascii_alphabetic() {
            return Ok((c, c));
        }
    } else if spec.len() == 3 && spec.as_bytes()[1] == b'-' {
        let mut chars = spec.chars();
        let start = chars.next().unwrap();
        chars.next(); // skip '-'
        let end = chars.next().unwrap();
        if start.is_ascii_alphabetic() && end.is_ascii_alphabetic() && start <= end {
            return Ok((start, end));
        }
    }
    Err(CliError::Other(anyhow::anyhow!(
        "invalid priority spec '{}': expected single letter (A) or range (A-C)",
        spec
    )))
}

pub fn run(
    todo_path: &Path,
    args: &ListpriArgs,
    _cfg: &Config,
    renderer: &Renderer,
) -> Result<(), CliError> {
    let list = TaskList::load(todo_path)?;

    let (start, end) = match &args.priorities {
        Some(spec) => parse_priority_range(spec)?,
        None => ('A', 'Z'),
    };

    // Default filter: incomplete tasks only
    let filter = Filter::from_query("-DONE");
    let tasks = list.filter(&filter);

    // Post-filter: keep only tasks whose priority falls in [start, end]
    let prioritized: Vec<(usize, &todotxt_core::Task)> = tasks
        .into_iter()
        .filter(|(_id, task)| matches!(task.priority, Some(p) if p >= start && p <= end))
        .collect();

    renderer.print_tasks(&prioritized);
    renderer.print_count(prioritized.len());
    Ok(())
}

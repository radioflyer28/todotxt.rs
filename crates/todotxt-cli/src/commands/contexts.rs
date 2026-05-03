use crate::{output::Renderer, CliError};
use std::{collections::BTreeSet, path::Path};
use todotxt_core::TaskList;

pub fn run(todo_path: &Path, renderer: &Renderer) -> Result<(), CliError> {
    let list = TaskList::load(todo_path)?;
    let contexts: BTreeSet<String> = list
        .tasks()
        .iter()
        .flat_map(|t| t.contexts.iter().cloned())
        .collect();
    let items: Vec<String> = contexts.into_iter().map(|c| format!("@{}", c)).collect();
    renderer.print_lines(&items);
    Ok(())
}

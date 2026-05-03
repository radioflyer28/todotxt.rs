use crate::{output::Renderer, CliError};
use std::{collections::BTreeSet, path::Path};
use todotxt_core::TaskList;

pub fn run(todo_path: &Path, renderer: &Renderer) -> Result<(), CliError> {
    let list = TaskList::load(todo_path)?;
    let projects: BTreeSet<String> = list
        .tasks()
        .iter()
        .flat_map(|t| t.projects.iter().cloned())
        .collect();
    let items: Vec<String> = projects.into_iter().map(|p| format!("+{}", p)).collect();
    renderer.print_lines(&items);
    Ok(())
}

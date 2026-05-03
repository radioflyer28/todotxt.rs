use crate::{output::Renderer, CliError};
use std::{collections::HashSet, path::Path};
use todotxt_core::TaskList;

pub fn run(todo_path: &Path, renderer: &Renderer) -> Result<(), CliError> {
    let mut list = TaskList::load(todo_path)?;

    // Find duplicate indices: keep first occurrence, mark later ones for removal
    let mut seen: HashSet<String> = HashSet::new();
    let mut to_remove: Vec<usize> = Vec::new();

    for (idx, task) in list.tasks().iter().enumerate() {
        let raw = task.to_raw().to_owned();
        if seen.contains(&raw) {
            to_remove.push(idx);
        } else {
            seen.insert(raw);
        }
    }

    let removed_count = to_remove.len();

    if removed_count == 0 {
        if !renderer.quiet {
            println!("No duplicate tasks found.");
        }
        return Ok(());
    }

    // Delete in reverse order so earlier indices remain valid after each removal
    for idx in to_remove.into_iter().rev() {
        list.delete(idx)?;
    }

    if !renderer.quiet {
        println!(
            "Removed {} duplicate task{}.",
            removed_count,
            if removed_count == 1 { "" } else { "s" }
        );
    }

    Ok(())
}

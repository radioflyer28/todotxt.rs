use crate::{cli::ListArgs, config::Config, output::Renderer, CliError};
use std::path::Path;
use todotxt_core::{Filter, TaskList};

pub fn run(todo_path: &Path, _args: &ListArgs, cfg: &Config, renderer: &Renderer) -> Result<(), CliError> {
    // Load todo.txt
    let todo_list = TaskList::load(todo_path)?;

    // Resolve done.txt path (same pattern as archive.rs)
    let done_path = cfg
        .done_file
        .clone()
        .unwrap_or_else(|| todo_path.parent().unwrap_or(Path::new(".")).join("done.txt"));

    // Show all tasks — no threshold or hidden suppression
    let no_filter = Filter {
        suppress_hidden: false,
        suppress_future_threshold: false,
        ..Filter::default()
    };

    let todo_tasks = todo_list.filter(&no_filter);

    // Load done.txt if it exists; missing file is treated as empty
    if done_path.exists() {
        let done_list = TaskList::load(&done_path)?;
        let done_tasks = done_list.filter(&no_filter);

        // Print both sections merged
        let total = todo_tasks.len() + done_tasks.len();
        renderer.print_tasks(&todo_tasks);
        renderer.print_tasks(&done_tasks);
        renderer.print_count(total);
    } else {
        renderer.print_tasks(&todo_tasks);
        renderer.print_count(todo_tasks.len());
    }

    Ok(())
}

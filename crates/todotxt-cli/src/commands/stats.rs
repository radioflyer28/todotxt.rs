use crate::{
    output::{Renderer, Stats},
    CliError,
};
use std::path::Path;
use todotxt_core::{Filter, TaskList};

pub fn run(todo_path: &Path, renderer: &Renderer) -> Result<(), CliError> {
    let list = TaskList::load(todo_path)?;
    let all = list.filter(&Filter::new());
    let today = chrono::Local::now().date_naive();

    let mut stats = Stats {
        total: all.len(),
        complete: 0,
        incomplete: 0,
        due_today: 0,
        overdue: 0,
    };

    for (_, task) in &all {
        if task.completed {
            stats.complete += 1;
        } else {
            stats.incomplete += 1;
            if let Some(due) = task.due_date {
                if due == today {
                    stats.due_today += 1;
                } else if due < today {
                    stats.overdue += 1;
                }
            }
        }
    }

    renderer.print_stats(&stats);
    Ok(())
}

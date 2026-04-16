use crate::{config::Config, output::json_success, output::Renderer, CliError};
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;
use todotxt_core::TaskList;

/// Move all completed tasks from todo.txt to done.txt (`archive`).
///
/// Both files are written atomically via temp-file rename. Idempotent —
/// 0 completed tasks exits 0 and writes nothing (D-04).
pub fn run_archive(todo_path: &Path, cfg: &Config, renderer: &Renderer) -> Result<(), CliError> {
    let list = TaskList::load(todo_path)?;

    let completed: Vec<_> = list.tasks().iter().filter(|t| t.completed).cloned().collect();
    let incomplete: Vec<_> = list.tasks().iter().filter(|t| !t.completed).cloned().collect();
    let count = completed.len();

    // Resolve done.txt path from config or as sibling of todo.txt.
    let done_path = cfg
        .done_file
        .clone()
        .unwrap_or_else(|| todo_path.parent().unwrap_or(Path::new(".")).join("done.txt"));

    // Ensure done.txt parent directory exists.
    if let Some(parent) = done_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Build done.txt content: existing lines + newly archived tasks.
    let existing_done = if done_path.exists() {
        std::fs::read_to_string(&done_path)?
    } else {
        String::new()
    };

    let new_done_content = if count == 0 {
        existing_done.clone()
    } else {
        let appended = completed.iter().map(|t| t.to_raw()).collect::<Vec<_>>().join("\n");
        if existing_done.is_empty() {
            format!("{appended}\n")
        } else {
            let base = existing_done.trim_end_matches('\n');
            format!("{base}\n{appended}\n")
        }
    };

    // Atomic write: done.txt
    let done_parent = done_path.parent().unwrap_or(Path::new("."));
    let mut temp_done = NamedTempFile::new_in(done_parent)?;
    temp_done.write_all(new_done_content.as_bytes())?;
    temp_done.flush()?;
    temp_done.as_file().sync_all()?;
    temp_done
        .persist(&done_path)
        .map_err(|e| CliError::Other(anyhow::anyhow!("{}", e.error)))?;

    // Atomic write: todo.txt (incomplete tasks only).
    let new_todo_content = if incomplete.is_empty() {
        String::new()
    } else {
        format!(
            "{}\n",
            incomplete.iter().map(|t| t.to_raw()).collect::<Vec<_>>().join("\n")
        )
    };
    let todo_parent = todo_path.parent().unwrap_or(Path::new("."));
    let mut temp_todo = NamedTempFile::new_in(todo_parent)?;
    temp_todo.write_all(new_todo_content.as_bytes())?;
    temp_todo.flush()?;
    temp_todo.as_file().sync_all()?;
    temp_todo
        .persist(todo_path)
        .map_err(|e| CliError::Other(anyhow::anyhow!("{}", e.error)))?;

    if renderer.json {
        println!("{}", json_success(serde_json::json!({ "count": count })));
    } else if !renderer.quiet {
        eprintln!(
            "Archived {} completed task{}.",
            count,
            if count == 1 { "" } else { "s" }
        );
    }

    Ok(())
}

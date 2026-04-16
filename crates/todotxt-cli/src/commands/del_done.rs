use crate::{output::json_success, output::Renderer, CliError};
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;
use todotxt_core::TaskList;

/// Delete all completed tasks from todo.txt in-place (`del-done`).
///
/// Idempotent — 0 completed tasks exits 0 without modifying the file.
pub fn run_del_done(todo_path: &Path, renderer: &Renderer) -> Result<(), CliError> {
    let list = TaskList::load(todo_path)?;

    let count = list.tasks().iter().filter(|t| t.completed).count();
    let incomplete: Vec<_> = list.tasks().iter().filter(|t| !t.completed).cloned().collect();

    let new_content = if incomplete.is_empty() {
        String::new()
    } else {
        format!(
            "{}\n",
            incomplete.iter().map(|t| t.to_raw()).collect::<Vec<_>>().join("\n")
        )
    };

    let todo_parent = todo_path.parent().unwrap_or(Path::new("."));
    let mut temp = NamedTempFile::new_in(todo_parent)?;
    temp.write_all(new_content.as_bytes())?;
    temp.flush()?;
    temp.as_file().sync_all()?;
    temp.persist(todo_path)
        .map_err(|e| CliError::Other(anyhow::anyhow!("{}", e.error)))?;

    if renderer.json {
        println!("{}", json_success(serde_json::json!({ "count": count })));
    } else if !renderer.quiet {
        eprintln!(
            "Deleted {} completed task{}.",
            count,
            if count == 1 { "" } else { "s" }
        );
    }

    Ok(())
}

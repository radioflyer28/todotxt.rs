use crate::{config::Config, output::json_success, output::Renderer, CliError};
use chrono::Local;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use todotxt_core::{plan_archive_rotation, TaskList};

fn append_archive_content(existing: &str, addition: &str) -> String {
    let mut parts = Vec::new();
    for part in [existing, addition] {
        let normalized = part.trim_end_matches('\n');
        if !normalized.is_empty() {
            parts.push(normalized);
        }
    }

    if parts.is_empty() {
        String::new()
    } else {
        format!("{}\n", parts.join("\n"))
    }
}

fn write_text_atomically(path: &Path, content: &str) -> Result<(), CliError> {
    let parent = path.parent().unwrap_or(Path::new("."));
    let mut temp = NamedTempFile::new_in(parent)?;
    temp.write_all(content.as_bytes())?;
    temp.flush()?;
    temp.as_file().sync_all()?;
    temp.persist(path)
        .map_err(|e| CliError::Other(anyhow::anyhow!("{}", e.error)))?;
    Ok(())
}

/// Move all completed tasks from todo.txt to done.txt (`archive`).
///
/// Both files are written atomically via temp-file rename. Idempotent —
/// 0 completed tasks exits 0 and writes nothing (D-04).
pub fn run_archive(todo_path: &Path, cfg: &Config, renderer: &Renderer) -> Result<(), CliError> {
    let list = TaskList::load(todo_path)?;

    let completed: Vec<_> = list
        .tasks()
        .iter()
        .filter(|t| t.completed)
        .cloned()
        .collect();
    let incomplete: Vec<_> = list
        .tasks()
        .iter()
        .filter(|t| !t.completed)
        .cloned()
        .collect();
    let count = completed.len();
    if count == 0 {
        if renderer.json {
            println!(
                "{}",
                json_success(serde_json::json!({ "count": 0, "rotated_to": null }))
            );
        } else if !renderer.quiet {
            eprintln!("Archived 0 completed tasks.");
        }
        return Ok(());
    }

    // Resolve done.txt path from config or as sibling of todo.txt.
    let done_path = cfg.done_file.clone().unwrap_or_else(|| {
        todo_path
            .parent()
            .unwrap_or(Path::new("."))
            .join("done.txt")
    });

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

    let existing_modified = if done_path.exists() {
        Some(std::fs::metadata(&done_path)?.modified()?)
    } else {
        None
    };
    let decision = plan_archive_rotation(
        &done_path,
        cfg.archive_rotation_cadence,
        Local::now().date_naive(),
        existing_modified,
        !existing_done.trim().is_empty(),
    );

    let rotated_to: Option<PathBuf> = if let Some(rotated_path) = decision.rotated_path.clone() {
        let existing_rotated = if rotated_path.exists() {
            std::fs::read_to_string(&rotated_path)?
        } else {
            String::new()
        };
        let rotated_content = append_archive_content(&existing_rotated, &existing_done);
        write_text_atomically(&rotated_path, &rotated_content)?;
        Some(rotated_path)
    } else {
        None
    };
    let active_existing = if rotated_to.is_some() {
        String::new()
    } else {
        existing_done.clone()
    };
    let appended = completed
        .iter()
        .map(|t| t.to_raw())
        .collect::<Vec<_>>()
        .join("\n");
    let new_done_content = append_archive_content(&active_existing, &appended);

    // Atomic write: done.txt
    write_text_atomically(&done_path, &new_done_content)?;

    // Atomic write: todo.txt (incomplete tasks only).
    let new_todo_content = if incomplete.is_empty() {
        String::new()
    } else {
        format!(
            "{}\n",
            incomplete
                .iter()
                .map(|t| t.to_raw())
                .collect::<Vec<_>>()
                .join("\n")
        )
    };
    write_text_atomically(todo_path, &new_todo_content)?;

    if renderer.json {
        println!(
            "{}",
            json_success(serde_json::json!({
                "count": count,
                "rotated_to": rotated_to.as_ref().map(|path| path.display().to_string())
            }))
        );
    } else if !renderer.quiet {
        let task_suffix = if count == 1 { "" } else { "s" };
        if let Some(rotated_path) = rotated_to {
            eprintln!(
                "Archived {} completed task{}. Rotated previous done.txt to {}.",
                count,
                task_suffix,
                rotated_path.display()
            );
        } else {
            eprintln!("Archived {} completed task{}.", count, task_suffix);
        }
    }

    Ok(())
}

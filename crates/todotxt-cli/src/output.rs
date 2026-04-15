use comfy_table::presets::NOTHING;
use comfy_table::{Cell, ContentArrangement, Table};
use owo_colors::colors::{Green, Red, White, Yellow};
use owo_colors::{OwoColorize, Stream};
use serde::Serialize;
use serde_json::json;
use todotxt_core::Task;

/// Initialize global color support.
///
/// Must be called once at startup, after parsing `--no-color` flag.
/// Respects both `--no-color` CLI flag and `NO_COLOR` environment variable.
pub fn init_color(no_color: bool) {
    let env_no_color = std::env::var_os("NO_COLOR").is_some();
    if no_color || env_no_color {
        owo_colors::set_override(false);
    }
}

/// Central output renderer — carries global output flags.
pub struct Renderer {
    pub json: bool,
    pub quiet: bool,
}

impl Renderer {
    pub fn new(json: bool, quiet: bool) -> Self {
        Self { json, quiet }
    }

    /// Print an informational notice to stderr (suppressed by `--quiet`).
    pub fn info(&self, msg: &str) {
        if !self.quiet {
            eprintln!("{}", msg);
        }
    }

    /// Print an error to stderr. Never suppressed by `--quiet`.
    pub fn error(&self, msg: &str) {
        eprintln!("error: {}", msg);
    }

    /// Print a list of tasks — as JSON or as a human table.
    pub fn print_tasks(&self, tasks: &[(usize, &Task)]) {
        if self.json {
            let dtos: Vec<_> = tasks.iter().map(|(idx, t)| task_dto(*idx, t)).collect();
            println!("{}", json_success(dtos));
        } else {
            if tasks.is_empty() {
                if !self.quiet {
                    eprintln!("No tasks found.");
                }
                return;
            }
            let table = build_task_table(tasks);
            println!("{table}");
        }
    }

    /// Print a "N tasks found" footer (suppressed by `--quiet` or `--json`).
    pub fn print_count(&self, n: usize) {
        if !self.quiet && !self.json {
            eprintln!("-- {} task{} --", n, if n == 1 { "" } else { "s" });
        }
    }
}

/// Produce the standard JSON success envelope: `{"schema_version":1,"data":<T>}`.
pub fn json_success<T: Serialize>(data: T) -> String {
    serde_json::to_string(&json!({
        "schema_version": 1,
        "data": data
    }))
    .expect("JSON serialization should not fail")
}

/// Produce the standard JSON error envelope: `{"schema_version":1,"error":"<msg>"}`.
pub fn json_error(msg: &str) -> String {
    serde_json::to_string(&json!({
        "schema_version": 1,
        "error": msg
    }))
    .expect("JSON serialization should not fail")
}

// ── Private helpers ──────────────────────────────────────────────────────────

/// DTO for task JSON serialization (avoids deriving on the core Task type).
#[derive(Serialize)]
struct TaskDto {
    id: usize,
    raw: String,
    completed: bool,
    priority: Option<char>,
    projects: Vec<String>,
    contexts: Vec<String>,
    due_date: Option<String>,
}

fn task_dto(idx: usize, task: &Task) -> TaskDto {
    TaskDto {
        id: idx + 1, // 1-based display ID
        raw: task.to_raw().to_string(),
        completed: task.completed,
        priority: task.priority,
        projects: task.projects.clone(),
        contexts: task.contexts.clone(),
        due_date: task.due_date.map(|d| d.format("%Y-%m-%d").to_string()),
    }
}

/// Build a comfy-table with NOTHING preset (D-07: header row only, no borders).
fn build_task_table(tasks: &[(usize, &Task)]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(NOTHING)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["ID", "PRI", "Task"]);

    for (idx, task) in tasks {
        let id = idx + 1;
        let pri_cell = match task.priority {
            Some('A') => "(A)"
                .if_supports_color(Stream::Stdout, |t| t.fg::<Red>())
                .to_string(),
            Some('B') => "(B)"
                .if_supports_color(Stream::Stdout, |t| t.fg::<Yellow>())
                .to_string(),
            Some('C') => "(C)"
                .if_supports_color(Stream::Stdout, |t| t.fg::<Green>())
                .to_string(),
            Some(p) => {
                let s = format!("({p})");
                s.if_supports_color(Stream::Stdout, |t| t.fg::<White>())
                    .to_string()
            }
            None => String::new(),
        };
        table.add_row(vec![
            Cell::new(id.to_string()),
            Cell::new(pri_cell),
            Cell::new(task.to_raw().to_string()),
        ]);
    }
    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_success_has_schema_version() {
        let output = json_success(serde_json::json!([]));
        assert!(output.contains("\"schema_version\":1"));
        assert!(output.contains("\"data\":"));
    }

    #[test]
    fn json_error_has_schema_version() {
        let output = json_error("something went wrong");
        assert!(output.contains("\"schema_version\":1"));
        assert!(output.contains("\"error\":"));
        assert!(output.contains("something went wrong"));
    }

    #[test]
    fn renderer_info_suppressed_by_quiet() {
        // Smoke-test: just ensure it compiles and doesn't panic
        let r = Renderer::new(false, true);
        r.info("this should be suppressed");
    }

    #[test]
    fn renderer_error_never_suppressed() {
        // Smoke-test: just ensure it compiles and doesn't panic
        let r = Renderer::new(false, true);
        r.error("this is always shown");
    }
}

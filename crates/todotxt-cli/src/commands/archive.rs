use crate::{output::Renderer, CliError};
use std::path::Path;
use crate::config::Config;

/// Archive all completed tasks to done.txt (`archive`).
pub fn run_archive(
    _todo_path: &Path,
    _cfg: &Config,
    _renderer: &Renderer,
) -> Result<(), CliError> {
    // TODO: Implement archive logic
    // - Resolve done_file path from config (default: sibling of todo.txt)
    // - Load TaskList from todo.txt
    // - Separate completed vs incomplete tasks
    // - If any completed tasks:
    //   - Write incomplete tasks back to todo.txt
    //   - Append completed tasks to done.txt (or create if missing)
    //   - Atomic two-file write (temp + rename pattern)
    // - Print "{count} tasks archived" to stderr
    // - Exit 0 (even if 0 tasks archived)

    todo!("run_archive not yet implemented")
}

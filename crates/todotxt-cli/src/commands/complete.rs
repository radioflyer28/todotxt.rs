use crate::{output::Renderer, CliError};
use std::path::Path;

pub fn run_do(todo_path: &Path, ids: &[usize], renderer: &Renderer) -> Result<(), CliError> {
    let _ = (todo_path, ids, renderer);
    todo!("do command not yet implemented")
}

pub fn run_undo(todo_path: &Path, ids: &[usize], renderer: &Renderer) -> Result<(), CliError> {
    let _ = (todo_path, ids, renderer);
    todo!("undo command not yet implemented")
}

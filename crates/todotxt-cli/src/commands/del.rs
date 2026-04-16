use crate::{output::Renderer, CliError};
use std::path::Path;

pub fn run(todo_path: &Path, ids: &[usize], renderer: &Renderer) -> Result<(), CliError> {
    let _ = (todo_path, ids, renderer);
    todo!("del command not yet implemented")
}

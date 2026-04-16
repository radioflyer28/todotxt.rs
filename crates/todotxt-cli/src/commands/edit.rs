use crate::{output::Renderer, CliError};
use std::path::Path;

pub fn run(todo_path: &Path, id: usize, text: &str, renderer: &Renderer) -> Result<(), CliError> {
    let _ = (todo_path, id, text, renderer);
    todo!("edit command not yet implemented")
}

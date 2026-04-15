use crate::{cli::ListArgs, config::Config, output::Renderer, CliError};
use std::path::Path;

pub fn run(_todo_path: &Path, _args: &ListArgs, _cfg: &Config, _renderer: &Renderer) -> Result<(), CliError> {
    todo!("implemented in task 3")
}

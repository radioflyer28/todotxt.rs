use clap::CommandFactory;
use clap_complete::{generate, Shell};

use crate::cli::Cli;

/// Generate shell completions and write to stdout.
///
/// Supports all shells provided by `clap_complete::Shell`:
/// Bash, Zsh, Fish, PowerShell, Elvish.
pub fn run(shell: Shell) {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "todotxt", &mut std::io::stdout());
}

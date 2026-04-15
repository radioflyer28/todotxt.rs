mod cli;
mod commands;
mod config;
mod output;

use clap::Parser;
use cli::{Cli, Commands};
use std::process;

/// CLI-specific error type encoding exit code semantics.
pub enum CliError {
    /// Target not found — exit code 1 (e.g., task ID out of range in `show`)
    NotFound(String),
    /// All other errors — exit code 2
    Other(anyhow::Error),
}

impl<E: Into<anyhow::Error>> From<E> for CliError {
    fn from(e: E) -> Self {
        CliError::Other(e.into())
    }
}

fn run(cli: &Cli) -> Result<(), CliError> {
    // Resolve config path (portable mode or explicit --config)
    let config_path = if let Some(explicit) = &cli.config {
        explicit.clone()
    } else {
        let platform = config::Config::default_path()
            .ok_or_else(|| CliError::Other(anyhow::anyhow!("cannot determine config directory")))?;
        config::Config::resolve_path(&platform)
    };

    let cfg = config::Config::load_or_create(&config_path)?;

    // Resolve todo_file: --todo-file flag overrides config (D-01)
    let todo_path = if let Some(explicit) = &cli.todo_file {
        explicit.clone()
    } else {
        cfg.resolve_todo_file()?
    };

    let renderer = output::Renderer::new(cli.json, cli.quiet);

    match &cli.command {
        Commands::List(args) => commands::list::run(&todo_path, args, &cfg, &renderer)?,
        Commands::Stats => commands::stats::run(&todo_path, &renderer)?,
        Commands::Projects => commands::projects::run(&todo_path, &renderer)?,
        Commands::Contexts => commands::contexts::run(&todo_path, &renderer)?,
        Commands::Show { id } => commands::show::run(&todo_path, *id, &renderer)?,
        Commands::Completions { shell } => commands::completions::run(*shell),
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    // P-02: init_color BEFORE any output or dispatch
    output::init_color(cli.no_color);

    match run(&cli) {
        Ok(()) => {}
        Err(CliError::NotFound(msg)) => {
            if cli.json {
                println!("{}", output::json_error(&msg));
            } else {
                eprintln!("error: {}", msg);
            }
            process::exit(1);
        }
        Err(CliError::Other(e)) => {
            if cli.json {
                println!("{}", output::json_error(&e.to_string()));
            } else {
                eprintln!("error: {}", e);
            }
            process::exit(2);
        }
    }
}

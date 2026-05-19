#![deny(warnings)]

mod cli;
mod commands;
mod config;
pub mod date;
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
        Commands::Add(args) => commands::add::run(
            &todo_path,
            &args.text,
            args.date,
            args.no_date,
            &cfg,
            &renderer,
        )?,
        Commands::Do { ids } => commands::complete::run_do(&todo_path, ids, &renderer)?,
        Commands::Undo { ids } => commands::complete::run_undo(&todo_path, ids, &renderer)?,
        Commands::Del { ids } => commands::del::run(&todo_path, ids, &renderer)?,
        Commands::Edit { id, text } => commands::edit::run(&todo_path, *id, text, &renderer)?,
        Commands::Append {
            id,
            text,
            normalize,
        } => commands::append::run(&todo_path, *id, text, *normalize, &renderer)?,
        Commands::Prepend { id, text } => commands::prepend::run(&todo_path, *id, text, &renderer)?,
        Commands::Pri { ids, priority } => {
            commands::priority::run_pri(&todo_path, ids, *priority, &renderer)?
        }
        Commands::Depri { ids } => commands::priority::run_depri(&todo_path, ids, &renderer)?,
        Commands::Due { id, date } => commands::due::run_due(&todo_path, *id, date, &renderer)?,
        Commands::Postpone { id, days } => {
            commands::due::run_postpone(&todo_path, *id, *days, &renderer)?
        }
        Commands::Archive => commands::archive::run_archive(&todo_path, &cfg, &renderer)?,
        Commands::DelDone => commands::del_done::run_del_done(&todo_path, &renderer)?,
        Commands::Listpri(args) => commands::listpri::run(&todo_path, args, &cfg, &renderer)?,
        Commands::Listall(args) => commands::listall::run(&todo_path, args, &cfg, &renderer)?,
        Commands::Deduplicate => commands::deduplicate::run(&todo_path, &renderer)?,
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

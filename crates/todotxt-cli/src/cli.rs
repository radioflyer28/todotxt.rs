use clap::{Args, Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;

/// todotxt — a todo.txt command-line manager
#[derive(Parser, Debug)]
#[command(name = "todotxt", version, about, long_about = None)]
pub struct Cli {
    // ── Global flags (P-07: on Cli struct, not on subcommands) ────────────
    /// Path to todo.txt file (overrides config todo_file)
    #[arg(long, global = true)]
    pub todo_file: Option<PathBuf>,

    /// Path to config file (overrides platform default and portable detection)
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// Output results as JSON envelope {"schema_version":1,"data":...}
    #[arg(long, global = true)]
    pub json: bool,

    /// Disable ANSI color output (also respected via NO_COLOR env var)
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Suppress all non-data output (errors still go to stderr)
    #[arg(long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List tasks matching optional filters
    #[command(alias = "ls")]
    List(ListArgs),

    /// Show summary statistics
    Stats,

    /// List all unique project tags (+project)
    Projects,

    /// List all unique context tags (@context)
    Contexts,

    /// Print the raw todo.txt line for a task by 1-based ID
    Show {
        /// 1-based task ID (from list output)
        id: usize,
    },

    /// Generate shell completions and print to stdout
    Completions {
        /// Target shell
        #[arg(value_enum)]
        shell: Shell,
    },
}

/// Arguments for the `list` / `ls` subcommand.
///
/// Filter tokens are AND-combined. Tokens starting with `:` are preset names.
#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter tokens: plain words, +project, @context, due:today, DONE, -DONE, :preset_name
    /// Multiple tokens are AND-combined.
    pub filters: Vec<String>,

    /// Additional filter query with spaces (combined AND with positional filters)
    #[arg(long = "filter", short = 'f')]
    pub filter_query: Option<String>,
}

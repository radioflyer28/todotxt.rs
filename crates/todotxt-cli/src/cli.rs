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

    /// Add a new task to todo.txt
    Add(AddArgs),

    /// Mark one or more tasks as complete (prepends "x YYYY-MM-DD ")
    #[command(name = "do")]
    Do {
        /// One or more 1-based task IDs
        ids: Vec<usize>,
    },

    /// Unmark one or more completed tasks (removes "x YYYY-MM-DD " prefix)
    Undo {
        /// One or more 1-based task IDs
        ids: Vec<usize>,
    },

    /// Delete one or more tasks by 1-based ID
    #[command(alias = "delete")]
    Del {
        /// One or more 1-based task IDs
        ids: Vec<usize>,
    },

    /// Replace a task's full text
    Edit {
        /// 1-based task ID
        id: usize,
        /// Replacement text (entire line; no creation date injected)
        text: String,
    },

    /// Append text to the end of a task
    Append {
        /// 1-based task ID
        id: usize,
        /// Text to append (leading space added automatically)
        text: String,
    },

    /// Prepend text before a task's body (after priority/date prefixes)
    Prepend {
        /// 1-based task ID
        id: usize,
        /// Text to prepend (trailing space added automatically)
        text: String,
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

/// Arguments for the `add` subcommand.
#[derive(Args, Debug)]
pub struct AddArgs {
    /// Full task text (e.g., "Buy milk +groceries @home")
    pub text: String,
    /// Force-prepend today's creation date (YYYY-MM-DD) regardless of config
    #[arg(long)]
    pub date: bool,
    /// Suppress creation date even when auto_creation_date = true in config
    #[arg(long)]
    pub no_date: bool,
}

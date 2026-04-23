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
    #[command(alias = "lsprj")]
    Projects,

    /// List all unique context tags (@context)
    #[command(alias = "lsc")]
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
    #[command(alias = "a")]
    Add(AddArgs),

    /// Mark one or more tasks as complete (prepends "x YYYY-MM-DD ")
    #[command(name = "do", alias = "done")]
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
    #[command(aliases = ["delete", "rm"])]
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
    #[command(alias = "app")]
    Append {
        /// 1-based task ID
        id: usize,
        /// Text to append (leading space added automatically)
        text: String,
    },

    /// Prepend text before a task's body (after priority/date prefixes)
    #[command(alias = "prep")]
    Prepend {
        /// 1-based task ID
        id: usize,
        /// Text to prepend (trailing space added automatically)
        text: String,
    },

    /// Set priority (A-Z) for one or more tasks
    #[command(name = "pri", alias = "p")]
    Pri {
        /// Priority letter (A-Z)
        priority: char,
        /// One or more 1-based task IDs
        #[arg(trailing_var_arg = true)]
        ids: Vec<usize>,
    },

    /// Remove priority from one or more tasks
    #[command(alias = "dp")]
    Depri {
        /// One or more 1-based task IDs
        ids: Vec<usize>,
    },

    /// Set due date on a task
    Due {
        /// 1-based task ID
        id: usize,
        /// Date string (today, tomorrow, YYYY-MM-DD, or weekday)
        date: String,
    },

    /// Move task's due date forward by N days
    Postpone {
        /// 1-based task ID
        id: usize,
        /// Number of days to postpone
        days: u32,
    },

    /// Archive all completed tasks to done.txt
    Archive,

    /// Delete all completed tasks from todo.txt
    #[command(name = "del-done")]
    DelDone,

    /// List tasks filtered by priority (A-Z or range like A-C). Default: A-Z.
    #[command(name = "listpri", alias = "lsp")]
    Listpri(ListpriArgs),

    /// List tasks from both todo.txt and done.txt, merged.
    #[command(name = "listall", alias = "lsa")]
    Listall(ListArgs),

    /// Remove exact duplicate lines from todo.txt.
    #[command(name = "deduplicate")]
    Deduplicate,
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

    /// Show all tasks including deferred (future t:) and hidden (h:1)
    #[arg(long)]
    pub all: bool,

    /// Emit todo.sh-style numbered plain-text output ({N} {raw_task})
    #[arg(long)]
    pub compat: bool,
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

/// Arguments for the `listpri` / `lsp` subcommand.
#[derive(Args, Debug)]
pub struct ListpriArgs {
    /// Priority filter: single letter (A) or range (A-C). Defaults to A-Z if omitted.
    pub priorities: Option<String>,
}


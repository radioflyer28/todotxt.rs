# Phase 3 Research: CLI Foundation — Config + Output + Read Commands

**Phase:** 03 — CLI Foundation — Config + Output + Read Commands
**Researched:** 2026-04-15
**Requirements Addressed:** READ-01–08, CFG-01–02, PLAT-01

---

## Standard Stack

### Verified Crate Versions

| Crate | Version | Purpose |
|-------|---------|---------|
| `clap` | 4.6 | CLI parsing — derive API, subcommands, global flags |
| `clap_complete` | 4.6.2 | Shell completion script generation |
| `anyhow` | 1.0 | Error propagation in CLI; exit-code mapping |
| `owo-colors` | 4.3.0 | Terminal colorization + NO_COLOR/tty detection |
| `comfy-table` | 7.2.2 | Table rendering with header, presets, dynamic width |
| `serde_json` | 1 (workspace) | JSON output envelope serialization |
| `directories` | 6.0.0 | Platform-appropriate config/data directory paths |
| `toml` | 0.8 | TOML config file parsing/serialization |
| `assert_cmd` | 2.2.0 | Integration testing of CLI binaries |
| `assert_fs` | 1.x | Temporary filesystem fixtures for integration tests |
| `predicates` | 3.x | Output predicates used with assert_cmd assertions |

### Add to Workspace `[workspace.dependencies]`

```toml
clap = { version = "4.6", features = ["derive"] }
clap_complete = "4.6"
anyhow = "1.0"
owo-colors = { version = "4", features = ["supports-colors"] }
comfy-table = "7"
directories = "6"
toml = "0.8"
assert_cmd = "2"
assert_fs = "1"
predicates = "3"
```

---

## Architecture Patterns

### CLI Crate Structure

```
crates/todotxt-cli/
├── src/
│   ├── main.rs           # Entry point: parse args, dispatch, map exit codes
│   ├── cli.rs            # clap derive structs (Cli, Commands, global flags)
│   ├── config.rs         # Config struct, TOML load/save, portable mode
│   ├── output.rs         # Renderer (human table + JSON envelope)
│   └── commands/
│       ├── mod.rs
│       ├── list.rs       # list / ls
│       ├── stats.rs      # stats
│       ├── projects.rs   # projects
│       ├── contexts.rs   # contexts
│       ├── show.rs       # show <id>
│       └── completions.rs # completions <shell>
├── tests/
│   ├── list_tests.rs
│   ├── stats_tests.rs
│   ├── show_tests.rs
│   ├── config_tests.rs
│   └── completions_tests.rs
└── Cargo.toml
```

### clap 4.6 Derive Pattern

```rust
// cli.rs — top-level derive struct
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "todotxt", version, about)]
pub struct Cli {
    /// Path to todo.txt file (overrides config)
    #[arg(long, global = true, value_name = "FILE")]
    pub todo_file: Option<PathBuf>,

    /// Path to config file (overrides default location)
    #[arg(long, global = true, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Output as JSON ({"schema_version":1,"data":...})
    #[arg(long, global = true)]
    pub json: bool,

    /// Suppress ANSI color output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Suppress all non-data output (headers, notices)
    #[arg(long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List tasks (alias: ls)
    #[command(alias = "ls")]
    List(ListArgs),
    Stats(StatsArgs),
    Projects,
    Contexts,
    Show { id: usize },
    Completions { shell: clap_complete::aot::Shell },
}
```

**Key clap 4.6 notes:**
- `global = true` on args propagates them to all subcommands automatically
- `#[command(alias = "ls")]` gives `list` its `ls` alias
- `value_parser(value_parser!(Shell))` works automatically when `Shell` is the field type

### clap_complete Pattern (runtime generation)

```rust
// commands/completions.rs
use clap::CommandFactory;
use clap_complete::aot::{generate, Shell};
use std::io;

pub fn run(shell: Shell, cli_type: &mut impl clap::CommandFactory) {
    let mut cmd = <YourCli as CommandFactory>::command();
    generate(shell, &mut cmd, "todotxt", &mut io::stdout());
}
```

Shell variants: `Shell::Bash`, `Shell::Zsh`, `Shell::Fish`, `Shell::PowerShell`

### Config Pattern (directories + toml + serde)

```rust
// config.rs
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    pub todo_file: Option<PathBuf>,
    #[serde(default)]
    pub presets: HashMap<String, PresetConfig>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct PresetConfig {
    pub filter: Option<String>,
}

impl Config {
    /// Load from path. Creates default config at that path if it doesn't exist.
    pub fn load_or_create(path: &Path) -> anyhow::Result<Self> { ... }

    /// Platform config path via directories crate.
    pub fn default_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "todotxt")
            .map(|dirs| dirs.config_dir().join("config.toml"))
    }
}
```

**directories 6.0 API:**
- `ProjectDirs::from("qualifier", "organization", "application") -> Option<ProjectDirs>`
- `dirs.config_dir() -> &Path` → platform config directory
  - Linux: `~/.config/todotxt/`
  - Windows: `%APPDATA%\todotxt\`
  - macOS: `~/Library/Application Support/todotxt/`
- Returns `None` if no home directory is found (rare edge case)

**Portable mode integration:**
- Call `todotxt_core::resolve_config_path(binary_dir, platform_dir)` where `binary_dir` is the
  directory of `std::env::current_exe()` and `platform_dir` is from `ProjectDirs::config_dir()`
- If a `config.toml` exists beside the binary, that path is used; otherwise the platform path

**TOML parsing:**
```rust
let content = std::fs::read_to_string(&path)?;
let config: Config = toml::from_str(&content)?;
```

**Auto-create on first run:**
```rust
// When config file doesn't exist:
let default_config = Config {
    todo_file: dirs::home_dir().map(|h| h.join("todo.txt")),
    presets: HashMap::new(),
};
let toml_str = toml::to_string_pretty(&default_config)?;
std::fs::create_dir_all(config_path.parent().unwrap())?;
std::fs::write(&config_path, toml_str)?;
```

### Output / Renderer Pattern

```rust
// output.rs
pub struct Renderer {
    pub json: bool,
    pub no_color: bool,
    pub quiet: bool,
}

impl Renderer {
    /// Write data to stdout (tasks, stats, etc.)
    pub fn print_tasks(&self, tasks: &[(usize, &Task)]) { ... }

    /// Write info/notice to stderr (suppressed by --quiet)
    pub fn info(&self, msg: &str) {
        if !self.quiet {
            eprintln!("{}", msg);
        }
    }

    /// Write error to stderr (NEVER suppressed by --quiet)
    pub fn error(&self, msg: &str) {
        eprintln!("error: {}", msg);
    }
}
```

### owo-colors NO_COLOR Pattern

```rust
use owo_colors::{OwoColorize, Stream::Stdout, set_override};

// At startup, after parsing --no-color flag:
fn init_color(no_color: bool) {
    // Check both --no-color flag AND NO_COLOR environment variable
    let env_no_color = std::env::var_os("NO_COLOR").is_some();
    if no_color || env_no_color {
        set_override(false);  // requires "supports-colors" feature
    }
}

// Usage in rendering:
let badge = format!("({})", priority);
badge.if_supports_color(Stdout, |t| t.red())
```

**CRITICAL:** `owo-colors` requires the `supports-colors` feature flag to use `set_override()` and
`if_supports_color()`. Without it, only basic `.red()` / `.green()` methods are available (always colored).

### comfy-table Header-Row-Only Pattern

Per CONTEXT.md D-07 — header row, no borders:

```rust
use comfy_table::presets::NOTHING;  // No borders at all
use comfy_table::{Table, Cell, ContentArrangement};

fn build_task_table(tasks: &[(usize, &Task)]) -> Table {
    let mut table = Table::new();
    table.load_preset(NOTHING)
         .set_content_arrangement(ContentArrangement::Dynamic)
         .set_header(vec!["ID", "PRI", "Task"]);

    for (idx, task) in tasks {
        let id = idx + 1;  // 1-based display ID
        let pri = task.priority.map(|p| format!("({})", p)).unwrap_or_default();
        table.add_row(vec![
            id.to_string(),
            pri,
            task.to_raw().to_string(),
        ]);
    }
    table
}
```

**comfy-table preset reference:**
- `comfy_table::presets::NOTHING` — no borders, no separators, just aligned columns
- `comfy_table::presets::ASCII_FULL` — full ASCII borders
- `ContentArrangement::Dynamic` — auto-wrap to terminal width

### JSON Envelope Pattern

Per READ-06 and ROADMAP deliverables:

```rust
use serde::Serialize;
use serde_json::json;

// Success envelope:
// {"schema_version": 1, "data": <T>}
fn json_success<T: Serialize>(data: T) -> String {
    serde_json::to_string(&json!({
        "schema_version": 1,
        "data": data
    })).unwrap()
}

// Error envelope (used when --json AND error):
// {"schema_version": 1, "error": "message"}
fn json_error(msg: &str) -> String {
    serde_json::to_string(&json!({
        "schema_version": 1,
        "error": msg
    })).unwrap()
}
```

**Task JSON serialization** — derive `Serialize` on a DTO struct, not on `Task` itself:

```rust
#[derive(Serialize)]
struct TaskDto {
    id: usize,
    raw: String,
    completed: bool,
    priority: Option<char>,
    projects: Vec<String>,
    contexts: Vec<String>,
    due_date: Option<String>,   // ISO 8601: YYYY-MM-DD
}
```

### Exit Code Pattern (main.rs)

Per READ-08:

```rust
use std::process;

fn main() {
    match run() {
        Ok(()) => process::exit(0),
        Err(e) => {
            // Distinguish "not found" (exit 1) from "error" (exit 2)
            match e.downcast_ref::<CliError>() {
                Some(CliError::NotFound(msg)) => {
                    eprintln!("error: {}", msg);
                    process::exit(1);
                }
                _ => {
                    eprintln!("error: {}", e);
                    process::exit(2);
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum CliError {
    #[error("{0}")]
    NotFound(String),
}
```

**Exit codes:**
- `0` — success (including `list` with zero results — zero results is not "not found")
- `1` — not found / no match (e.g., `show 999` on a 10-task file)
- `2` — error (IO error, missing config `todo_file`, invalid args)

**JSON + error:** When `--json` is active and an error occurs, output the JSON error envelope to
stdout (not stderr) and exit with the appropriate code. This ensures machine-readable error handling.

### assert_cmd Integration Test Pattern

```rust
// tests/list_tests.rs
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn list_empty_file_exits_zero_json() {
    let dir = assert_fs::TempDir::new().unwrap();
    let todo_file = dir.child("todo.txt");
    todo_file.write_str("").unwrap();

    Command::cargo_bin("todotxt")
        .unwrap()
        .arg("--todo-file").arg(todo_file.path())
        .arg("--json")
        .arg("list")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("\"schema_version\":1"))
        .stdout(predicate::str::contains("\"data\":[]"));
}

#[test]
fn show_nonexistent_id_exits_one() {
    let dir = assert_fs::TempDir::new().unwrap();
    let todo_file = dir.child("todo.txt");
    todo_file.write_str("(A) Task one\n").unwrap();

    Command::cargo_bin("todotxt")
        .unwrap()
        .arg("--todo-file").arg(todo_file.path())
        .arg("show").arg("999")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("not found"));
}
```

**Important:** `assert_cmd` needs `dev-dependencies` with `assert_cmd`, `assert_fs`, `predicates`.
Tests go in `crates/todotxt-cli/tests/` (integration tests directory, not unit tests in `src/`).

### Preset + Filter Composition

Per CONTEXT.md D-10–12:

```rust
// In list command: resolve preset and compose with inline filters
fn resolve_filter(
    args: &[String],          // positional args (all filter tokens)
    preset_key: Option<&str>, // `:work` → "work"
    filter_flag: Option<&str>, // --filter "..."
    config: &Config,
) -> Filter {
    let mut tokens = Vec::new();

    // 1. Expand preset (`:name` positional prefix)
    if let Some(key) = preset_key {
        if let Some(preset) = config.presets.get(key) {
            if let Some(filter_str) = &preset.filter {
                tokens.push(filter_str.clone());
            }
        }
    }

    // 2. Inline filter tokens from positional args
    if !args.is_empty() {
        tokens.push(args.join(" "));
    }

    // 3. --filter flag (complex queries)
    if let Some(f) = filter_flag {
        tokens.push(f.to_string());
    }

    Filter::from_query(&tokens.join(" "))
}
```

**Disambiguation rule (D-12):** If a positional arg starts with `:`, it's a preset reference (strip
the `:` to get the key). Otherwise it's a filter token. A plain `work` is ALWAYS a filter token.

---

## Don't Hand-Roll

| Task | Use Instead |
|------|------------|
| Platform config paths | `directories::ProjectDirs::from("", "", "todotxt")` |
| TOML parsing/serialization | `toml::from_str()` / `toml::to_string_pretty()` |
| ANSI color handling | `owo-colors` with `set_override()` for global no-color |
| Shell completion generation | `clap_complete::aot::generate()` |
| CLI test process spawning | `assert_cmd::Command::cargo_bin()` |
| Temp files in tests | `assert_fs::TempDir` (not `tempfile::NamedTempFile` — avoids Windows handle conflicts) |
| JSON serialization | `serde_json::json!()` macro for one-off envelopes |

---

## Common Pitfalls

### P-01: `set_override()` requires `supports-colors` feature
`owo_colors::set_override()` is behind the `supports-colors` feature flag. Without it, `--no-color`
will have no effect on owo-colors output. **Fix:** declare `owo-colors = { version = "4", features = ["supports-colors"] }`.

### P-02: NO_COLOR env var must be checked before first color use
Call `init_color()` in `main()` BEFORE dispatching to any command handler. Once a colored string is
formatted, the override won't retroactively strip ANSI codes from already-rendered output.

### P-03: `directories::ProjectDirs::from()` returns `Option`
It returns `None` if no home directory is found. Always handle this case — fall back to CWD or
propagate an error with code 2.

### P-04: comfy-table `NOTHING` preset removes all separators
With `NOTHING`, there is no header/body separator line. If the design requires a separator under the
header (but no borders), use `comfy_table::presets::UTF8_NO_BORDERS` or set the header separator
manually with `table.set_style(TableComponent::HeaderLines, '─')`.

### P-05: assert_cmd binary must be built before tests run
`Command::cargo_bin("todotxt")` builds the binary lazily. In CI, always run `cargo build -p
todotxt-cli` before `cargo test -p todotxt-cli` to avoid false failures from stale binaries.

### P-06: Exit codes and `anyhow::Result` in main
`fn main() -> anyhow::Result<()>` exits with code `1` on any error. For exit codes 0/1/2 semantics,
use `fn main()` returning `()` and call `std::process::exit()` explicitly. Do NOT use
`anyhow::Result` return type on `fn main` — it prevents granular exit codes.

### P-07: clap global flag propagation
`#[arg(long, global = true)]` flags propagate to subcommands but must be defined on the top-level
`Cli` struct, not inside individual subcommand `Args` structs. If defined inside a subcommand, they
won't be available from the parent level.

### P-08: Config file path creation
When auto-creating config on first run, always call `std::fs::create_dir_all(path.parent())?`
before `std::fs::write()`. The config directory may not exist yet.

### P-09: JSON output goes to stdout, errors to stderr — EXCEPT `--json` errors
When `--json` is active and an error occurs, write the JSON error envelope to **stdout** (not
stderr), then exit with the appropriate code. This ensures scripting consumers can parse errors
as JSON from stdout. Non-JSON mode always writes errors to stderr.

### P-10: `list` filter with no results → exit 0, not exit 1
Exit code 1 means "not found" (e.g., `show <id>` for a nonexistent ID). Filtering with no matches
is a valid result, not "not found" — `list :work` with no matching tasks exits 0 with an empty
table (or empty JSON array).

---

## Architectural Responsibility Map

| Layer | Responsibility |
|-------|---------------|
| `config.rs` | Load/save/auto-create TOML config; path resolution; preset storage |
| `cli.rs` | Clap derive structs; argument types; no logic |
| `output.rs` | All formatting: table rendering, JSON envelope, color, quiet mode |
| `main.rs` | Parse args; dispatch; exit-code mapping; init_color() |
| `commands/*.rs` | Business logic: load TaskList, apply filter/sort, call renderer |
| `todotxt-core` | All task/file operations — CLI commands must NOT do file I/O directly |

**Key constraint:** Commands call `TaskList::load()`, `TaskList::filter()`, `TaskList::sort()` from
`todotxt-core`. They do NOT read files or apply filter logic themselves. All rendering goes through
`Renderer` in `output.rs`.

---

## Validation Architecture

### Dimension 8: Validation Strategy for Phase 3

**Unit test coverage (in `src/`):**
- `config.rs`: Config deserialization from TOML string; preset lookup; path resolution logic
- `output.rs`: JSON envelope shape (schema_version: 1); Renderer::info suppressed by quiet; error output not suppressed by quiet

**Integration test coverage (in `tests/`):**
- Exit code contract: `list` empty file → 0; `show 999` → 1; missing todo_file key in config → 2
- NO_COLOR env var: `list` with `NO_COLOR=1` produces no ANSI sequences
- `--json` output: `schema_version: 1` present; `data` field present; JSON parses without error
- Completions: `completions bash`, `completions zsh`, `completions fish`, `completions powershell` all exit 0 with non-empty stdout
- Preset: `list :work` applies preset filter

**What must NOT regress:**
- `cargo test -p todotxt-core` must still pass after adding `todotxt-cli` crate
- `cargo clippy --workspace -- -D warnings` must pass on all crates

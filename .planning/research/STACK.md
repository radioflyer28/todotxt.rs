# Technology Stack — Rust todotxt-core + CLI

**Project:** todotxt.net → Rust Port (v1.0: Core + CLI)
**Researched:** 2025-08-03
**Confidence:** HIGH (all versions verified against crates.io API)

---

## Recommended Stack

### Core Library (`todotxt-core`)

| Crate | Version | Purpose | Why |
|-------|---------|---------|-----|
| `winnow` | 1.0.1 | todo.txt line parser | Zero-copy, composable, replaces nom. todo.txt's grammar (optional priority, dates, key:value tags scattered in free text) benefits from parser combinators over regex — edge cases like `(A)` in task body are handled correctly. |
| `serde` | 1.0.228 | Serialize/deserialize Task model | Industry standard; derive macros on `Task` struct enable both JSON output and config round-tripping with zero boilerplate. |
| `serde_json` | 1.0.149 | JSON serialization | CLI's structured output mode requires it; keeping it in core means CLI gets JSON for free. |
| `chrono` | 0.4.44 | Date parsing and arithmetic | `NaiveDate` is the exact right type for todo.txt dates (YYYY-MM-DD, no timezone). Better ecosystem integration than `time` crate; `chrono::format::strftime` handles parsing. |
| `thiserror` | 2.0.18 | Library error types | Library crates must expose typed errors — `anyhow` is wrong here. `thiserror` 2.x derive macros generate clean `ParseError`, `IoError`, etc. with zero runtime overhead. |
| `notify` | 8.2.0 | File system events | Stable release (not 9.0.0-rc.2). Cross-platform FSEvents/inotify/ReadDirectoryChangesW abstraction. |
| `notify-debouncer-mini` | 0.7.0 | Debounced file watching | Raw `notify` events fire multiple times per save. `debouncer-mini` coalesces into single events after a quiet period — matches what the C# `FileChangeObserver` does (1-second delay before reload). |
| `regex` | 1.12.3 | Supplemental pattern matching | Used for filter expressions (project/context/text search). After `winnow` parses the Task, `regex` handles runtime user-supplied search patterns. |

### CLI Binary (`todotxt`)

| Crate | Version | Purpose | Why |
|-------|---------|---------|-----|
| `clap` | 4.6.0 | Argument parsing | The standard. Derive macros (`#[derive(Parser)]`) produce type-safe arg structs, auto-generated `--help`, subcommands, and shell completions. No alternative worth considering. |
| `anyhow` | 1.0.102 | CLI error propagation | Binaries don't need typed errors — `anyhow` gives `?`-everywhere ergonomics and clean error output. Pair with `thiserror` in the library layer. |
| `directories` | 6.0.0 | Cross-platform config paths | Returns `AppDirs` with `config_dir()`, `data_dir()`, `cache_dir()` per platform (XDG on Linux, `~/Library/Application Support` on macOS, `%APPDATA%` on Windows). Use this directly over `dirs` — it provides app-namespaced paths. |
| `toml` | 1.1.2 | Config file format | `TOML + serde` is the Rust-native config idiom. Human-editable, self-documenting, no YAML footguns. Parse directly into a typed `Config` struct. |
| `owo-colors` | 4.3.0 | Terminal color output | Compile-time color support, no global state (contrast: `colored` uses a global mutex). Supports `NO_COLOR` env var. Use for human-readable output; JSON mode gets no color. |
| `comfy-table` | 7.2.2 | Tabular output formatting | For `list` command human output. Handles Unicode width, dynamic column sizing, cross-platform rendering. `tabled` is an alternative but `comfy-table` has simpler API for this use case. |
| `tracing` | 0.1.44 | Structured logging | Prefer over `log` + `env_logger`. `tracing` spans work well for tracking file-watch events and parse errors. Add `tracing-subscriber` for the CLI subscriber. |
| `shellexpand` | 3.1.2 | `~` path expansion in config | Config files will contain `~/todo.txt` style paths. `shellexpand::tilde()` handles this cross-platform before passing to `std::path`. |

### Shared Dev / Test

| Crate | Version | Purpose | Notes |
|-------|---------|---------|-------|
| `tempfile` | 3.27.0 | Temp files in tests | `NamedTempFile` + `TempDir` for file I/O tests in `todotxt-core`. |
| `assert_cmd` | 2.2.0 | CLI integration tests | Runs the binary as a subprocess, asserts on stdout/stderr/exit code. The right way to test a CLI. |
| `predicates` | 3.1.4 | Assertions for `assert_cmd` | Fluent string/regex matchers; used with `assert_cmd`. |
| `insta` | 1.47.2 | Snapshot testing | Run `cargo insta review` to accept/update snapshots. Ideal for parser round-trip tests and CLI output formatting — catches regressions in `Task::to_string()` and JSON shape. |
| `rstest` | 0.26.1 | Parameterized tests | `#[rstest]` table-driven tests for the parser. Each todo.txt format variant (priority only, date only, both dates, key:value pairs, completed, etc.) gets a row. |

---

## Cargo Workspace Structure

```
todotxt.net/              ← git root (C# app stays here, untouched)
├── ToDo.Net.sln
├── Client/
├── ToDoLib/
├── ...
└── rust/                 ← NEW: Rust workspace root
    ├── Cargo.toml        ← [workspace] members = ["todotxt-core", "todotxt"]
    ├── Cargo.lock        ← committed (binary crate in workspace)
    ├── .cargo/
    │   └── config.toml   ← workspace-level Cargo config (lint levels, etc.)
    ├── todotxt-core/
    │   ├── Cargo.toml    ← lib crate; no [[bin]]
    │   ├── src/
    │   │   ├── lib.rs
    │   │   ├── task.rs       ← Task struct + serde derives
    │   │   ├── parser.rs     ← winnow parser
    │   │   ├── task_list.rs  ← TaskList + file I/O
    │   │   ├── filter.rs     ← filter/sort logic
    │   │   ├── watcher.rs    ← notify + debouncer wrapper
    │   │   └── error.rs      ← thiserror error types
    │   └── tests/
    │       ├── parser_tests.rs
    │       └── task_list_tests.rs
    └── todotxt/
        ├── Cargo.toml    ← bin crate; depends on todotxt-core
        ├── src/
        │   ├── main.rs
        │   ├── cli.rs        ← clap derive structs
        │   ├── commands/
        │   │   ├── add.rs
        │   │   ├── list.rs
        │   │   ├── complete.rs
        │   │   ├── delete.rs
        │   │   ├── edit.rs
        │   │   └── archive.rs
        │   ├── output.rs     ← human vs JSON output formatting
        │   └── config.rs     ← toml config + directories paths
        └── tests/
            └── cli_tests.rs  ← assert_cmd integration tests
```

**Workspace `Cargo.toml` skeleton:**

```toml
[workspace]
members = ["todotxt-core", "todotxt"]
resolver = "2"

[workspace.dependencies]
# Pin versions once; all crates inherit with `workspace = true`
serde       = { version = "1.0", features = ["derive"] }
serde_json  = "1.0"
thiserror   = "2.0"
anyhow      = "1.0"
chrono      = { version = "0.4", features = ["serde"] }
winnow      = "1.0"
regex       = "1.12"
notify      = "8.2"
notify-debouncer-mini = "0.7"
clap        = { version = "4.6", features = ["derive"] }
toml        = "1.1"
directories = "6.0"
owo-colors  = "4.3"
comfy-table = "7.2"
tracing     = "0.1"
shellexpand = "3.1"
tempfile    = "3.27"
assert_cmd  = "2.2"
predicates  = "3.1"
insta       = "1.47"
rstest      = "0.26"

[workspace.lints.rust]
unsafe_code = "forbid"
```

---

## Alternatives Considered and Rejected

| Category | Recommended | Rejected | Why Rejected |
|----------|-------------|----------|--------------|
| Parsing | `winnow` 1.0.1 | `nom` 8.0 | `winnow` IS nom's successor; nom 8 introduced its own breaking changes. Same author, winnow has cleaner error types and `&str`-first API. |
| Parsing | `winnow` 1.0.1 | `pest` | PEG grammar files add a build step and complexity for a line-based format. `winnow` composables are simpler to maintain. |
| Parsing | `winnow` 1.0.1 | pure `regex` | Regex works for the happy path but struggles with todo.txt edge cases: `(A)` appearing inside task body, key:value pairs with colons in URLs, etc. |
| Date/time | `chrono` 0.4 | `time` 0.3 | Both are maintained. `chrono`'s `NaiveDate` is the canonical "date without timezone" type; `chrono` has far more serde/format integrations. `time` is better for timestamp math — not needed here. |
| Config | `toml` + `serde` | `config` (config-rs) | `config-rs` supports layered sources (env, file, defaults) — useful for complex apps. Overkill here; adds ~10 transitive dependencies for features we don't need. |
| Config | `toml` + `serde` | `figment` | Same story as config-rs. `figment` is excellent for web frameworks with many config sources. A CLI tool with one config file doesn't need it. |
| Config | `toml` + `serde` | `confy` | `confy` wraps `directories` + `serde` but forces opinionated naming and doesn't support portable mode (config beside binary). We need portable mode parity with the C# app. |
| File watching | `notify` 8.2 | `notify` 9.0.0-rc.2 | RC releases are not production-ready. `notify` 8.2 is the current stable, released 2025-08-03. |
| Colors | `owo-colors` 4.3 | `colored` 3.1 | `colored` uses a global `AtomicBool` for NO_COLOR detection and a mutex around color state. `owo-colors` is zero-global-state; colors are opt-in per value. Better for a library that might be embedded. |
| Error handling (lib) | `thiserror` 2.0 | `anyhow` | `anyhow` in a library hides error type information from downstream users. `thiserror` produces `std::error::Error` impl with a concrete type — callers can match on it. |
| Error handling (CLI) | `anyhow` 1.0 | `thiserror` | CLI binaries don't expose errors to callers — ergonomic `?` propagation and human-readable error chains are what matter. `anyhow` is correct here. |
| Arg parsing | `clap` 4.6 | `argh`, `pico-args`, `lexopt` | `clap` derive is the industry standard with shell completion, rich help generation, and subcommand support. The alternatives are faster to compile but lack subcommand ergonomics needed for a multi-command CLI. |
| Tables | `comfy-table` 7.2 | `tabled` 0.20 | `tabled` is powerful (derives, styles) but API is more complex. `comfy-table` is simpler for dynamic runtime table construction. |

---

## Cross-Platform Specifics

### Config File Paths (via `directories` crate)

```
Windows:  %APPDATA%\todotxt\config.toml   (e.g. C:\Users\user\AppData\Roaming\todotxt\config.toml)
macOS:    ~/Library/Application Support/todotxt/config.toml
Linux:    ~/.config/todotxt/config.toml   (XDG_CONFIG_HOME)
```

**Portable mode** (C# parity): If a `config.toml` exists beside the binary, use it instead of the platform path. Check `std::env::current_exe()` parent directory first, fall back to `directories::ProjectDirs`.

### File Watching

`notify` 8.x uses:
- **Windows**: `ReadDirectoryChangesW`
- **macOS**: FSEvents
- **Linux**: inotify

All three are wrapped uniformly. The C# `FileChangeObserver` had a 1-second sleep to let the writer release the file lock — replicate this with `notify-debouncer-mini`'s `Duration` argument.

### Path Handling

- Always use `std::path::Path` / `PathBuf` — never raw strings for file paths
- `shellexpand::tilde()` before converting user-supplied path strings to `PathBuf`
- On Windows, `notify` watches require absolute paths — call `.canonicalize()` on the watch path

### Line Endings

- `todo.txt` files may have `\r\n` on Windows, `\n` on Linux/macOS
- Parse with `.trim_end_matches(|c| c == '\r' || c == '\n')` before passing to winnow
- Write with `\n` (Unix line endings) — standard for todo.txt format

---

## Installation Snippet

```toml
# rust/todotxt-core/Cargo.toml
[package]
name = "todotxt-core"
version = "0.1.0"
edition = "2021"

[dependencies]
winnow      = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
thiserror   = { workspace = true }
chrono      = { workspace = true }
regex       = { workspace = true }
notify      = { workspace = true }
notify-debouncer-mini = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
insta    = { workspace = true }
rstest   = { workspace = true }
```

```toml
# rust/todotxt/Cargo.toml
[package]
name = "todotxt"
version = "0.1.0"
edition = "2021"

[dependencies]
todotxt-core = { path = "../todotxt-core" }
clap         = { workspace = true }
anyhow       = { workspace = true }
serde_json   = { workspace = true }
toml         = { workspace = true }
directories  = { workspace = true }
owo-colors   = { workspace = true }
comfy-table  = { workspace = true }
tracing      = { workspace = true }
shellexpand  = { workspace = true }

[dev-dependencies]
assert_cmd  = { workspace = true }
predicates  = { workspace = true }
tempfile    = { workspace = true }
```

---

## Confidence Assessment

| Area | Confidence | Source |
|------|------------|--------|
| All crate versions | HIGH | Verified via crates.io API, 2025-08-03 |
| `winnow` for parsing | HIGH | Current successor to `nom`; 1.0.0 stable released |
| `notify` 8.2 stability | HIGH | Verified stable branch; 9.x is still RC |
| `thiserror` 2.x | HIGH | 2.0 released, actively maintained |
| `directories` 6.0 vs `dirs` | HIGH | `directories` provides app-namespaced paths; `dirs` only provides raw OS dirs |
| `owo-colors` over `colored` | MEDIUM | Technical rationale verified; community preference still `colored` by download count but `owo-colors` is architecturally cleaner |
| `comfy-table` over `tabled` | MEDIUM | Both maintained; `comfy-table` API simpler for this use case based on docs review |

---

## Sources

- crates.io API (live query, 2025-08-03): https://crates.io/api/v1/crates/{name}
- `winnow` docs: https://docs.rs/winnow/1.0.1/winnow/
- `notify` changelog: https://crates.io/crates/notify
- `thiserror` 2.0 migration: https://crates.io/crates/thiserror
- `directories` crate: https://crates.io/crates/directories
- `clap` derive guide: https://docs.rs/clap/4.6.0/clap/_derive/index.html

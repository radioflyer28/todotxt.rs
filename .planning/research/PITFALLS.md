# Domain Pitfalls: Rust CLI + Library for todo.txt

**Domain:** Rust CLI tool + core library, ported from C# .NET WPF app
**Researched:** 2025-01-24
**Confidence:** HIGH (codebase directly inspected; Rust pitfalls from production experience + official docs)

---

## Critical Pitfalls

Mistakes that cause data loss, rewrites, or broken downstream consumers.

---

### Pitfall C-1: Porting the Raw-String Identity Bug from C#

**What goes wrong:** The C# `TaskList` uses `t.Raw == task.Raw` for Delete and Update lookups (lines 175, 216, 221 of `TaskList.cs`). If you port this directly to Rust, two tasks with identical text are indistinguishable — deleting one silently deletes the first match. The Rust version will perpetuate a known data-corruption bug.

**Why it happens:** Developers port existing logic faithfully because it "works" in the reference implementation. The C# codebase has no integration tests for duplicate tasks, so the bug is invisible during porting.

**Consequences:** Silent data loss when two tasks have the same text. No error is thrown; the wrong task is deleted. Extremely hard to debug.

**Prevention:**
- Assign a stable `id: usize` (index into the loaded list, not a UUID — UUID is overkill for a local file) at load time
- Use id-based lookup for all mutations: `tasks.get_mut(id)` instead of linear scan
- The `raw` string field is for serialization fidelity only, never for identity
- Write a test: add two identical tasks, delete the second one, verify the first remains

**Detection:** Write this test first, before any other test. If it fails, you've ported the bug.

**Phase:** Core library (Phase 1). Fix before CLI exists.

---

### Pitfall C-2: Non-Atomic File Writes Corrupt the todo.txt File

**What goes wrong:** The C# `WriteAllTasksToFile()` opens `StreamWriter` directly over the target file and writes line-by-line. If the process crashes mid-write (power loss, SIGKILL, panic), the file is truncated and tasks are lost. There is no backup. The C# codebase acknowledges this (`Missing Critical Features: No Backup Before Write`). If the Rust port does the same, it inherits the bug.

**Why it happens:** `File::create(path)` truncates immediately. Writing to a temp file and renaming is not the "obvious" approach.

**Consequences:** Any crash or panic during a write corrupts the user's todo.txt permanently. A CLI tool called from a shell script or AI agent is more likely to be killed mid-operation than an interactive GUI.

**Prevention:**
```rust
// WRONG: truncates immediately
let mut f = File::create(&self.path)?;

// RIGHT: atomic write via temp file + rename
use std::io::Write;
let tmp_path = self.path.with_extension("tmp");
{
    let mut f = File::create(&tmp_path)?;
    for task in &self.tasks {
        writeln!(f, "{}", task)?;
    }
    f.flush()?;
    f.sync_all()?;  // flush OS buffers
}
std::fs::rename(&tmp_path, &self.path)?;  // atomic on POSIX; best-effort on Windows
```

On Windows, `rename` is not atomic if the destination exists. Use the `tempfile` crate's `NamedTempFile::persist()` which handles cross-platform atomic-ish replacement. The `fs_err` crate adds context to I/O errors automatically.

**Detection:** Run the test: create a file, kill the process mid-write with SIGKILL, verify the original file is intact.

**Phase:** Core library (Phase 1). Never ship without atomic writes.

---

### Pitfall C-3: UTF-8 BOM in Existing Files Breaks First-Line Parsing

**What goes wrong:** The C# source files themselves have a UTF-8 BOM (`0xEF 0xBB 0xBF`) — visible as `﻿` at the top of `Task.cs` and `TaskList.cs`. Users who created their todo.txt on Windows with Notepad may also have a BOM. Rust's `std::io::BufReader::lines()` does NOT strip the BOM. The first line read will start with `\u{FEFF}` which breaks parsing: `"\u{FEFF}(A) Task"` does not match the priority regex and does not match the completed pattern.

**Why it happens:** Rust's standard library is BOM-agnostic. Developers test with clean files and never encounter the issue until a real user's file breaks.

**Consequences:** First task in any BOM-prefixed file silently loses its priority, due date, or is incorrectly flagged as unparseable. The file is not corrupted but data appears wrong in output.

**Prevention:**
```rust
// Strip BOM from the first line only
fn strip_bom(s: &str) -> &str {
    s.strip_prefix('\u{FEFF}').unwrap_or(s)
}
```

Call `strip_bom` on the first line read from any file. When writing back, do not write a BOM (todo.txt is UTF-8 without BOM by convention).

**Detection:** Add a test: parse a file whose first byte sequence is `EF BB BF`. Verify the first task parses correctly.

**Phase:** Core library (Phase 1), file I/O module.

---

### Pitfall C-4: Line Ending Corruption Breaks Interop with Other Tools

**What goes wrong:** The C# app actively detects whether the file uses `\r\n` or `\n` and preserves it (`GetPreferredFileLineEndingFromFile`). If the Rust port uses `writeln!` (which emits `\n` on all platforms), it will rewrite a Windows CRLF file as LF, breaking tools that opened the same file expecting CRLF. Worse: the C# GUI running simultaneously with the Rust CLI will silently disagree on line endings.

**Why it happens:** `writeln!` is the natural Rust idiom. It writes `\n` everywhere. Developers don't test on Windows files.

**Consequences:** Interop break with C# GUI, todo.sh, or any Windows editor that opened the same file.

**Prevention:**
- Detect line endings on load (same algorithm as C#: scan the first 4000 bytes)
- Preserve detected ending on write
- The `line-ending` or `newline` crates can help, but the C# algorithm is simple enough to port directly

**Warning signs:** Tests pass on Linux but fail on Windows; or files opened in Windows Notepad show everything on one line.

**Phase:** Core library (Phase 1), file I/O module.

---

### Pitfall C-5: Unstable JSON Schema Breaks AI Agent Consumers

**What goes wrong:** You ship a JSON output format, an AI agent skill builds against it, then a patch version changes a field name from `due_date` to `dueDate` or adds a non-optional field. The agent skill silently gets `null` values or crashes deserialization. This is the highest-consequence breaking change for the stated use case.

**Why it happens:** JSON schema stability is not enforced by the Rust compiler. Serde field renames are one attribute change. Nobody notices until an agent workflow breaks silently.

**Consequences:** Every downstream consumer (AI agents, shell scripts, other tools) that depends on the JSON output breaks on upgrade, often silently.

**Prevention:**
- Define a versioned schema from day 1: include a `"schema_version": 1` field in every JSON response
- Use `serde(rename_all = "camelCase")` or `serde(rename_all = "snake_case")` — pick one and commit
- Never add non-optional fields to existing objects without bumping the schema version
- Treat CLI JSON output as a public API with semver semantics
- Add a snapshot test that serializes a task list to JSON and diffs against a committed fixture file — CI fails if the schema changes unexpectedly

**Schema contract (commit this day 1):**
```json
{
  "schema_version": 1,
  "tasks": [
    {
      "id": 0,
      "raw": "(A) 2024-01-15 Call dentist +Health @phone due:2024-01-31",
      "completed": false,
      "priority": "A",
      "creation_date": "2024-01-15",
      "completion_date": null,
      "body": "Call dentist",
      "projects": ["Health"],
      "contexts": ["phone"],
      "due_date": "2024-01-31",
      "threshold_date": null
    }
  ]
}
```

**Detection:** Add a golden-file snapshot test. Commit the JSON fixture. Any schema change fails CI.

**Phase:** CLI JSON output (Phase 2). Design schema before first commit of `--json` flag.

---

### Pitfall C-6: Regex Recompilation in the Parse Hot Path

**What goes wrong:** Rust's `Regex::new()` is expensive — it compiles a DFA. The C# code creates `new Regex(pattern)` inside the constructor for every task, which is also inefficient, but C# caches compiled regexes internally. Rust does not. If you port the C# constructor pattern naively, each task parse compiles 6+ regexes.

**Why it happens:** Direct port of the C# constructor. Feels natural because `let reg = Regex::new(PATTERN)?;` looks just like C#.

**Consequences:** Parsing a 1000-task file is measurably slower than it should be. Profiling will show regex compilation, not regex matching, as the bottleneck.

**Prevention:** Use `std::sync::OnceLock` (stable since Rust 1.70) or the `once_cell` / `lazy_static` crate to compile regexes exactly once:
```rust
use std::sync::OnceLock;
use regex::Regex;

static PRIORITY_RE: OnceLock<Regex> = OnceLock::new();

fn priority_regex() -> &'static Regex {
    PRIORITY_RE.get_or_init(|| Regex::new(r"^\(([A-Z])\) ").unwrap())
}
```

Alternative: use the `regex` crate's `LazyLock` (stable Rust 1.80+) which is even simpler.

**Warning signs:** `cargo flamegraph` shows `regex::Regex::new` in the hot path.

**Phase:** Core library (Phase 1), Task parser.

---

## Moderate Pitfalls

Mistakes that cause significant rework but not data loss.

---

### Pitfall M-1: Leaking Implementation Types in the Public Crate API

**What goes wrong:** You expose concrete types in the `todotxt` crate's public API that you later need to change. For example: `pub fn tasks(&self) -> &Vec<Task>` instead of `&[Task]`. Or: `pub enum SortOrder { Priority, DueDate, Alpha }` without `#[non_exhaustive]`. Adding a variant to a non-exhaustive enum is a breaking change — downstream code that pattern-matches it won't compile.

**Why it happens:** "I'll just make it public for now" during rapid development. The TUI and GUI crates that will consume the library are not written yet, so there's no pressure to define a stable API.

**Consequences:** When the TUI crate (v1.1) is built, it match-exhausts enums that the core library needs to extend. This forces a semver major bump or API contortion.

**Prevention:**
- Mark all public enums that may gain variants with `#[non_exhaustive]`
- Return `&[Task]` not `&Vec<Task>` from collection getters
- Use `impl Iterator<Item = &Task>` for iteration
- Review every `pub` with the question: "Will this type/signature need to change when the TUI is built?"
- Before publishing to crates.io, run `cargo-semver-checks` (available as a cargo plugin)

**Phase:** Core library (Phase 1). API review before first tag.

---

### Pitfall M-2: Custom Error Types That Don't Compose

**What goes wrong:** You define `TodoError` with `#[derive(Debug)]` but forget `impl std::error::Error for TodoError` or `impl Display`. The `?` operator works internally but library consumers can't use `anyhow::Context` or `Box<dyn std::error::Error>` with your errors.

**Why it happens:** Rust's error handling is more explicit than C#'s exception hierarchy. It's easy to miss `Display` or `source()`.

**Consequences:** Library consumers hit "the trait `std::error::Error` is not implemented for `TodoError`" when trying to use `?` with `anyhow` or `eyre`.

**Prevention:** Use `thiserror` for all library errors — it derives `Display`, `Error`, and `source()` from a single macro:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TodoError {
    #[error("IO error reading {path}: {source}")]
    Io { path: PathBuf, #[source] source: std::io::Error },
    #[error("parse error on line {line}: {message}")]
    Parse { line: usize, message: String },
}
```

Use `anyhow` in the CLI binary only (not the library). Never use `anyhow` in the library crate — it prevents callers from matching error variants.

**Phase:** Core library (Phase 1), error module.

---

### Pitfall M-3: Config Paths That Break on Linux/macOS (XDG Non-Compliance)

**What goes wrong:** The C# app stores settings in the executable directory (portable mode) or relies on .NET's `Application.UserAppDataPath`. Porting this naively uses `std::env::current_exe()` for non-portable mode or hard-codes `~/.config/todo`. On Linux, XDG Base Directory Specification requires `$XDG_CONFIG_HOME` (defaults to `~/.config`). On macOS, the convention is `~/Library/Application Support`. On Windows it's `%APPDATA%`.

**Why it happens:** `~/.config/appname` feels universal but is wrong on macOS and Windows.

**Consequences:** Config stored in wrong location; `ls ~/.config` on macOS clutters the home directory; settings not found on macOS.

**Prevention:** Use the `dirs` crate (or `directories` crate) which returns the correct platform path:
```rust
use directories::ProjectDirs;

let project_dirs = ProjectDirs::from("com", "todotxt", "todotxt")
    .ok_or(TodoError::NoConfigDir)?;
let config_path = project_dirs.config_dir().join("config.toml");
```

For portable mode: check if a `todo.toml` exists beside the binary first; if so, use it. Fall back to platform config dir.

**Phase:** CLI settings/config (Phase 2).

---

### Pitfall M-4: Panic in the CLI Instead of Graceful Errors

**What goes wrong:** `unwrap()` and `expect()` are the easiest way to write Rust, and they're fine in tests. In a CLI binary called by an AI agent, a panic produces a non-zero exit code plus a Rust backtrace on stderr with no useful JSON. The agent sees `thread 'main' panicked at 'called Result::unwrap() on Err value: Os { code: 2, kind: NotFound, message: "No such file or directory" }'` on stderr and has to parse prose to understand what went wrong.

**Why it happens:** Fast iteration uses `unwrap()`. Cleanup is deferred and never happens.

**Consequences:** AI agent skills can't distinguish "task not found" from "file not found" from "malformed input" — all are unstructured panics.

**Prevention:**
- Use `anyhow` in `main.rs`: `fn main() -> anyhow::Result<()>`
- Use the `color-eyre` or `miette` crate for human-readable error formatting
- When `--json` is active, catch all top-level errors and emit them as JSON before exiting:
```json
{"error": {"code": "FILE_NOT_FOUND", "message": "todo.txt not found at /path/to/todo.txt"}}
```
- Exit code conventions: 0 = success, 1 = user error, 2 = system/IO error

**Warning signs:** `grep -r 'unwrap()' src/` finds hits in non-test code.

**Phase:** CLI (Phase 2). Apply discipline from the first command.

---

### Pitfall M-5: File Watcher Double-Subscription and Debouncing

**What goes wrong:** The C# `FileChangeObserver` uses `Thread.Sleep(1000)` to debounce file changes. Porting this to Rust with the `notify` crate without proper debouncing fires the reload callback multiple times per save (editors write files in multiple operations). A CLI tool that reacts to file changes (watch mode) will reload 3-5 times per user save.

**Why it happens:** `notify` fires events for each file system operation (create temp, write, rename), not per logical save.

**Consequences:** In watch mode, the CLI spams stdout with redundant output. AI agents polling the watch mode output get duplicate events.

**Prevention:** Use `notify-debouncer-mini` (part of the `notify` crate workspace) which provides built-in debouncing:
```rust
use notify_debouncer_mini::{new_debouncer, notify::*, DebounceEventResult};
let mut debouncer = new_debouncer(Duration::from_millis(500), move |res: DebounceEventResult| {
    // fires once per logical save
})?;
```

**Phase:** Core library file watching (Phase 1 or Phase 2 depending on sequencing).

---

### Pitfall M-6: Priority Regex Case-Sensitivity — Fix vs. Preserve Compatibility

**What goes wrong:** The C# code uses `RegexOptions.IgnoreCase` on the priority pattern (`(a)` is accepted as a valid priority, which violates the todo.txt spec). The Rust port must decide: fix the bug or preserve backward compatibility with malformed files. If fixed silently, users with `(a)` tasks see their priority disappear after migration.

**Why it happens:** The C# has a `//TODO priority regex need to only recognice upper case single chars` comment that was never addressed.

**Consequences:** If you fix it correctly (uppercase only) without a migration path, users lose data.

**Prevention:**
- Fix it correctly: `[A-Z]` only, case-sensitive
- On load: if a line starts with `(a-z)` (lowercase), emit a warning (not an error) and preserve the Raw string
- Provide a `todo migrate` subcommand that uppercases lowercase priorities with confirmation
- Document the spec-compliance fix in the changelog

**Phase:** Core library (Phase 1). Decision must be made before first beta.

---

### Pitfall M-7: Mixing stdout and stderr Breaks JSON Piping

**What goes wrong:** You write the JSON task list to stdout but log "Loaded 42 tasks from todo.txt" to stdout as well. A downstream `todo list --json | jq '.tasks'` fails with a JSON parse error because stdout contains non-JSON lines.

**Why it happens:** `println!` is the default for everything. The distinction only matters when piping.

**Consequences:** Every pipe operation from an AI agent fails unless the agent strips log lines manually.

**Prevention:**
- Human-readable output (progress, warnings, info) → `stderr` (`eprintln!`)
- Machine-readable output (task list, operation result) → `stdout` (`println!`)
- When `--json` is active: structured JSON only on stdout, structured error JSON on stdout with exit code 1, diagnostics on stderr
- This is the `clap` + `serde_json` idiom used by `jq`, `gh`, `az`, and every production CLI

**Phase:** CLI (Phase 2). Establish the convention before writing the first command.

---

## Minor Pitfalls

Mistakes that cause friction but are fixable without rework.

---

### Pitfall m-1: Windows Terminal Color Support Detection

**What goes wrong:** Using ANSI escape codes directly on Windows 8.1 / Server 2016 or older terminals produces literal `\x1b[32m` characters instead of colors. The C# app avoids this by being a WPF GUI, but the Rust CLI targets all platforms.

**Prevention:** Use the `supports-color` crate to detect terminal capabilities, or use `colored` / `owo-colors` which auto-detect. Never hard-code ANSI codes. When `--json` is active, strip all color codes.

**Phase:** CLI output formatting.

---

### Pitfall m-2: `dirs::config_dir()` Returns `None` in Some CI Environments

**What goes wrong:** `ProjectDirs::from(...)` returns `None` when `$HOME` is unset (some Docker containers, minimal CI environments). An `unwrap()` panics. A `?` propagates an opaque error.

**Prevention:** Handle the `None` case explicitly with a useful error: `"Cannot determine config directory. Set $HOME or use --config to specify a path."`

**Phase:** CLI settings.

---

### Pitfall m-3: `cargo test` Does Not Test Cross-Platform Paths

**What goes wrong:** Tests that construct paths with string literals like `"/tmp/todo.txt"` pass on Linux/macOS and fail on Windows. Or tests that hard-code `\` as separator pass on Windows only.

**Prevention:** Always use `PathBuf` and `Path::join()`. In tests, use `tempfile::tempdir()` for temporary files. Never use `/tmp` in test paths. Use `std::env::temp_dir()` if absolutely necessary.

**Phase:** Core library (Phase 1). Enforce from the first test.

---

### Pitfall m-4: `serde` Derives on Public Types Are Semver-Breaking

**What goes wrong:** You add `#[derive(Serialize, Deserialize)]` to `Task` in the library crate. Later you add a field. Downstream crates that deserialize `Task` from JSON now get an error for old JSON that lacks the new field (unless you add `#[serde(default)]`). Removing `#[derive(Deserialize)]` is also a breaking change.

**Prevention:**
- Add `#[serde(default)]` to every optional field
- Consider whether `Task` in the library should derive `Serialize`/`Deserialize` at all, or whether serialization should live in the CLI crate with a separate DTO struct

**Phase:** Core library API design (Phase 1).

---

### Pitfall m-5: Porting the Sequential Regex Mutation Pattern Directly

**What goes wrong:** The C# parser strips tokens from `raw` by replacing regex matches on a mutating string variable. The order is load-bearing: completed → priority → due date → threshold date → created date → projects → contexts → body. If you port this pattern directly, you get the same fragility: a date inside a URL over-matches `CreatedDatePattern` and corrupts the body.

**Prevention:** Do not port this pattern. Instead, use a single-pass parser with explicit field extraction:
1. Check for completion prefix first (anchored `^x `)
2. Check for priority second (anchored `^\([A-Z]\) `)
3. Extract `due:YYYY-MM-DD`, `t:YYYY-MM-DD` as key-value tokens anywhere in the string (not by stripping)
4. Find `+word` and `@word` tokens
5. Reconstruct body by removing extracted tokens from a copy

This prevents the over-match corruption and is easier to test. The `Raw` field always stores the original unmodified string for round-trip fidelity.

**Phase:** Core library (Phase 1), Task parser.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|---|---|---|
| Core library: Task parser | Sequential regex mutation (C-2 clone, m-5) | Single-pass extractor, not mutating strip |
| Core library: Task parser | BOM on first line (C-3) | strip_bom() on first line at load |
| Core library: Task parser | Regex recompilation (C-6) | `OnceLock<Regex>` or `LazyLock` |
| Core library: Task parser | Priority case bug port (M-6) | Fix to uppercase-only + migration path |
| Core library: Task identity | Raw-string identity bug (C-1) | Index-based identity, fix before CLI |
| Core library: File I/O | Non-atomic writes (C-2) | tempfile + rename |
| Core library: File I/O | Line ending corruption (C-4) | Detect and preserve on write |
| Core library: File watching | Debounce missing (M-5) | notify-debouncer-mini |
| CLI: Error handling | Panics leak to agents (M-4) | anyhow in main, JSON errors |
| CLI: Output | stdout/stderr mixing (M-7) | Strict: JSON on stdout, logs on stderr |
| CLI: JSON schema | Schema instability (C-5) | Version field + snapshot tests |
| CLI: Config paths | Wrong platform paths (M-3) | `directories` crate |
| Library crate: API | Non-exhaustive enums (M-1) | #[non_exhaustive] from day 1 |
| Library crate: Errors | Non-composable errors (M-2) | thiserror in lib, anyhow in bin |
| All: Tests | Platform-specific path literals (m-3) | PathBuf + tempfile crate |

---

## Sources

- Direct inspection of `ToDoLib/Task.cs`, `ToDoLib/TaskList.cs`, `Client/MainWindowViewModel.cs`, `.planning/codebase/CONCERNS.md`
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/
- Rust `notify` crate: https://docs.rs/notify/latest/notify/
- Rust `thiserror` crate: https://docs.rs/thiserror/latest/thiserror/
- Rust `directories` crate: https://docs.rs/directories/latest/directories/
- Rust `tempfile` crate: https://docs.rs/tempfile/latest/tempfile/
- todo.txt format spec: https://github.com/todotxt/todo.txt
- Semver compatibility reference: https://doc.rust-lang.org/cargo/reference/semver.html

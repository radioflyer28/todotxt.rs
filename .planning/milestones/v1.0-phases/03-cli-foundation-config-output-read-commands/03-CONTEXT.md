# Phase 3: CLI Foundation — Config + Output + Read Commands - Context

**Gathered:** 2026-04-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Establish all cross-cutting CLI conventions (output discipline, exit codes, JSON envelope, color,
config loading) and implement every read command — delivering a fully usable read-only CLI.
No write operations in this phase. All work is inside `crates/todotxt-cli/`.

</domain>

<decisions>
## Implementation Decisions

### Config Bootstrapping

- **D-01:** On first run with no config file, auto-create a default `config.toml` at the
  platform-appropriate path (via `directories` crate). Print an informational message to stderr
  noting where the file was created (suppressed by `--quiet`).
- **D-02:** The auto-created config includes `todo_file = "~/todo.txt"` so the CLI is usable
  out of the box without any manual setup.
- **D-03:** If the config file exists but the `todo_file` key is absent (user manually deleted it),
  the CLI exits with code 2 and prints an error to stderr. There is no implicit default once the
  user has taken ownership of the config.
- **D-04:** The `--todo-file` CLI flag always overrides the config value. Platform config path
  follows `directories` crate conventions: `~/.config/todotxt/config.toml` (Linux),
  `%APPDATA%\todotxt\config.toml` (Windows), `~/Library/Application Support/todotxt/config.toml`
  (macOS). Portable mode (binary-adjacent `config.toml`) takes precedence per CORE-08 / `resolve_config_path()`.

### List Table Layout

- **D-05:** `list` displays three columns: **ID** | **Priority** | **Task text** (no done/date columns).
- **D-06:** Priority column shows the badge `(A)`, `(B)`, etc. — no text colorization on the task
  text column. Tasks with no priority show an empty priority cell.
- **D-07:** Table style: **header row only** — a header line above data rows, no borders,
  no ASCII/Unicode box-drawing. Data rows are plain aligned text. Implemented via `comfy-table`
  with no-borders preset.

### Filter + Preset UX

- **D-08:** `list` positional arguments are always treated as AND filter tokens, passed directly
  to `Filter::from_query()` (space-joined). Example: `todotxt list +work @home` filters by both.
- **D-09:** `--filter "<query>"` flag accepts a quoted multi-token filter string for complex
  queries. When both positional args and `--filter` are provided, their tokens are merged (AND).
- **D-10:** Named presets are invoked with a `:` prefix: `todotxt list :work` looks up
  `[presets.work]` in config. No fallback behavior — `:work` is a preset lookup only.
- **D-11:** Presets can be combined with extra filter args:
  `todotxt list :work due:today` applies the `work` preset AND filters for `due:today`.
  The preset's filter terms are prepended to the session's filter terms, then AND-combined.
- **D-12:** A plain positional arg without `:` prefix is never treated as a preset name —
  `list work` filters by substring "work", not by the `[presets.work]` preset.

### show Format

- **D-13:** `show <id>` prints the raw todo.txt line exactly as stored in the file — no
  pretty-printing, no field labels, no extra formatting. The `--json` flag wraps it in the
  standard JSON envelope: `{"schema_version": 1, "data": {"id": N, "raw": "..."}}`.

### Quiet Mode Scope

- **D-14:** `--quiet` suppresses **all non-data output**: column headers, "N tasks found" footers,
  "Config created at..." notices, and "No tasks found" messages. Only pure data rows are emitted.
  Error messages (stderr) are **not** suppressed by `--quiet` — errors always surface.

### Agent's Discretion

The following areas have no specific preference — the agent should apply standard practice:

- Color scheme for priority badges (e.g., red for A, yellow for B, green for C, no color for D+)
- `stats` human-readable output format (aligned key-value pairs vs. simple lines)
- `projects` and `contexts` output format (one per line, sorted alphabetically)
- JSON field names for `stats` response (snake_case per project convention from Phase 1)
- Column width policy for the `list` table (task text column fills available terminal width)
- `completions` output goes to stdout with no extra decoration (piped to shell's completion file)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Roadmap & Requirements
- `.planning/ROADMAP.md` — Phase 3 requirements (READ-01–08, CFG-01–02, PLAT-01), deliverables,
  UAT criteria, verification steps, and dependency on Phase 2
- `.planning/REQUIREMENTS.md` — Full requirement descriptions for READ-01–08, CFG-01–02, PLAT-01

### Prior Phase Context (locked decisions)
- `.planning/phases/01-workspace-bootstrap-core-library-foundation/01-CONTEXT.md` —
  Index-based identity (1-based user-facing IDs), `TodoError`/`thiserror` 2.0,
  task builder pattern (`with_*` methods), JSON field naming (`snake_case`)
- `.planning/phases/02-core-library-completion/02-CONTEXT.md` —
  `Filter::from_query()` (space-separated AND tokens), `suppress_hidden`/`suppress_future_threshold`
  defaults (true), `SortOrder` enum, `resolve_config_path()` portable mode API

### Core Library Public API
- `crates/todotxt-core/src/lib.rs` — All public exports: `Filter`, `FilterTerm`, `SortOrder`,
  `Task`, `TaskList`, `LineEnding`, `TodoError`, `resolve_config_path`, optional `FileWatcher`
- `crates/todotxt-core/src/filter.rs` — `Filter::from_query()`, `Filter::new()`, `FilterTerm`
  variants, `TaskList::filter()` return type `Vec<(usize, &Task)>`
- `crates/todotxt-core/src/task_list.rs` — `TaskList::filter()`, `TaskList::sort()`, `TaskList::load()` error contract

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/todotxt-core/src/filter.rs` — `Filter::from_query(q: &str) -> Filter` accepts
  space-separated tokens; use this directly for parsing positional `list` args
- `crates/todotxt-core/src/portable.rs` — `resolve_config_path(binary_dir, platform_dir) -> PathBuf`
  — call this with `directories`-provided path to implement portable mode
- `crates/todotxt-cli/src/main.rs` — stub (`fn main()` placeholder); replace entirely in Phase 3

### Established Patterns
- All CLI errors use `anyhow::Result` + `TodoError` for library errors; exit-code mapping done in `main.rs`
- Clippy `-D warnings` enforced on all crates; use `is_some_and()` / `is_none_or()` idioms
- Integration tests use `assert_cmd` crate (Phase 3 first usage); fixture todo.txt files in `tests/fixtures/`

### Integration Points
- `TaskList::filter(&filter) -> Vec<(usize, &Task)>` — returns 0-based index + task reference;
  CLI displays `index + 1` as the user-visible 1-based ID
- `TodoError` variants that map to exit code 2: `Io`, `Parse` (programmer error); exit code 1:
  task-not-found cases (to be determined in implementation)

</code_context>

<specifics>
## Specific Ideas

- The `:preset` syntax (colon prefix) is deliberate — it avoids ambiguity with filter tokens
  and is visually distinct. `list :work due:today` cleanly composes preset + extra filters.
- "Error if not configured" for missing `todo_file` (D-03) reflects intentional ownership:
  once a user has a config, they own it — the CLI shouldn't silently guess their file path.
- Header-row-only table (D-07) reflects unix-tool aesthetics: scriptable output, minimal chrome.

</specifics>

<deferred>
## Deferred Ideas

None raised during discussion.

</deferred>

---

*Phase: 03-cli-foundation-config-output-read-commands*
*Context gathered: 2026-04-15*

# Requirements: todotxt.net → Rust Port

**Defined:** 2026-04-15
**Milestone:** v1.0 — Core Library + CLI
**Core Value:** A fast, cross-platform todo.txt tool with a first-class CLI for both human and AI agent use.

## v1.0 Requirements

### Core Library (CORE)

- [ ] **CORE-01**: User's todo.txt file is parsed into structured tasks with all fields: priority, projects, contexts, due date, threshold date, creation date, completion date, and body text
- [ ] **CORE-02**: Tasks serialize back to strict todo.txt format, round-tripping cleanly without altering user-authored text
- [ ] **CORE-03**: TaskList supports add, update, and delete operations with atomic file writes (no corruption on crash or concurrent access)
- [ ] **CORE-04**: TaskList detects external file changes and reloads with a 1-second debounce (matches C# FileChangeObserver behavior)
- [ ] **CORE-05**: Filter engine supports: substring match, negation prefix (`-term`), `DONE`/`-DONE` keywords, and `due:today`/`due:past`/`due:future`/`due:active` tokens
- [ ] **CORE-06**: Sort engine supports ordering by: priority, due date, alphabetical, project, context (matching C# SortType options)
- [ ] **CORE-07**: File reader strips UTF-8 BOM and normalizes line endings (CRLF → LF) on load; preserves original line endings on save
- [ ] **CORE-08**: Portable mode: when a config/settings file exists beside the binary, it takes precedence over platform config directories

### CLI — Read Commands (READ)

- [x] **READ-01**: User can list tasks with `list`/`ls`, optionally filtered by inline filter arguments
- [x] **READ-02**: User can view task statistics with `stats`: total, complete, incomplete, due today, overdue counts
- [x] **READ-03**: User can list all `+projects` present in the todo.txt file with `projects`
- [x] **READ-04**: User can list all `@contexts` present in the todo.txt file with `contexts`
- [x] **READ-05**: User can view a single task by numeric ID with `show <id>`
- [x] **READ-06**: User can get structured JSON output from any command with `--json` flag (includes `schema_version` field)
- [x] **READ-07**: User can suppress color output with `--no-color` and suppress informational output with `--quiet`
- [x] **READ-08**: CLI exits with code 0 (success), 1 (not found / no match), or 2 (error) consistently across all commands

### CLI — Write Commands (WRITE)

- [ ] **WRITE-01**: User can add a new task with `add "<task text>"`, optionally auto-prepending creation date
- [ ] **WRITE-02**: User can mark one or more tasks done with `do <id>` (prepends `x <date>` per todo.txt spec)
- [ ] **WRITE-03**: User can unmark a completed task with `undo <id>` (removes `x <date>` prefix)
- [ ] **WRITE-04**: User can delete a task by ID with `del <id>`
- [ ] **WRITE-05**: User can replace a task's full text with `edit <id> "<new text>"`
- [ ] **WRITE-06**: User can append text to a task with `append <id> "<text>"`
- [ ] **WRITE-07**: User can prepend text to a task with `prepend <id> "<text>"`

### CLI — Task Enrichment (ENRICH)

- [x] **ENRICH-01**: User can set or change a task's priority with `pri <id> <A-Z>`
- [x] **ENRICH-02**: User can remove a task's priority with `depri <id>`
- [x] **ENRICH-03**: User can set a due date on a task with `due <id> <date>` (accepts `today`, `tomorrow`, weekday names, and `YYYY-MM-DD`)
- [x] **ENRICH-04**: User can move a task's due date forward by N days with `postpone <id> <N>`

### CLI — Bulk Operations (BULK)

- [ ] **BULK-01**: User can archive all completed tasks to `done.txt` (beside `todo.txt`) with `archive`
- [ ] **BULK-02**: User can delete all completed tasks from `todo.txt` with `del-done`

### Config & Settings (CFG)

- [x] **CFG-01**: CLI reads/writes a TOML config file at a platform-appropriate path (via `directories` crate: `~/.config/todotxt/config.toml` on Linux, `%APPDATA%\todotxt\config.toml` on Windows, `~/Library/Application Support/todotxt/config.toml` on macOS)
- [x] **CFG-02**: User can save and load up to 9 named filter presets in config (e.g., `[presets.work]`, `[presets.today]`)

### Platform (PLAT)

- [x] **PLAT-01**: CLI generates shell completions for bash, zsh, fish, and PowerShell via `completions <shell>` subcommand

## Future Requirements (Deferred)

### TUI Interface

- Interactive terminal UI (ratatui) — SEED-001

### GUI Interface

- Native cross-platform desktop GUI — SEED-002

### todo.sh Compatibility

- Drop-in `todo.sh` command interface — SEED-003

### CI/CD & Release

- GitHub Actions matrix (Windows/Linux/macOS) — SEED-004
- Release binaries via GitHub Releases — SEED-004

## Out of Scope

| Feature | Reason |
|---------|--------|
| TUI interface | Deferred to v1.1 — seed planted |
| Native GUI | Deferred to v1.2 — seed planted |
| todo.sh compatibility layer | Deferred — seed planted |
| CI matrix + release binaries | Deferred — seed planted |
| Interactive prompts / REPL | Anti-feature for agent use; CLI must be scriptable |
| Fuzzy search | Not in reference C# implementation; deferred |
| Windows system tray | GUI milestone responsibility, not CLI |
| Plugin/addon system | todo.sh compatibility scope; not v1.0 |
| Web/network sync | Out of scope for this project |

## Traceability

| REQ-ID | Phase | Status |
|--------|-------|--------|
| CORE-01 | Phase 1 | Pending |
| CORE-02 | Phase 1 | Pending |
| CORE-03 | Phase 1 | Pending |
| CORE-07 | Phase 1 | Pending |
| CORE-04 | Phase 2 | Pending |
| CORE-05 | Phase 2 | Pending |
| CORE-06 | Phase 2 | Pending |
| CORE-08 | Phase 2 | Pending |
| READ-01 | Phase 3 | Complete |
| READ-02 | Phase 3 | Complete |
| READ-03 | Phase 3 | Complete |
| READ-04 | Phase 3 | Complete |
| READ-05 | Phase 3 | Complete |
| READ-06 | Phase 3 | Complete |
| READ-07 | Phase 3 | Complete |
| READ-08 | Phase 3 | Complete |
| CFG-01 | Phase 3 | Complete |
| CFG-02 | Phase 3 | Complete |
| PLAT-01 | Phase 3 | Complete |
| WRITE-01 | Phase 4 | Pending |
| WRITE-02 | Phase 4 | Pending |
| WRITE-03 | Phase 4 | Pending |
| WRITE-04 | Phase 4 | Pending |
| WRITE-05 | Phase 4 | Pending |
| WRITE-06 | Phase 4 | Pending |
| WRITE-07 | Phase 4 | Pending |
| ENRICH-01 | Phase 5 | Complete |
| ENRICH-02 | Phase 5 | Complete |
| ENRICH-03 | Phase 5 | Complete |
| ENRICH-04 | Phase 5 | Complete |
| BULK-01 | Phase 5 | Pending |
| BULK-02 | Phase 5 | Pending |

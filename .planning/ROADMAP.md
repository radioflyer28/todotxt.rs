# Roadmap: v1.0 — Core Library + CLI

**Milestone:** v1.0 — todotxt.net Rust Port: Core Library + CLI
**Phase count:** 8
**Status:** In Progress

---

## Phases

- [ ] **Phase 1: Workspace Bootstrap + Core Library Foundation** — Cargo workspace, `todotxt-core` crate: parser, Task model, TaskList CRUD, atomic writes, BOM/CRLF handling
- [ ] **Phase 2: Core Library Completion** — Filter engine, sort engine, file watching, batch operations, portable mode
- [ ] **Phase 3: CLI Foundation — Config + Output + Read Commands** — All cross-cutting CLI conventions, config/presets, all read commands, shell completions
- [ ] **Phase 4: CLI Write Commands** — Full task lifecycle: add, complete, undo, delete, edit, append, prepend
- [ ] **Phase 5: Task Enrichment + Bulk Operations** — Priority commands, due date commands, archive, del-done
- [ ] **Phase 6: Cross-Platform Polish + Integration Tests** — Cross-platform validation, integration test suite, README
- [ ] **Phase 7: Retroactive Core Library Verification** — Produce Phase 01 VERIFICATION.md; correct Phase 02 VERIFICATION.md REQ-ID mapping for CORE-04..08
- [ ] **Phase 8: Retroactive CLI Verification** — Produce Phase 04 VERIFICATION.md (WRITE-01..07); Phase 05 VERIFICATION.md (ENRICH-01..04, BULK-01..02); Phase 06 VERIFICATION.md

---

## Phase Details

### Phase 1: Workspace Bootstrap + Core Library Foundation

**Goal:** Establish the Cargo workspace and implement the `todotxt-core` crate with a single-pass parser, immutable Task model, TaskList CRUD, and atomic file writes — resolving all critical C# data-layer bugs before any other code is written.

**Requirements:**
- CORE-01: todo.txt parser — all fields (priority, projects, contexts, due/threshold/creation/completion dates, body)
- CORE-02: Task serializer — strict round-trip (no mutating user-authored text)
- CORE-03: TaskList CRUD — atomic file writes (write to `.tmp`, rename)
- CORE-07: UTF-8 BOM stripping + CRLF/LF normalization on load; preserve on save

**Plans:** 2 plans

Plans:
- [x] 01-01-PLAN.md — Workspace scaffold + Task parser with winnow + builder methods + tests
- [x] 01-02-PLAN.md — TaskList with atomic file I/O, BOM/CRLF handling + integration tests

**Deliverables:**
- `Cargo.toml` workspace at repo root with members `crates/todotxt-core` and `crates/todotxt-cli`
- `crates/todotxt-core/src/error.rs`: `TodoError` enum via `thiserror` 2.0
- `crates/todotxt-core/src/task.rs`: `Task` struct with all todo.txt fields; `winnow` 1.0.1 single-pass parser; builder mutation methods (`with_completed`, `with_priority`, `with_due`, etc.)
- `crates/todotxt-core/src/task_list.rs`: `TaskList` with index-based identity, `add()`, `update()`, `delete()`, atomic writes via `tempfile::NamedTempFile::persist()`, BOM stripping (`\u{FEFF}`), CRLF detection and round-trip preservation
- `crates/todotxt-core/src/lib.rs`: public API re-exports
- `crates/todotxt-core/tests/`: unit tests via `rstest`; `insta` snapshot round-trip tests; duplicate-task deletion test; BOM/CRLF fixture tests

**UAT Criteria:**
- [ ] Given a todo.txt file with priority, projects, contexts, creation date, due date, and threshold date on the same task, `Task::parse()` returns a struct with all fields correctly populated and `task.to_string()` reproduces the original line byte-for-byte
- [ ] Given a todo.txt file with a UTF-8 BOM (`\u{FEFF}`) on the first line, `TaskList::load()` strips the BOM and parses tasks without error; the saved file does not contain a BOM
- [ ] Given a todo.txt file with Windows CRLF line endings, `TaskList::load()` parses correctly and `TaskList::save()` writes CRLF line endings back
- [ ] Given a `TaskList` with 3 tasks, calling `delete(1)` (0-based index) removes the second task and `save()` writes exactly 2 lines to a new file atomically (original file is not truncated mid-write if the process is killed)
- [ ] `cargo test -p todotxt-core` passes with zero failures and zero warnings

**Verification:**
- [ ] `cargo test -p todotxt-core` passes
- [ ] `cargo clippy -p todotxt-core -- -D warnings` passes
- [ ] `insta` snapshot tests show no unexpected diffs (`cargo insta test`)
- [ ] Round-trip property: for every line in `tests/fixtures/todo.txt`, `Task::parse(line).to_string() == line`

**Depends on:** None

---

### Phase 2: Core Library Completion

**Goal:** Complete the `todotxt-core` public API with a full filter engine, sort engine, file watching with debounce, batch mutations, and portable mode — delivering a self-contained library with no remaining open requirements.

**Requirements:**
- CORE-04: File watching — 1-second debounce (`notify` + `notify-debouncer-mini`)
- CORE-05: Filter engine — substring, negation, `DONE`/`-DONE`, `due:` tokens, `h:1` hidden tag, threshold suppression
- CORE-06: Sort engine — priority, due date, alphabetical, project, context
- CORE-08: Portable mode — config beside binary takes precedence over platform config dirs

**Deliverables:**
- `crates/todotxt-core/src/filter.rs`: `Filter` type and `TaskList::filter(&filter)` method; supports substring match, `-term` negation, `DONE`/`-DONE` keywords, `due:today`/`due:past`/`due:future`/`due:active` tokens, `h:1` hidden tag exclusion, threshold date suppression (tasks with `t:YYYY-MM-DD` > today hidden)
- `crates/todotxt-core/src/sort.rs`: `SortOrder` enum (Priority, DueDate, Alphabetical, Project, Context) and `TaskList::sort(order)` method
- `crates/todotxt-core/src/watcher.rs`: `FileWatcher` wrapping `notify` 8.2.0 + `notify-debouncer-mini` 0.7.0; fires callback after 1-second debounce; exposed as optional feature `watching`
- `task_list.rs` additions: `batch_update(ids, f)` for multi-task mutations; `reload()` to re-read file from disk
- `crates/todotxt-core/src/portable.rs`: `resolve_config_path(binary_dir, platform_dir) -> PathBuf` — returns binary-adjacent path when config file exists there
- `crates/todotxt-core/tests/`: filter matrix tests (all token types); sort stability tests; file watcher integration test (requires real filesystem, gated with `#[ignore]` by default)

**UAT Criteria:**
- [ ] Given tasks mixing complete and incomplete items, `filter("DONE")` returns only completed tasks and `filter("-DONE")` returns only incomplete tasks
- [ ] Given tasks where some have `due:` dates in the past, present, and future, `filter("due:today")` returns only today's tasks, `filter("due:past")` overdue only, `filter("due:active")` returns today + past
- [ ] Given a task with `h:1` tag, it does not appear in any filter result (it is hidden); a task with `t:` threshold in the future is also suppressed
- [ ] Given tasks with mixed priorities and due dates, `sort(Priority)` places `(A)` before `(B)` before unprioritized; `sort(DueDate)` places earliest dates first with no-due-date tasks last
- [ ] Given a config file placed beside the binary at `./config.toml`, `resolve_config_path()` returns that path even when a platform config dir also exists
- [ ] `cargo test -p todotxt-core` passes (all tests including new filter/sort/portable tests)

**Verification:**
- [ ] `cargo test -p todotxt-core` passes
- [ ] `cargo clippy -p todotxt-core -- -D warnings` passes
- [ ] Filter token matrix: each of the 8 filter modes has at least one passing test
- [ ] Sort test: all 5 sort orders have at least one passing test
- [ ] `cargo insta test` — no unexpected snapshot diffs

**Depends on:** Phase 1

---

### Phase 3: CLI Foundation — Config + Output + Read Commands

**Goal:** Establish all cross-cutting CLI conventions (output discipline, exit codes, JSON envelope, color, config loading) and implement every read command — delivering a fully usable read-only CLI.

**Requirements:**
- READ-01: `list`/`ls` with filter args
- READ-02: `stats` command
- READ-03: `projects` command
- READ-04: `contexts` command
- READ-05: `show <id>` single task
- READ-06: `--json` flag with `schema_version` field
- READ-07: `--no-color` / `--quiet` flags; `NO_COLOR` env var
- READ-08: Consistent exit codes (0=success, 1=not found, 2=error)
- CFG-01: TOML config at platform-appropriate path (`directories` crate)
- CFG-02: Named filter presets in config (up to 9)
- PLAT-01: Shell completions via `completions <shell>` subcommand

**Deliverables:**
- `crates/todotxt-cli/Cargo.toml`: dependencies (`clap` 4.6, `anyhow` 1.0, `owo-colors`, `comfy-table`, `serde_json`, `directories` 6.0, `toml`)
- `crates/todotxt-cli/src/config.rs`: `Config` struct; TOML load/save; `directories`-based path resolution; portable mode integration; `[presets.*]` named filter preset support (max 9)
- `crates/todotxt-cli/src/output.rs`: `Renderer` with human (table/colored) and JSON modes; stdout for data, stderr for info/errors; `--no-color`/`NO_COLOR` detection via `owo-colors`; `--quiet` flag suppresses info lines; JSON envelope shape `{"schema_version": 1, "data": ...}`
- `crates/todotxt-cli/src/cli.rs`: `clap` derive structs for all global flags (`--json`, `--no-color`, `--quiet`, `--config`, `--todo-file`) and all subcommands
- `crates/todotxt-cli/src/main.rs`: `fn main() -> anyhow::Result<()>`; exit-code mapping (0/1/2); JSON error envelope on `--json` + error
- `crates/todotxt-cli/src/commands/list.rs`, `stats.rs`, `projects.rs`, `contexts.rs`, `show.rs`
- `crates/todotxt-cli/src/commands/completions.rs`: `clap_complete` shell completion generation for bash, zsh, fish, PowerShell
- `crates/todotxt-cli/tests/`: `assert_cmd` integration tests for all read commands; exit-code assertions; JSON output schema validation

**Plans:** 5/5 plans executed

Plans:
- [x] 03-01-PLAN.md — Foundation: Cargo.toml deps, config.rs, output.rs (Wave 1)
- [x] 03-02-PLAN.md — CLI wiring: cli.rs, main.rs, list/stats/projects/contexts/show commands (Wave 2)
- [x] 03-03-PLAN.md — Completions + integration tests (Wave 3)
- [x] 03-04-PLAN.md — Gap closure: core CR normalization + mixed line-ending regression tests (Wave 4)
- [x] 03-05-PLAN.md — Gap closure: default incomplete-only list semantics + unknown-preset/output regressions (Wave 5)
**UAT Criteria:**
- [ ] `todotxt list` prints all tasks in a formatted table with priority colorization; `todotxt ls +work` prints only tasks containing `+work`
- [ ] `todotxt stats` prints total, complete, incomplete, due-today, and overdue counts as human-readable text; `todotxt stats --json` returns a JSON object with `schema_version: 1` and those same counts as fields
- [ ] `todotxt show 1` prints the first task; `todotxt show 999` exits with code 1 and prints an error to stderr
- [ ] `todotxt list --no-color` produces output with no ANSI escape sequences; `NO_COLOR=1 todotxt list` has the same effect
- [ ] `todotxt list --json` on an empty todo.txt exits with code 0 and returns `{"schema_version":1,"data":[]}`; `todotxt list --json` when todo.txt does not exist exits with code 2 and returns `{"schema_version":1,"error":"..."}`
- [ ] `todotxt completions bash` prints a bash completion script to stdout with no errors; same for `zsh`, `fish`, `powershell`
- [ ] Given `[presets.work]` filter in config, `todotxt list work` applies that preset's filter as if passed inline

**Verification:**
- [ ] `cargo test -p todotxt-cli` passes (including `assert_cmd` integration tests)
- [ ] `cargo clippy -p todotxt-cli -- -D warnings` passes
- [ ] Manual smoke test: `todotxt list`, `todotxt stats`, `todotxt show 1`, `todotxt list --json` all produce correct output against a sample `todo.txt`
- [ ] Exit code contract: `echo $?` after each error scenario returns expected code (0, 1, or 2)

**Depends on:** Phase 2

**UI hint**: yes

---

### Phase 4: CLI Write Commands

**Goal:** Complete the full task lifecycle — add, complete, undo, delete, edit, append, prepend — giving users and agents the ability to create and mutate tasks through the CLI.

**Requirements:**

**Plans:** 0/5 plans executed

Plans:
- [x] 04-01-PLAN.md — Core builder (with_text_prepended) + Config/Output infra (Wave 1)
- [x] 04-02-PLAN.md — CLI scaffold + add command (Wave 2)
- [x] 04-03-PLAN.md — do/undo/del commands (Wave 3)
- [x] 04-04-PLAN.md — edit/append/prepend commands (Wave 3, parallel)
- [x] 04-05-PLAN.md — Integration tests (Wave 4)

- WRITE-01: `add "<text>"` with optional auto-creation-date
- WRITE-02: `do <id>` — mark complete (prepend `x <date>`)
- WRITE-03: `undo <id>` — unmark complete
- WRITE-04: `del <id>` — delete by ID
- WRITE-05: `edit <id> "<text>"` — full replace
- WRITE-06: `append <id> "<text>"`
- WRITE-07: `prepend <id> "<text>"`

**Deliverables:**
- `crates/todotxt-cli/src/commands/add.rs`: parses `add "<text>"`; `--no-date` flag to suppress auto-creation-date; date injected as `YYYY-MM-DD` prefix when flag absent; prints new task ID to stdout
- `crates/todotxt-cli/src/commands/complete.rs`: `do <id>` prepends `x YYYY-MM-DD ` per todo.txt spec; `undo <id>` strips `x <date> ` prefix; both accept single ID
- `crates/todotxt-cli/src/commands/delete.rs`: `del <id>` removes task by 1-based ID; prints deleted task text to stdout before removal
- `crates/todotxt-cli/src/commands/edit.rs`: `edit <id> "<text>"` fully replaces task text; does not inject creation date
- `crates/todotxt-cli/src/commands/append.rs`: `append <id> "<text>"` appends text with a leading space
- `crates/todotxt-cli/src/commands/prepend.rs`: `prepend <id> "<text>"` inserts text after any priority/date prefix, with a trailing space
- All write commands use atomic save from `TaskList::save()`
- Integration tests for all write commands: file state assertions post-command; `--json` output assertions; error path assertions (invalid ID → exit 1)

**UAT Criteria:**
- [ ] `todotxt add "Buy milk +shopping"` creates a new task as the last line of `todo.txt` with today's date prepended (e.g., `2026-04-14 Buy milk +shopping`) and prints `Added: 5` (or current last ID + 1)
- [ ] `todotxt do 2` marks task 2 done: line becomes `x 2026-04-14 <original text>`; `todotxt undo 2` removes the `x YYYY-MM-DD ` prefix, restoring the original text exactly
- [ ] `todotxt del 3` removes task 3 from `todo.txt`, prints the deleted line to stdout, and subsequent `todotxt list` does not show that task (IDs of remaining tasks shift down by 1)
- [ ] `todotxt edit 1 "New task text +project"` replaces task 1's entire text with `New task text +project`; the old text is gone
- [ ] `todotxt append 2 "due:2026-05-01"` results in task 2's line ending with ` due:2026-05-01`
- [ ] `todotxt prepend 2 "(A)"` results in task 2's line starting with `(A) ` (prepended before other text, after existing priority if any — behavior matches todo.txt spec)
- [ ] All write commands with `--json` return `{"schema_version":1,"data":{...}}` with the affected task's fields

**Verification:**
- [ ] `cargo test -p todotxt-cli` passes (including new write command tests)
- [ ] `cargo test --workspace` passes
- [ ] Atomic write test: file is not corrupted if process is killed after `NamedTempFile` write but before rename (manual or test-harness verification)
- [ ] `cargo clippy --workspace -- -D warnings` passes

**Depends on:** Phase 3

---

### Phase 5: Task Enrichment + Bulk Operations

**Goal:** Add priority manipulation, smart due-date management, and bulk archive/cleanup operations — completing the full command surface of the CLI.

**Plans:** 5/6 plans executed

Plans:
- [x] 05-01-PLAN.md — Config extension + CLI wiring foundation (Wave 1)
- [x] 05-02-PLAN.md — Date parsing utility for due/postpone (Wave 2)
- [x] 05-03-PLAN.md — pri/depri commands with multi-ID support (Wave 3)
- [x] 05-04-PLAN.md — due/postpone commands with date parsing (Wave 3, parallel)
- [x] 05-05-PLAN.md — archive/del-done commands with atomicity (Wave 4)
- [ ] 05-06-PLAN.md — Integration test suite (29+ tests) (Wave 5)

**Requirements:**
- ENRICH-01: `pri <id> <A-Z>` — set priority
- ENRICH-02: `depri <id>` — remove priority
- ENRICH-03: `due <id> <date>` — set due date (accepts `today`, `tomorrow`, weekday names, `YYYY-MM-DD`)
- ENRICH-04: `postpone <id> <N>` — move due date forward N days
- BULK-01: `archive` — move completed tasks to `done.txt`
- BULK-02: `del-done` — delete all completed tasks from `todo.txt`

**Deliverables:**
- `crates/todotxt-cli/src/commands/priority.rs`: `pri <id> <A-Z>` sets or replaces priority; `depri <id>` removes priority token; validates letter A–Z (case-insensitive, normalized to uppercase)
- `crates/todotxt-cli/src/commands/due.rs`: `due <id> <date>` parses natural date inputs (`today`, `tomorrow`, `monday`–`sunday`, `YYYY-MM-DD`) and sets/replaces `due:YYYY-MM-DD` tag on task; `postpone <id> <N>` shifts existing due date forward by N calendar days (errors if no due date set)
- `crates/todotxt-cli/src/commands/archive.rs`: `archive` moves all completed tasks (lines starting with `x `) from `todo.txt` to `done.txt` (appends; creates `done.txt` if absent); prints count moved; atomic: both files written before either is modified
- `crates/todotxt-cli/src/commands/del_done.rs`: `del-done` deletes all completed tasks from `todo.txt` in-place; prints count deleted
- Date parsing utility in `crates/todotxt-cli/src/date.rs`: `parse_date_input(s: &str, today: NaiveDate) -> Result<NaiveDate>` (shared by `due` and `postpone`)
- Integration tests: `pri`/`depri` round-trips; due date parsing for all input formats; `postpone` with valid and missing due date; `archive` creates `done.txt` and leaves `todo.txt` clean; `del-done` count accuracy

**UAT Criteria:**
- [ ] `todotxt pri 3 A` sets task 3's priority to `(A)`; if task 3 already had `(B)`, the line now reads `(A) <rest>`; `todotxt depri 3` removes the priority entirely
- [ ] `todotxt due 2 tomorrow` sets task 2's due date to `due:YYYY-MM-DD` (tomorrow's date); `todotxt due 2 friday` sets due date to the next upcoming Friday; `todotxt due 2 2026-12-31` sets it to `due:2026-12-31`
- [ ] `todotxt postpone 4 7` moves task 4's due date 7 days forward; if task 4 has no due date, command exits with code 2 and an error message
- [ ] `todotxt archive` moves all `x`-prefixed tasks to `done.txt` (creating it if needed); `todo.txt` contains no completed tasks afterward; running `archive` on an already-clean file prints "0 tasks archived" and exits 0
- [ ] `todotxt del-done` deletes all completed tasks from `todo.txt`; prints "Deleted N completed tasks"; N matches the count from `todotxt stats` (completed count)
- [ ] All enrichment and bulk commands respect `--json` flag and return structured output with `schema_version: 1`

**Verification:**
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] Date parsing test matrix: `today`, `tomorrow`, `monday`–`sunday`, `YYYY-MM-DD` — all parse to correct `NaiveDate` relative to a fixed test date
- [ ] Archive atomicity: both `todo.txt` and `done.txt` reflect correct state after `archive`; no task appears in both files

**Depends on:** Phase 4

---

### Phase 6: Cross-Platform Polish + Integration Tests

**Goal:** Validate correct behavior on Windows, Linux, and macOS; harden the integration test suite; and produce a README that makes the tool immediately usable by humans and agents.

**Requirements:**
- (All CORE, READ, WRITE, ENRICH, BULK, CFG, PLAT requirements verified end-to-end)

**Deliverables:**
- `crates/todotxt-core/tests/platform.rs`: Windows CRLF fixture round-trip test; macOS config path resolution test; Linux XDG path resolution test — each gated with `#[cfg(target_os = ...)]` or `#[ignore]` for CI cross-platform matrix
- `crates/todotxt-cli/tests/integration/`: end-to-end scenario tests using `assert_cmd` + `tempdir`: full workflow (add → list → do → archive); JSON output contract tests for every command; exit code contract tests for all error paths
- `README.md` updated at repo root: Installation (cargo install + pre-built binary link placeholder); Quick Start (5-command walkthrough); Full command reference table; JSON schema documentation (fields, `schema_version`, error envelope); Config file format + preset example; Shell completion instructions for all 4 shells
- `crates/todotxt-core/src/lib.rs` and `crates/todotxt-cli/src/main.rs`: `#![deny(warnings)]` added; all `todo!()` / `unimplemented!()` macros removed
- `.github/workflows/` placeholder: `ci.yml` with `cargo test --workspace` step documented (not wired to CI yet — SEED-004 scope)

**UAT Criteria:**
- [ ] `cargo test --workspace` passes with zero failures on the development machine
- [ ] `cargo clippy --workspace -- -D warnings` produces zero warnings
- [ ] Full workflow smoke test: on a fresh `todo.txt`, run `add` → `list` → `do` → `stats` → `archive` in sequence; each command produces correct output and the file state is correct after each step
- [ ] `todotxt list --json | python -m json.tool` (or `jq .`) succeeds — output is valid JSON with `schema_version` field present
- [ ] README Quick Start: a developer with Rust installed can follow the README and have `todotxt list` working in under 5 minutes
- [ ] Shell completion smoke test: `todotxt completions bash | bash` does not produce errors; same for `zsh`, `fish`, `powershell`

**Verification:**
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo doc --workspace --no-deps` builds without errors
- [ ] JSON schema contract: `schema_version: 1` present in all `--json` outputs (verified by grep over integration test fixtures)
- [ ] No `unwrap()` calls in library code (`grep -r "\.unwrap()" crates/todotxt-core/src/` returns zero results outside test files)

**Depends on:** Phase 5

**Plans:**
- [ ] 06-01-PLAN.md — Wave 1: Compiler hardening (deny warnings, unwrap audit)
- [ ] 06-02-PLAN.md — Wave 2: Platform tests (CRLF round-trip, portable mode)
- [ ] 06-03-PLAN.md — Wave 2: E2E integration scenario tests
- [ ] 06-04-PLAN.md — Wave 3: CI yml + README (7 sections)

---

### Phase 7: Retroactive Core Library Verification

**Goal:** Produce the missing Phase 01 VERIFICATION.md and correct the Phase 02 VERIFICATION.md requirement traceability so that CORE-01..03 + CORE-07 (Phase 1) and CORE-04..06 + CORE-08 (Phase 2) are properly attributed and evidenced — unblocking the milestone audit score for the core library requirement group.

**Requirements:**
- CORE-01: todo.txt parser (Phase 1 closure)
- CORE-02: Task serializer (Phase 1 closure)
- CORE-03: TaskList CRUD (Phase 1 closure)
- CORE-07: BOM/CRLF handling (Phase 1 closure)
- CORE-04: File watching (Phase 2 REQ-ID correction)
- CORE-05: Filter engine (Phase 2 REQ-ID correction)
- CORE-06: Sort engine (Phase 2 REQ-ID correction)
- CORE-08: Portable mode (Phase 2 REQ-ID correction)

**Gap Closure:** Closes CORE-01..08 orphaned/unsatisfied gaps from v1.0-MILESTONE-AUDIT.md.

**Deliverables:**
- `.planning/phases/01-workspace-bootstrap-core-library-foundation/01-VERIFICATION.md`: full phase verification report for Phase 1 (test evidence, UAT closure, deliverable checklist)
- `.planning/phases/02-core-library-completion/02-VERIFICATION.md`: corrected report with accurate REQ-ID traceability (CORE-04..08); existing passing evidence preserved
- `REQUIREMENTS.md` traceability table: CORE-01..08 all moved to `Pending` → satisfied under new verification closure

**Plans:**
- [ ] 07-01-PLAN.md — Produce Phase 01 VERIFICATION.md (Wave 1)
- [ ] 07-02-PLAN.md — Correct Phase 02 VERIFICATION.md REQ-ID mapping (Wave 2)

**UAT Criteria:**
- [ ] `01-VERIFICATION.md` exists with status `passed`, lists CORE-01..03 + CORE-07 in requirement traceability table
- [ ] `02-VERIFICATION.md` requirement traceability table correctly maps CORE-04 (file watch), CORE-05 (filter), CORE-06 (sort), CORE-08 (portable mode) with evidence
- [ ] Re-running milestone audit shows CORE group as 8/8 satisfied

**Verification:**
- [ ] Both VERIFICATION.md files have `status: passed` frontmatter
- [ ] REQUIREMENTS.md traceability shows CORE-01..08 with updated verification phase

**Depends on:** Phase 6

---

### Phase 8: Retroactive CLI Verification

**Goal:** Produce the missing VERIFICATION.md reports for Phases 04, 05, and 06 — formally closing WRITE-01..07, ENRICH-01..04, BULK-01..02, and the Phase 06 cross-platform umbrella against existing implementation evidence and passing tests.

**Requirements:**
- WRITE-01..07 (Phase 4 closure)
- ENRICH-01..04 (Phase 5 closure)
- BULK-01..02 (Phase 5 closure)
- Phase 06 umbrella: deny warnings, platform tests, E2E tests, README

**Gap Closure:** Closes WRITE-01..07, ENRICH-01..04, BULK-01..02 unsatisfied/orphaned gaps from v1.0-MILESTONE-AUDIT.md; closes Phase 06 missing verification.

**Deliverables:**
- `.planning/phases/04-cli-write-commands-update-archive/04-VERIFICATION.md`: verification report for WRITE-01..07 with test suite evidence and UAT closure
- `.planning/phases/05-task-enrichment-bulk-operations/05-VERIFICATION.md`: verification report for ENRICH-01..04 + BULK-01..02 with test suite evidence
- `.planning/phases/06-cross-platform-polish-integration-tests/06-VERIFICATION.md`: verification report confirming deny warnings, platform tests, E2E scenarios, README sections
- `REQUIREMENTS.md` traceability: WRITE-01..07, ENRICH-01..04, BULK-01..02 updated to `Complete` in traceability table

**Plans:**
- [ ] 08-01-PLAN.md — Produce Phase 04 VERIFICATION.md (WRITE-01..07) (Wave 1)
- [ ] 08-02-PLAN.md — Produce Phase 05 VERIFICATION.md (ENRICH/BULK) (Wave 2)
- [ ] 08-03-PLAN.md — Produce Phase 06 VERIFICATION.md (Wave 2, parallel)

**UAT Criteria:**
- [ ] All three VERIFICATION.md files have `status: passed` frontmatter
- [ ] `04-VERIFICATION.md` lists WRITE-01..07 in traceability table with `cargo test -p todotxt-cli` evidence
- [ ] `05-VERIFICATION.md` lists ENRICH-01..04 and BULK-01..02 with command and test evidence
- [ ] `06-VERIFICATION.md` covers deny warnings, platform CRLF tests, 5 E2E scenarios, 7-section README
- [ ] Re-running milestone audit shows 32/32 requirements satisfied

**Verification:**
- [ ] All three VERIFICATION.md files exist with `status: passed`
- [ ] `cargo test --workspace` passes (unchanged)
- [ ] `cargo clippy --workspace -- -D warnings` passes (unchanged)

**Depends on:** Phase 7

---

## Progress Table

| Phase | Goal Summary | Requirements | Status | Completed |
|-------|-------------|--------------|--------|-----------|
| 1. Workspace Bootstrap + Core Library Foundation | Parser, Task model, TaskList CRUD, atomic writes | CORE-01, CORE-02, CORE-03, CORE-07 | Complete | — |
| 2. Core Library Completion | Filter, sort, file watch, portable mode | CORE-04, CORE-05, CORE-06, CORE-08 | Complete | — |
| 3. CLI Foundation — Config + Output + Read Commands | Config, output, all read commands, completions | READ-01..08, CFG-01..02, PLAT-01 | Complete | — |
| 4. CLI Write Commands | Add, do, undo, del, edit, append, prepend | WRITE-01..07 | Complete | — |
| 5. Task Enrichment + Bulk Operations | Pri, depri, due, postpone, archive, del-done | ENRICH-01..04, BULK-01..02 | Complete | — |
| 6. Cross-Platform Polish + Integration Tests | E2E validation, README, cross-platform tests | All (verification) | Complete | — |
| 7. Retroactive Core Library Verification | Phase 01 + 02 VERIFICATION.md | CORE-01..08 | Not started | — |
| 8. Retroactive CLI Verification | Phase 04 + 05 + 06 VERIFICATION.md | WRITE-01..07, ENRICH-01..04, BULK-01..02 | Not started | — |

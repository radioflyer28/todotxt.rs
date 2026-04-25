# Phase 21: Smart Text Normalization — Context

**Gathered:** 2026-04-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Make TUI append and edit flows todo.txt-aware: when the user appends or edits text
containing recognized todo.txt tokens (`(A)`, `+project`, `@context`, `due:`, `t:`),
those tokens land in canonical field positions rather than being tacked onto the raw string.

Phase 21 delivers:
1. **`normalize_append` in `todotxt-core`** — shared fn that merges appended token text
   into an existing Task's fields and rebuilds via `rebuild_raw()`
2. **TUI append wired** — `handle_append_text_key` calls `normalize_append` when
   `normalize_append` is enabled in config; falls back to raw append when disabled
3. **TUI edit wired** — `save_and_exit` applies same normalization on save when
   `normalize_edit` is enabled in config
4. **CLI `append` extended** — CLI append command accepts a `--normalize` flag that
   calls the same `normalize_append` helper from `todotxt-core`
5. **Config toggles** — `normalize_append` and `normalize_edit` bool flags in `config.toml`

Phase 20 deferred all token-aware behavior here (Phase 20 D-08). Phase 22 owns
keymap and help parity. Phase 23 owns final UAT.

</domain>

<decisions>
## Implementation Decisions

### Append Merge Strategy (NORM-01 – NORM-05)

- **D-01:** Use **parse-then-merge**: parse the appended text into recognized tokens
  (priority, +projects, @contexts, due:, t:, plain body words), merge each into the
  corresponding field of the existing `Task` struct, then rebuild via `rebuild_raw()`.
  Do NOT use append-then-reparse (naive concatenation): `Task::parse` on a combined
  string absorbs appended tokens into body instead of elevating them to struct fields.
- **D-02:** Unknown / unrecognized tokens (e.g., `rec:+1w`, `foo:bar`, arbitrary key:value)
  are preserved verbatim in `body` as-is. This is the existing `rebuild_raw()` behavior
  (NORM-05). No `custom_fields` map needed in `Task` struct — out of scope.

### Priority Conflict (NORM-01)

- **D-03:** When appended text contains a priority token and the original task already
  has a priority, the **appended priority wins** (replaces the original). Example:
  `"(A) fix bug"` + append `"(B)"` → `"(B) fix bug"`. This lets the user reprioritize
  from the append bar without opening the full edit flow.
- **D-04:** When appended text contains a priority token and the original task has no
  priority, the token becomes the task's priority (standard merge behavior).

### Edit Flow Normalization Scope (NORM-01 – NORM-04)

- **D-05:** Edit flow normalization is **configurable** via `normalize_edit = true/false`
  in `config.toml`. When enabled, `save_and_exit()` applies the same parse-then-merge
  normalization to the edited text. When disabled, edit saves the raw edited string via
  `Task::parse()` (current behavior — user controls token placement manually).
- **D-06:** Default value for `normalize_edit` is **true** (normalizing on edit is the
  expected behavior for most users; power users who want raw control opt out).

### Append Flow Normalization Toggle

- **D-07:** Separate toggle `normalize_append = true/false` in `config.toml`, independent
  of `normalize_edit`. Default: **true**.
- **D-08:** When `normalize_append = false`, TUI append falls back to the Phase 20
  behavior (raw concatenation). The `--normalize` flag on the CLI is additive — it
  triggers normalization regardless of any config default.

### CLI `append` Command (NORM-06)

- **D-09:** The CLI `append` command gains a `--normalize` flag. When passed, it calls
  `normalize_append()` from `todotxt-core` before writing the task. Without `--normalize`,
  existing CLI behavior (raw string append) is unchanged — this is NOT a breaking change.
- **D-10:** The CLI does NOT read `normalize_append` from config.toml automatically —
  normalization must be explicitly requested via `--normalize` for CLI invocations. This
  keeps CLI behavior predictable for scripts and AI agents.

### Core API Shape (NORM-06)

- **D-11:** The shared helper lives in `todotxt-core` as a **standalone function**:
  ```rust
  pub fn normalize_append(task: &Task, append_text: &str) -> Task
  ```
  This is the primary extension point. Both TUI and CLI call this function directly.
  It is NOT a builder method — the existing `with_*` builder pattern is for single-field
  mutations; `normalize_append` is a multi-field merge operation.
- **D-12:** `normalize_append` is in `todotxt-core/src/task.rs` (or a new
  `todotxt-core/src/normalize.rs` if the planner judges the function complex enough to
  warrant its own module — planner decides).
- **D-13:** `normalize_append` is **publicly exported** from `todotxt-core` so both
  `todotxt-tui` and `todotxt-cli` can import it without duplication.

### Agent's Discretion

- Whether `normalize_append` lives in `task.rs` or a new `normalize.rs` — planner decides
  based on estimated function complexity.
- Default values for `normalize_append` and `normalize_edit` config flags if a `config.toml`
  section already defines normalization-adjacent settings — planner aligns with existing
  config section conventions.
- How the TUI reads these new config toggles (direct field on existing `Config` struct vs
  a nested `[normalization]` table) — planner decides based on existing config structure.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Core normalization infrastructure
- `crates/todotxt-core/src/task.rs` — `Task` struct fields (priority, projects, contexts,
  due_date, threshold_date, body), `rebuild_raw()` fn (canonical serialization — the
  output format `normalize_append` must produce), existing `with_*` builder methods
  (pattern to follow), `Task::parse()` (used to re-parse after rebuild)

### TUI integration points
- `crates/todotxt-tui/src/app.rs` — `handle_append_text_key()` (Phase 20 implementation
  of bulk append — this is where `normalize_append` is wired in), `save_and_exit()`
  (edit commit path — where `normalize_edit` toggle is checked), `AppMode` enum,
  `App` struct

### TUI config
- `crates/todotxt-tui/src/` — TUI config struct (locate config.toml parsing; `normalize_append`
  and `normalize_edit` fields are added here)

### CLI integration point
- `crates/todotxt-cli/src/` — `append` command handler (locate the append subcommand;
  `--normalize` flag is added here)

### Phase 20 contracts (must not be broken)
- `.planning/phases/20-bulk-actions-selection-ux/20-CONTEXT.md` — D-08 (append flow
  currently does raw concat; Phase 21 wraps that path with normalization), D-09
  (descending-index order for symmetry — unchanged)

### Requirements
- `.planning/REQUIREMENTS.md` — NORM-01 (priority placement), NORM-02 (project tags),
  NORM-03 (context preservation), NORM-04 (due/t: normalization), NORM-05 (unknown
  text preserved), NORM-06 (shared core logic, no TUI duplication)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `rebuild_raw(task: &Task) -> String` in `todotxt-core/src/task.rs` — already outputs
  canonical format (priority → creation_date → body → +projects → @contexts → due: → t:).
  `normalize_append` calls this after merging fields.
- `Task::parse(line: &str) -> Task` — used as final step after `rebuild_raw` to ensure
  all fields stay in sync (existing pattern in `with_*` builders).
- `handle_append_text_key()` in `app.rs` — Phase 20 implementation. Currently does:
  `format!("{} {}", t.to_raw().trim_end(), text)` then `Task::parse(&new_raw)`.
  Phase 21 replaces the inner logic with a call to `normalize_append` when the config
  toggle is on.
- Existing `with_priority()`, `with_due_date()` etc. builder methods show the
  `let new_task = Task { field, ..self }; let new_raw = rebuild_raw(&new_task); Task::parse(&new_raw)`
  pattern that `normalize_append` should follow.

### Established Patterns
- All Task mutations in `todotxt-core` go through: mutate struct fields → `rebuild_raw()` →
  `Task::parse()` to sync raw. `normalize_append` follows this exact pattern.
- Config in TUI: struct fields on `Config` with serde defaults — new bool toggles follow
  the same shape.

### Integration Points
- `crates/todotxt-core/src/task.rs` (or `normalize.rs`): new `pub fn normalize_append`
- `crates/todotxt-tui/src/app.rs`: `handle_append_text_key` and `save_and_exit`
- TUI config struct: `normalize_append: bool` and `normalize_edit: bool` fields with defaults
- `crates/todotxt-cli/src/`: `append` subcommand handler — add `--normalize` flag

</code_context>

<specifics>
## Specific Ideas

- Config shape: two independent booleans — `normalize_append` and `normalize_edit` — both
  defaulting to `true`. User can set either to `false` in `config.toml` to opt out.
- CLI design: `--normalize` flag is explicit opt-in on the CLI (does NOT read config
  automatically) to preserve deterministic behavior for scripts and AI agent use.
- Priority replacement rule: last-writer-wins on append (appended priority replaces
  original). This lets the append bar double as a reprioritization path.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 21-smart-text-normalization*
*Context gathered: 2026-04-25*

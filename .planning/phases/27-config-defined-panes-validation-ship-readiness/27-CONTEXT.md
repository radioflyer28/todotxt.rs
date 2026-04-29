# Phase 27: Config-Defined Panes + Validation + Ship Readiness - Context

**Gathered:** 2026-04-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Three workstreams in the final v1.4 milestone phase:
1. **CFG-01/02/03** — Define `[[panes]]` config schema, deserialize on startup, initialize runtime pane state from config entries, with warn-and-skip fallback for invalid entries.
2. **PATH-01/02/03** — Add `--todo`/`--archive`/`--config` CLI flags to the TUI binary (via clap), with archive auto-defaulting and CLI-wins precedence.
3. **27-03** — Unit tests for config deserialization and path resolution, CHANGELOG.md + README.md updates, and version bump to v1.4.0 in all Cargo.toml files.

</domain>

<decisions>
## Implementation Decisions

### Config Schema for Panes (CFG-01, CFG-02, CFG-03)
- **D-01:** Config pane definitions use `[[panes]]` array of tables — standard TOML idiom for arrays of named items. Maps to `Vec<PaneConfig>` via serde. Example:
  ```toml
  [[panes]]
  label = "Work"
  filter = "project:work"
  sort = "priority"
  group = true

  [[panes]]
  label = "Today"
  filter = "due:today"
  sort = "due_date"
  group = false
  ```
- **D-02:** Each `[[panes]]` entry supports four fields: `label` (string), `filter` (string), `sort` (string enum), `group` (bool). All fields optional with sane defaults (empty label → "Pane N", empty filter, `file_order` sort, `false` group).
- **D-03:** The `sort` field is a snake_case string matching `SortOrder` enum variants: `file_order`, `priority`, `due_date`, `alphabetical`. Deserialized via `serde(rename_all = "snake_case")` or custom impl.
- **D-04:** Invalid `[[panes]]` entries (unknown sort value, type mismatch, etc.) cause the entry to be skipped with a warning logged. The rest of the pane list loads normally. The app does not fail to start (CFG-03 safe fallback).
- **D-05:** `TuiConfig` gains a `panes: Vec<PaneConfig>` field (default empty vec) with `#[serde(default)]` so existing configs without `[[panes]]` continue to deserialize without error.

### Runtime Pane Persistence
- **D-06:** Runtime pane changes (add, delete, modify sort/group/filter) are persisted back to `config.toml` on quit. Config panes are the startup blueprint AND the persisted state.
- **D-07:** Persist scope: pane list only — `label`, `filter`, `sort`, `group` for each pane. Active pane index and hidden flag are NOT persisted.
- **D-08:** Persist timing: on quit only (when user exits via `q` / Ctrl+C). Single write path, no intra-session writes.
- **D-09:** Persist mechanism: atomic rewrite of the entire `config.toml` using the existing `config.save()` tmp+rename pattern. Non-pane config fields are preserved by round-tripping through `TuiConfig` serialization. No new dependencies (no `toml_edit`).
- **D-10:** On each startup, the `[[panes]]` section is loaded as the initial pane list. Runtime add/delete/filter changes update in-memory state; on quit those are flushed back.

### CLI Path Override Flags (PATH-01, PATH-02, PATH-03)
- **D-11:** Add `clap` to `todotxt-tui` for argument parsing — consistent with `todotxt-cli` which already uses clap. Declarative `#[derive(Parser)]` struct.
- **D-12:** Three flags: `--todo` / `-t`, `--archive` / `-a`, `--config` / `-c`. All accept `PathBuf` values.
- **D-13:** Archive auto-default (PATH-02): When `--todo` is passed without `--archive`, the archive path defaults to `{todo_dir}/done.txt`. Example: `--todo /tmp/work.txt` → archive = `/tmp/done.txt`.
- **D-14:** Precedence (PATH-01/03): CLI flags override `config.toml` values. If `--todo /tmp/work.txt` is passed, it wins over `todo_file = "..."` in config. CLI always wins.
- **D-15:** `--config` override: if `--config path` is passed, load `TuiConfig` from that path instead of the platform-default path. This happens before all other config reads.

### Validation and Close-Out (27-03)
- **D-16:** Test coverage: unit tests for (a) `[[panes]]` config deserialization — valid entries, invalid sort value skipped with warn, empty `[[panes]]` section, missing section; (b) path resolution logic — `--todo` without `--archive` defaults correctly, CLI flag overrides config value.
- **D-17:** Docs: update `CHANGELOG.md` with v1.4 release notes, update `README.md` with `[[panes]]` config example and `--todo`/`--archive`/`--config` CLI flag documentation.
- **D-18:** Version bump: all three Cargo.toml files (`todotxt-core`, `todotxt-cli`, `todotxt-tui`) bumped to `v1.4.0`.

### Agent's Discretion
- Exact `PaneConfig` struct field names and their `Default` impl
- Whether `sort` field deserialization uses `serde(rename_all)` or a custom `FromStr` + `Deserialize` wrapper
- How warnings for invalid pane entries are surfaced (stderr on startup, or via the existing app-level error log)
- Exact clap `Parser` struct layout and whether the TUI gains a `--version` flag as a side effect
- Whether pane persistence on quit requires a new App method or extends the existing quit handler

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Config and State Foundation
- `.planning/phases/26-pane-management-quick-hide-show/26-CONTEXT.md` — Pane lifecycle hotkeys, label numbering, 0-pane validity; the runtime pane model Phase 27 persists
- `.planning/phases/25-per-pane-query-behavior/25-CONTEXT.md` — Per-pane filter/sort/group state that Phase 27 makes config-loadable
- `.planning/REQUIREMENTS.md` §CFG-01, §CFG-02, §CFG-03 — Config-defined pane requirements
- `.planning/REQUIREMENTS.md` §PATH-01, §PATH-02, §PATH-03 — CLI path override requirements

### Existing Code
- `crates/todotxt-tui/src/config.rs` — `TuiConfig` struct (gains `panes: Vec<PaneConfig>`), `config.save()` atomic write pattern, `TuiConfig::load()` and `resolve_path()` — Phase 27 adds `[[panes]]` deserialization and quit-time persist here
- `crates/todotxt-tui/src/state.rs` — `Pane::new(id, label)` constructor; `Pane` struct fields that config values map to
- `crates/todotxt-tui/src/main.rs` — Current entry point with no arg parsing; Phase 27 adds clap `Parser` struct at the top, overriding config paths before `TuiConfig::load()`
- `crates/todotxt-tui/src/app.rs` — App quit handler (the write-back trigger point)

### Prior Patterns
- `.planning/phases/22-keymap-help-parity/22-CONTEXT.md` — Configurable keymap pattern; `[[panes]]` follows the same serde + `#[serde(default)]` approach

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`config.save(path)`** — atomic tmp+rename write already implemented; Phase 27 calls this on quit with updated `panes` vec
- **`TuiConfig::load(path)`** — already handles missing-file-returns-default; `[[panes]]` absent = empty vec with `#[serde(default)]`
- **`Pane::new(id, label)`** — constructor to call when initializing runtime panes from config entries
- **`TuiConfig::resolve_path()`** — portable mode logic already handles config path resolution; `--config` flag feeds into this before the call

### Established Patterns
- **`#[serde(default)]`** on every optional config section — no breaking changes to existing config.toml files
- **Atomic write (tmp+rename)** in `config.save()` — Phase 27 reuses this for the quit-time persist
- **clap usage in `todotxt-cli`** — Phase 27 mirrors this for the TUI's new `Args` struct

### Integration Points
- **`main.rs`:** Parse `Args` with clap before config load; use `args.config` to override config path, `args.todo` / `args.archive` to override after load
- **App quit path:** After main event loop exits, serialize current `App.panes` back to `TuiConfig.panes` and call `config.save()`
- **`TuiConfig`:** Add `panes: Vec<PaneConfig>` field; `PaneConfig` is the serde-friendly struct (string sort, etc.) that maps to `Pane` at startup

</code_context>

<specifics>
## Specific Ideas

- Config panes are the full source of truth for startup AND persistence — user doesn't need to understand a separate "runtime state file"
- `--config` override must apply before any other config field is read (needed so `--todo` override can come from the CLI-loaded config, not the platform config)
- Invalid `[[panes]]` entries should be skipped silently enough not to alarm users but logged so power users can debug config issues

</specifics>

<deferred>
## Deferred Ideas

- Per-pane hotkeys for direct pane jump (e.g., `[[panes]] hotkey = "ctrl+1"`) — defer to v2 (PANE-07/08 territory)
- Persist active pane index and hidden state across restarts — out of scope for Phase 27
- Surgical config.toml editing (toml_edit) to preserve comments and formatting — out of scope; atomic full-rewrite is sufficient for v1.4

</deferred>

---

*Phase: 27-config-defined-panes-validation-ship-readiness*
*Context gathered: 2026-04-28*

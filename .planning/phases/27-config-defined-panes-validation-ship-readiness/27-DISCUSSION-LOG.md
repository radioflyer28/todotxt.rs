# Phase 27: Config-Defined Panes + Validation + Ship Readiness - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Session date:** 2026-04-28
**Participants:** User + GitHub Copilot

---

## Area 1: Config Schema for Panes

**Q: What TOML structure should pane definitions use?**
Options: `[[panes]]` array of tables / `[panes.work]` named subtables / inline array
→ **Selected: `[[panes]]` array of tables** — standard TOML idiom, maps cleanly to Vec<PaneConfig>

**Q: Which fields should each [[panes]] entry support?**
Options: label/filter/sort/group / label/filter/sort/group/hotkey / label and filter only
→ **Selected: label, filter, sort, group** — matches Pane struct fields exactly

**Q: How should the sort field value be expressed in TOML?**
Options: snake_case string enum / short aliases / agent's discretion
→ **Selected: snake_case string enum** (`file_order`, `priority`, `due_date`, `alphabetical`) — matches SortOrder variants

**Q: When a [[panes]] entry has an invalid field value, what should happen?**
Options: warn and skip / fail with error / silent fallback
→ **Selected: Warn and skip** — skip the invalid entry, log a warning, load the rest normally (CFG-03)

---

## Area 2: CLI Path Override Flags

**Q: How should the TUI handle CLI argument parsing?**
Options: Add clap (same as todotxt-cli) / parse std::env::args manually / agent's discretion
→ **Selected: Add clap** — consistent workspace dependency, declarative, handles --help

**Q: What should the three CLI override flags be named?**
Options: `--todo/--archive/--config` / `--todo-file/--archive-file/--config-file` / `--todo-path/...`
→ **Selected: `--todo`, `--archive`, `--config`** — concise, mirrors common todo.txt CLI conventions

**Q: Should the flags have single-character short aliases?**
Options: Yes -t/-a/-c / No long-only / agent's discretion
→ **Selected: Yes** — `-t` / `-a` / `-c`

**Q: When --todo is passed without --archive, what does the archive path default to?**
Options: same dir as --todo with `done.txt` / same dir with `archive.txt` / no auto-default
→ **Selected: Same directory as --todo, filename `done.txt`** — e.g., `--todo /tmp/work.txt` → archive = `/tmp/done.txt`

**Q: When both --todo flag and todo_file in config.toml are set, which wins?**
Options: CLI wins / config wins / error on conflict
→ **Selected: CLI wins** — flags override config values, standard unix convention

---

## Area 3: Config Pane Lifecycle and Runtime Interaction

**Q: What's the relationship between config.toml panes and runtime panes?**
Options: config = startup blueprint only / config panes are pinned / runtime panes are persisted
→ **Selected: Runtime panes are persisted** — save state to config.toml on quit

**Q: What pane state gets persisted to config.toml on quit?**
Options: pane list only / pane list + active pane index / full state (panes + active + hidden)
→ **Selected: Pane list only** — label, filter, sort, group per pane

**Q: When should pane state be written back to config.toml?**
Options: on quit only / after each pane change / agent's discretion
→ **Selected: On quit only** — single write path, consistent with existing atomic save pattern

**Q: How should the persist write work?**
Options: atomic rewrite of entire config.toml / surgical update of [[panes]] only / agent's discretion
→ **Selected: Atomic rewrite** — same tmp+rename pattern as existing `config.save()`, no new dependencies

---

## Area 4: Validation and Close-Out Scope

**Q: What level of test coverage is expected for plan 27-03?**
Options: unit tests for config deserialization and path resolution / manual only / unit tests + integration test
→ **Selected: Unit tests for config deserialization and path resolution logic**

**Q: What documentation is in scope for close-out?**
Options: CHANGELOG.md + README.md / CHANGELOG only / CHANGELOG + README + config.toml.example
→ **Selected: CHANGELOG.md + README.md** — config schema example and CLI flag documentation

**Q: Should Phase 27 close-out include a version bump to v1.4.0?**
Options: bump to v1.4.0 in all three crates / no version bump / agent's discretion
→ **Selected: Bump to v1.4.0** — all three Cargo.toml files (todotxt-core, todotxt-cli, todotxt-tui)

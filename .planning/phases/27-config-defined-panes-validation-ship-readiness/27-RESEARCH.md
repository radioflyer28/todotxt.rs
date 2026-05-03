# Phase 27: Config-Defined Panes + Validation + Ship Readiness - Research

**Date:** 2026-04-28
**Phase:** 27 - Config-defined panes, CLI path overrides, validation, release readiness

## RESEARCH COMPLETE

### Technical Analysis

#### Current State (from Phases 25-26)
- `Pane` runtime state already supports `label`, `filter_query`, `sort_order`, `grouping`.
- TUI startup currently loads config via `TuiConfig::resolve_path()` and `TuiConfig::load()` in `main.rs`.
- TUI currently requires `todo_file` from config and errors when missing.
- Config persistence already exists (`TuiConfig::save`) with atomic tmp+rename write.
- Quit flow is centralized in app runtime (`should_quit` + save/exit path), providing one place to persist pane state on exit.

#### Requirements-to-Implementation Mapping
1. **CFG-01/CFG-02**
- Add `panes: Vec<PaneConfig>` to `TuiConfig` with `#[serde(default)]`.
- Define serde-compatible `PaneConfig` with optional fields and defaults.
- Map startup panes from config into runtime `Pane` state before first render.

2. **CFG-03**
- Parse pane entries with tolerant behavior.
- Invalid entries must be skipped with warning; app startup must continue.
- Unknown/invalid `sort` values must not crash startup.

3. **PATH-01/PATH-02/PATH-03**
- Add clap parser in TUI binary for `--todo/-t`, `--archive/-a`, `--config/-c`.
- `--config` controls config load path first.
- CLI values override config values.
- If `--todo` present and `--archive` absent, default archive to `{todo_dir}/done.txt`.

4. **27-03 verification and close-out**
- Add focused tests for pane config deserialization and path resolution precedence/defaulting.
- Update release docs and bump all crate versions to 1.4.0.

#### Existing Patterns To Reuse
- Serde defaults pattern in `TuiConfig` (`#[serde(default)]`) for backward-compatible config evolution.
- Config warning aggregation pattern used by keymap resolution for non-fatal invalid config values.
- Atomic config save pattern (`save`) for reliable config persistence.
- Existing clap usage pattern from `todotxt-cli` for argument struct and parsing.

#### Risks and Mitigations
- **Risk:** Persisting runtime panes could overwrite unrelated config fields.
- **Mitigation:** Round-trip through `TuiConfig` and update only `panes` field before `save()`.

- **Risk:** Sort string parsing mismatch between config and runtime enum.
- **Mitigation:** Use explicit snake_case serde mapping and a deterministic conversion layer to `SortOrder`.

- **Risk:** Path precedence logic drift between startup and tests.
- **Mitigation:** Extract resolution logic into testable helper(s) with explicit unit coverage for precedence and defaults.

### Validation Architecture

**Dimension 1 (Requirements Coverage):**
Plans must explicitly cover CFG-01, CFG-02, CFG-03, PATH-01, PATH-02, PATH-03.

**Dimension 2 (Config Compatibility):**
- Config files without `[[panes]]` still deserialize.
- Empty `[[panes]]` section yields empty pane list.
- Optional pane fields fall back to defaults.

**Dimension 3 (Failure Safety):**
- Invalid pane entries are skipped and warnings are emitted.
- Startup continues with remaining valid panes.

**Dimension 4 (Path Precedence and Defaulting):**
- `--config` changes source config file.
- CLI overrides config for todo/archive.
- `--todo` without `--archive` defaults to sibling `done.txt`.

**Dimension 5 (Persistence Contract):**
- Startup loads panes from config.
- Runtime pane edits mutate in-memory pane model.
- Quit path writes pane state back via config save path.

**Dimension 6 (Documentation + Release Consistency):**
- README documents pane config and new CLI flags.
- CHANGELOG includes v1.4 release notes.
- All crate versions aligned to 1.4.0.

---

## Research Summary

Phase 27 can be implemented entirely with existing architecture: add config schema for panes, wire clap-based path overrides before config-dependent initialization, and persist pane state on quit through existing config save mechanics. Primary complexity is keeping tolerant config parsing and deterministic path precedence behavior, which should be validated with targeted unit tests.

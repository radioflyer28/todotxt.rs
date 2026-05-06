# Phase 43 — View State Persistence: Discussion Log

**Session date**: 2025-07-14  
**Skill**: gsd-discuss-phase  
**Areas discussed**: 4  

---

## Area 1: State file vs config.toml

**Gray area**: Should `tui-state.toml` extend `config.toml` (add new keys alongside `[[panes]]`) or be a completely separate file? And what happens to `persist_panes_on_quit()`?

**Q1**: Should `tui-state.toml` be a new separate file or extend `config.toml`?  
**A**: Separate file — clean separation, `config.toml` stays user-managed.

**Q2**: What should happen to `persist_panes_on_quit()`?  
**A**: Retire it — the new state file mechanism replaces it entirely. `config.toml` is never written at runtime.

**Q3**: Where does `tui-state.toml` live?  
**A**: Same directory as `config.toml`, resolved with the same portable-mode logic.

**Decision captured**: D-01, D-03, D-04

---

## Area 2: When to detect "interactively modified"

**Gray area**: How does the app know which panes were actually changed by the user during the session? Options considered: runtime dirty flag on every state change, compare-on-quit against startup snapshot, or always write all panes.

**Q1**: Runtime dirty flag or compare-on-quit?  
**A**: Compare-on-quit — no code changes needed to every state mutation; simpler.

**Q2**: What counts as a modification?  
**A**: Any state change, including preset application.

**Q3**: What about write failure?  
**A**: Silent no-op — atomic write makes failure rare; no status bar warning needed.

**Decision captured**: D-05, D-06

---

## Area 3: Startup merge model

**Gray area**: When both `config.toml` and `tui-state.toml` exist at startup, which wins? Options: full replacement (state file owns everything when present) or per-pane merge (state file patches individual panes, config.toml fills the rest).

**Q1**: Full replacement or per-pane merge?  
**A**: Full replacement — simpler, predictable. When `tui-state.toml` is present and valid, it defines all panes. `config.toml [[panes]]` is fallback only.

**Q2**: What if state file fails to parse?  
**A**: Silent fallback to `config.toml` — no error shown.

**Q3**: Unknown fields in state file?  
**A**: Silently ignored (permissive deserialization, consistent with PRSV-02 and existing `TuiConfig::load()` behavior).

**Decision captured**: D-07, D-08

---

## Area 4: App-level vs pane-level state

**Gray area**: `App` has top-level fields `sort_order`, `filter_query`, `grouping`, `group_by` in addition to per-pane copies in `Pane`. Do both need to be saved?

**Q1**: Are app-level fields independent state or derived copies?  
**A**: Derived copies — `rebuild_and_reanchor()` syncs them from the active pane before render. They are not an independent source of truth.

**Q2**: Does `active_pane: usize` need to be saved?  
**A**: No — session-specific navigation state, not persistent view configuration.

**Q3**: What exactly gets saved per pane?  
**A**: `filter_query`, `sort_order`, `grouping`, `group_by`, `label`. Reuse existing `PaneConfig` struct.

**Decision captured**: D-09

---

## Write scope clarification (post-discussion)

After all areas were discussed, user confirmed: **write ALL panes on exit** (not just modified ones). This is the correct interpretation of the full replacement model (D-07) — if we only wrote modified panes, startup would see a partial list. Writing all panes ensures the state file fully defines the pane list, and on restart the app reproduces all pane states exactly as they were.

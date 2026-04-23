---
phase: 18-validation-ship-readiness
date: 2026-04-23
status: pending
---

# Phase 18: TUI UAT Checklist

Build the TUI binary first: `cargo build -p todotxt-tui`
Run it against a test todo.txt file with a mix of tasks including at least:
- One task with `+project` tag
- One task with `@context` tag
- One task with `t:` threshold date in the future (e.g., `t:2026-12-31`)
- One completed task (`x `)
- One task with a priority (`(A)`)

Mark each item PASS or FAIL. Add notes for any FAIL.

---

## Area 1: Task Grouping (`g` key) [V12-TUI-GROUP-01]

| # | Step | Expected | Result |
|---|------|----------|--------|
| G1 | Press `g` once | Status bar shows `\| group: on`; tasks are separated by header rows (REVERSED style) showing shared key value | |
| G2 | With grouping on, use `j`/`k` to navigate | Cursor skips header rows and lands only on task rows | |
| G3 | With grouping on, press `x` on a task | Task is marked complete; header navigation is unaffected | |
| G4 | Press `g` again | Status bar `\| group: on` disappears; no header rows visible; task list returns to ungrouped | |
| G5 | Change sort order with `o`, then press `g` | Groups reflect the new sort key (e.g., sort by project → `+project` headers; sort by context → `@context` headers) | |

PASS criteria: All 5 items PASS.

---

## Area 2: Deferred Task Toggle (`h` key) [V12-TUI-DEFER-01, V12-TUI-DEFER-02]

| # | Step | Expected | Result |
|---|------|----------|--------|
| D1 | Start TUI — check if future-threshold task is visible | Task with `t:2026-12-31` is NOT shown in the list by default | |
| D2 | Press `h` | Status bar shows `[+deferred]`; the future-threshold task becomes visible, rendered with DIM styling | |
| D3 | Visually compare deferred task to active tasks | Deferred task appears visibly lighter/dimmer than surrounding active tasks | |
| D4 | Press `h` again | `[+deferred]` disappears from status bar; deferred task is hidden again | |
| D5 | With `h` on, navigate to the deferred task and press `Enter` (preview) | Preview opens showing the task's content including the `t:` tag | |

PASS criteria: All 5 items PASS.

---

## Area 3: Filter Esc / Restore [V12-TUI-FILTER-01]

| # | Step | Expected | Result |
|---|------|----------|--------|
| F1 | Press `/` to open quick filter, type some text | Filter bar shows typed text, task list filters live | |
| F2 | Press `Esc` while filter text is changed from prior value | Filter text reverts to what it was before `/` was pressed; task list restores | |
| F3 | Press `/`, type text, press `Enter` to confirm | Filter applies and stays; pressing `Esc` again after confirm does NOT clear the filter | |
| F4 | Open filter with active filter already set, change it, press `Esc` | Reverts to the previously confirmed filter value (not empty) | |

PASS criteria: All 4 items PASS.

---

## Area 4: Filter Persist / Reload [V12-TUI-FILTER-03]

| # | Step | Expected | Result |
|---|------|----------|--------|
| P1 | Open an F-key preset panel (e.g., press `F1` or use the preset UI) and configure a filter string | Preset is saved to TOML config (check `~/.config/todotxt-tui/config.toml` or portable equivalent) | |
| P2 | Quit the TUI completely (`q`) and relaunch | Previously configured preset is still present and functional | |
| P3 | Apply the preset and verify it filters tasks correctly | Task list filters as expected with no "empty filter" or crash on startup | |

PASS criteria: All 3 items PASS.

---

## Summary

| Area | Result |
|------|--------|
| G: Task Grouping | |
| D: Deferred Toggle | |
| F: Filter Esc/Restore | |
| P: Filter Persist | |

Overall: PASS / FAIL (circle one)
Notes:

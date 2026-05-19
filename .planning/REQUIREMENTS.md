# Requirements: todotxt.net

**Defined:** 2026-05-15
**Core Value:** A fast, cross-platform todo.txt tool with a first-class CLI for both human and AI agent use.

## v1.6.3 Requirements

Requirements for the first 1.6.3 cycle. Each maps to one roadmap phase.

### Recurring Tasks

- [ ] **REC-01**: User can include a `rec:` token on a task with documented interval syntax for strict (`+1d`) and relative (`1d`) recurrence modes.
- [ ] **REC-02**: Completing a recurring task automatically creates the next occurrence without prompting.
- [ ] **REC-03**: Completing a recurring task creates a new incomplete task with updated recurrence fields and next due date.
- [ ] **REC-04**: CLI `do` command and TUI completion paths both apply the same recurring completion behavior.

### Done Rotation

- [ ] **DONE-01**: `done.txt` rotates during archive writes when a new configured time period begins, with monthly cadence shipping first.
- [ ] **DONE-02**: Rotation preserves prior archived entries in deterministic period-named files and starts a fresh active `done.txt`.
- [ ] **DONE-03**: Users can configure archive rotation cadence with safe defaults, while retention cleanup remains out of scope for v1.6.3.

### Filter Language

- [ ] **FILT-01**: Filter supports OR logic within a token via `|` (for example `@work|@home`).
- [ ] **FILT-02**: OR terms can be combined with space-separated AND terms without breaking existing behavior.
- [ ] **FILT-03**: Negated OR terms are supported and documented.

### TUI Usability

- [ ] **TUI-01**: Inactive panes do not render cursor highlight while preserving each pane's last selected row.
- [ ] **TUI-02**: Grouped views render a spacer row before each non-first group header to improve scanability.

### Date Picker Ergonomics

- [x] **DATE-UX-01**: User can apply the date setter workflow to real date-bearing fields such as due, threshold, and completed dates without repeated command or navigation friction.
- [x] **DATE-UX-02**: Left and right arrow navigation in the date picker jumps by week to make calendar movement faster.

### Match Dialog Auto-Select

- [x] **AUTO-SEL-01**: Match-driven dialogs auto-select the most relevant existing project/context token or suggestion when presented to the user.
- [x] **AUTO-SEL-02**: Auto-select behavior prefers continuity by selecting the current or best matching token instead of resetting to an arbitrary suggestion.

## v2 Candidates

Deferred to future milestones:

### Advanced Recurrence

- **REC-05**: Recurrence customization UI and recurrence templates.
- **REC-06**: Recurrence completion templates per project/context.

### Archive Automation

- **DONE-04**: Automatic compression for rotated log files.
- **DONE-05**: Cross-machine migration tooling for rotated done logs.
- **DONE-06**: Automatic retention cleanup for rotated period archives.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Prompt before creating the next recurring occurrence in v1.6.3 | User clarified that creating the next occurrence is implicit when completing a recurring task. |
| Expand filter grammar beyond `|`/negation in v1.6.3 | Keeps scope tight and parser stability high. |
| Full UI date-picker redesign in v1.6.3 | Better deferred to a later usability milestone. |
| Threshold-based or retention-cleanup archive rotation in v1.6.3 | Phase 49 now targets time-based cadence rotation only, with cleanup deferred. |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| REC-01 | Phase 48 | Complete |
| REC-02 | Phase 48 | Complete |
| REC-03 | Phase 48 | Complete |
| REC-04 | Phase 48 | Complete |
| DONE-01 | Phase 49 | Complete |
| DONE-02 | Phase 49 | Complete |
| DONE-03 | Phase 49 | Complete |
| FILT-01 | Phase 46 | Complete |
| FILT-02 | Phase 46 | Complete |
| FILT-03 | Phase 46 | Complete |
| TUI-01 | Phase 47 | Complete |
| TUI-02 | Phase 47 | Complete |
| DATE-UX-01 | Phase 50 | Completed |
| DATE-UX-02 | Phase 50 | Completed |
| AUTO-SEL-01 | Phase 50 | Completed |
| AUTO-SEL-02 | Phase 50 | Completed |

**Coverage:**
- v1.6.3 requirements: 16 total
- Mapped to phases: 16
- Unmapped: 0 ✓

---
*Requirements defined: 2026-05-15*
*Last updated: 2026-05-19 after Phase 50 input ergonomics execution*

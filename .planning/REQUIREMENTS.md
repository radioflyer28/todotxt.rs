# Requirements: v1.2 Compatibility + UX Alignment

Status: ACTIVE
Started: 2026-04-23

## Scope

This milestone targets compatibility and UX alignment improvements after v1.1 TUI shipping.
Primary goals are todo.sh compatibility, parity-oriented TUI behavior polish, and reliable persisted filter UX.

## Requirements

- [ ] V12-COMPAT-01: todo.sh compatibility layer implemented for agreed command/alias surface
- [ ] V12-COMPAT-02: compatibility behavior validated with regression tests (arguments, output, exit codes)
- [ ] V12-TUI-FILTER-01: filter flow supports Esc cancel/restore behavior (no unintended destructive clear)
- [ ] V12-TUI-FILTER-02: filter definition layout aligned with todotxt.net UX model
- [ ] V12-TUI-FILTER-03: configured filters are persisted to TOML and restored on startup
- [ ] V12-TUI-STATUS-01: status bar theme label shown conditionally (omit when default/not useful)
- [ ] V12-TUI-GROUP-01: sorting/grouping behavior groups tasks sharing the same key values (project/context/etc) in parity with todotxt.net intent
- [ ] V12-TUI-DEFER-01: deferred-task parity decision documented (`t:` semantics)
- [ ] V12-TUI-DEFER-02: if parity confirmed, deferred-task behavior implemented and tested end-to-end

## Notes

- V12-TUI-DEFER-02 is contingent on V12-TUI-DEFER-01.
- GUI and CI/CD packaging remain out of scope for this milestone.

## Traceability Template

| Req ID | Description | Planned Phase | Status |
|--------|-------------|---------------|--------|
| V12-COMPAT-01 | todo.sh compatibility layer | 15 | Planned |
| V12-COMPAT-02 | Compatibility regression tests | 15/18 | Planned |
| V12-TUI-FILTER-01 | Esc cancel/restore behavior | 16 | Planned |
| V12-TUI-FILTER-02 | Filter layout alignment | 16 | Planned |
| V12-TUI-FILTER-03 | Persist filters to TOML | 16 | Planned |
| V12-TUI-STATUS-01 | Conditional theme label in status bar | 17 | Planned |
| V12-TUI-GROUP-01 | Group/sort parity behavior | 17 | Planned |
| V12-TUI-DEFER-01 | Deferred-task parity decision | 14 | Planned |
| V12-TUI-DEFER-02 | Deferred-task implementation (if confirmed) | 15/17 | Planned |

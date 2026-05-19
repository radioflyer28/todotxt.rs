# Research Summary: v1.6.3

## Stack additions

- Reuse existing Rust workspace and date/math utilities first.
- No new UI rendering dependency is required.
- Add only focused configuration fields for rotation behavior if needed.

## Feature table stakes

- Filter OR parsing in `todotxt-core` with explicit tests is the highest-priority correctness work.
- Recurring task generation should be confirmation-based in v1.6.3 to avoid surprise.
- TUI cursor/spacing changes are mostly renderer and focus-logic updates with regression risk on navigation.
- done.txt rotation is isolated to archive flow and should include retention configuration.

## Pitfalls to watch

- Parser precedence and negation behavior can create false positives.
- Recurrence completion paths must avoid duplicate task creation.
- Spacer rows must be excluded from cursor selection behavior.
- Archive rotation must remain safe under write failures.


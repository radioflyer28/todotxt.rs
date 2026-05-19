# Features Research

## Feature category: Filter language

### Table stakes
- Add OR operator support within a token using `|`.
- Preserve existing AND behavior when tokens are separated by spaces.
- Keep query parsing deterministic and composable.

### Differentiators
- Accept negated OR as an expected extension for common workflows (`-@work|@home`).
- Include parser tests for precedence and malformed-token resilience.

### Complexity notes
- Parser work should remain local to filter module and avoid changing CLI surface.

## Feature category: Recurring tasks

### Table stakes
- Parse and persist `rec:` tokens without dropping metadata.
- Create next occurrence after completion based on documented rules.
- Prompt before creating the next occurrence in v1.6.3 to avoid surprising behavior.

### Differentiators
- Keep recurrence generation path consistent across CLI and TUI completion flows.
- Preserve `due`, `t`, and metadata ordering in the generated follow-up task.

### Complexity notes
- Core parsing and recurrence arithmetic should live in `todotxt-core`.
- Completion hooks must remain idempotent under retries and partial failures.

## Feature category: UI usability

### Table stakes
- Hide active cursor styling in inactive panes.
- Insert spacer row between grouped sections.
- Ensure group and cursor logic remains accessible with existing key bindings.

### Differentiators
- Improve scanability with spacing patterns that scale to dense tasks.

### Complexity notes
- Most changes are in rendering/navigation paths, low risk but high regression sensitivity.

## Feature category: Archive hygiene

### Table stakes
- Rotate `done.txt` when threshold exceeded.
- Start a fresh active `done.txt` after rotation.
- Keep audit/undo semantics clear in docs.

### Differentiators
- Retention policy for rotations should be bounded.
- Use a predictable naming sequence for rotated files.

### Complexity notes
- Keep rotation logic near existing archive command boundaries.
- Avoid blocking done.txt reads during rotation.


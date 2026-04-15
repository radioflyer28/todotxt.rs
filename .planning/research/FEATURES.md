# Feature Landscape: todo.txt Rust CLI

**Domain:** todo.txt task manager — Rust core library + CLI
**Researched:** 2025-01-31
**Source:** C# reference implementation (todotxt.net), todo.txt spec, todo.sh ecosystem analysis, AI agent consumption patterns

---

## Table Stakes

Features users expect from any todo.txt CLI. Missing = product feels broken or incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| `add <text>` | Universal in every todo.txt tool | Low | Must handle relative dates (today, tomorrow, weekday names); auto-prepend creation date if configured |
| `list` / `ls` | Core read operation | Low | Returns all tasks; must show line numbers (task IDs) |
| `done <id>` / `do <id>` | Mark completion with today's date | Low | Strips priority per spec; sets `x YYYY-MM-DD` prefix |
| `undone <id>` | Reverse completion | Low | Strips `x DATE` prefix; priority NOT restored (spec-correct) |
| `del <id>` / `rm <id>` | Remove task permanently | Low | No confirmation prompt in non-interactive mode; must use ID not raw text |
| `edit <id> <text>` | Replace task raw text | Low | Replace entire raw line; ID-addressed |
| `append <id> <text>` | Append text to a task | Low | Appends to existing raw line |
| Filter by project `+Proj` | Core todo.txt usage pattern | Low | Substring match on raw text |
| Filter by context `@ctx` | Core todo.txt usage pattern | Low | Substring match on raw text |
| Filter by free text | Universal | Low | Case-insensitive by default |
| Negation filter `-term` | Expected by todo.sh users | Low | Prefix `-` excludes matching tasks |
| Sort by priority | Most common sort | Low | `(A)` < `(B)` < no-priority |
| Sort by due date | Second most common sort | Low | No-due-date tasks sort last |
| Sort by file order | Default (preserve insertion order) | Low | Should be the out-of-box default |
| Human-readable output | Default output | Low | Numbered list, optional color |
| JSON output | Essential for scripting/agents | Low | `--json` / `-j` global flag |
| `--file <path>` global flag | Multiple todo files | Low | Overrides default/config file path |
| Strict todo.txt format compliance | Interop with todo.sh ecosystem | Medium | Parser must match spec exactly; preserve `Raw` string to avoid whitespace drift |
| Preserve original line content | No accidental rewrites | Medium | C# reference: uses `Raw` field; Rust must do same |
| Line-ending preservation | Cross-platform files | Low | Detect CRLF vs LF; preserve on write |
| Config file | Persistent defaults | Medium | TOML at platform-appropriate path; portable mode override |
| Portable mode | No install, beside binary | Low | Config next to binary if present |

---

## Differentiators

Features that distinguish this tool from basic todo.sh clones. Especially: AI/agent-friendly patterns.

### AI Agent / Machine Consumption (Primary Differentiator)

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Structured JSON output (`--json`) | Agents can parse task lists without text parsing | Low | Every command outputs JSON when flag set; see schema below |
| Consistent exit codes | Agents can react to errors without parsing stderr | Low | 0=success, 1=not found, 2=IO error, 3=parse error; see table below |
| `--no-color` flag | Prevents ANSI escape codes corrupting agent parsing | Low | Honor NO_COLOR env var too |
| `--quiet` / `-q` flag | Suppresses confirmations and progress; stdout only on success | Low | Essential for piping |
| `stats` subcommand | Agents can query task counts without parsing list output | Low | Returns total/incomplete/overdue/due-today counts as JSON or human-readable |
| `contexts` subcommand | Agents can discover valid contexts without parsing tasks | Low | Returns sorted list of unique `@context` values |
| `projects` subcommand | Agents can discover valid projects without parsing tasks | Low | Returns sorted list of unique `+Project` values |
| ID-addressed operations | Agents address tasks by stable numeric ID (line number) | Low | All mutations take `<id>` not text search |
| `show <id>` subcommand | Fetch single task by ID with full JSON detail | Low | Returns parsed fields, not just raw text |
| Stdin task input | Pipe tasks from other tools | Low | `add --stdin` reads raw task text from stdin |

### Beyond todo.sh Baseline

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Due date filters (`due:today`, `due:past`, `due:future`, `due:active`) | Time-based queries without scripting | Low | All from C# reference; `due:active` = has a due date AND not future |
| Threshold date support (`t:YYYY-MM-DD`) | Hide future tasks automatically | Low | `--hide-future` flag; implemented in C# reference |
| Hidden task support (`h:1`) | Filter extension for deferred tasks | Low | `--show-hidden` flag to reveal |
| Relative date parsing | Human-friendly input | Low | `due:today`, `due:tomorrow`, `due:monday` etc. in `add` text |
| `archive` subcommand | Move completed tasks to done.txt | Low | Standard todo.sh pattern; needs `--archive-file` config |
| `delete-done` subcommand | Purge completed tasks without archive | Low | Destructive alternative to archive |
| Priority manipulation: `pri <id> <A-Z>`, `depri <id>` | Direct priority set/clear | Low | Separate from `edit` for agent ergonomics |
| `postpone <id> <N>` | Bump due date by N days | Low | Implemented in C# reference; agent-useful for scheduling |
| Multiple simultaneous filters (AND logic) | Narrow queries | Low | Multi-filter as multiple `--filter` args or newline-separated |
| `DONE` / `-DONE` filter keywords | Quick completion status filter | Low | From C# reference; case-sensitive as in reference |
| Case-insensitive filter option (`--case-sensitive`) | Flexible search | Low | Default: case-insensitive |
| Sort by: alphabetical, project, context, creation date, completion date | Comprehensive sort coverage | Low | All from C# reference SortType enum |

---

## CLI Command Surface Area

Complete subcommand definition for v1.0:

```
todo [GLOBAL_FLAGS] <SUBCOMMAND> [SUBCOMMAND_FLAGS]

GLOBAL FLAGS:
  -f, --file <PATH>       Path to todo.txt file [env: TODO_FILE]
  -j, --json              Output as JSON
  -q, --quiet             Suppress informational messages; only output result
      --no-color          Disable ANSI color codes [also: NO_COLOR env]
      --hide-future       Hide tasks with threshold date in the future
      --show-hidden       Show tasks tagged h:1

SUBCOMMANDS:
  add  <text>             Add a new task (relative dates resolved at add time)
  ls   [filter...]        List tasks (applies global hide-future/show-hidden)
  show <id>               Show a single task with all parsed fields
  do   <id>...            Complete one or more tasks
  undo <id>...            Uncomplete one or more tasks
  del  <id>...            Delete one or more tasks permanently
  edit <id> <text>        Replace task raw text entirely
  app  <id> <text>        Append text to task
  pri  <id> <A-Z>         Set priority on task
  depri <id>              Remove priority from task
  inc  <id>               Increase priority (A→ doesn't wrap below A)
  dec  <id>               Decrease priority
  due   <id> <YYYY-MM-DD> Set due date (accepts: today, tomorrow, weekday)
  due-rm <id>             Remove due date
  postpone <id> <N>       Postpone due date by N days (N can be negative)
  archive                 Move completed tasks to done.txt
  del-done                Delete all completed tasks (no archive)
  stats                   Show task counts (total/incomplete/overdue/due-today)
  projects                List all unique +Project tags
  contexts                List all unique @Context tags
  config                  Show/edit configuration values

FILTER SYNTAX (for `ls`):
  +Project                Include tasks with this project
  @Context                Include tasks with this context
  -+Project               Exclude tasks with this project
  DONE                    Only completed tasks
  -DONE                   Exclude completed tasks (default behavior)
  due:today               Tasks due today
  due:past                Tasks with past due date
  due:future              Tasks with future due date
  due:active              Tasks with due date not in the future
  -due:today              Exclude tasks due today
  <free text>             Include tasks containing this substring
  -<free text>            Exclude tasks containing this substring

  Multiple filters are AND-combined.
  Default: excludes completed, excludes h:1, excludes future-threshold tasks.
```

---

## JSON Output Schema

When `--json` is set, every command emits structured JSON to stdout. Agents rely on this contract being stable.

### Task Object
```json
{
  "id": 5,
  "raw": "(A) 2024-01-15 Buy milk +Groceries @errands due:2024-01-20",
  "completed": false,
  "completed_date": null,
  "priority": "A",
  "creation_date": "2024-01-15",
  "due_date": "2024-01-20",
  "threshold_date": null,
  "projects": ["+Groceries"],
  "contexts": ["@errands"],
  "body": "Buy milk",
  "is_due": "NotDue",
  "hidden": false
}
```

### `ls` response
```json
{
  "tasks": [ ...task objects... ],
  "total": 42,
  "filtered": 12
}
```

### `stats` response
```json
{
  "total": 42,
  "incomplete": 38,
  "completed": 4,
  "overdue": 3,
  "due_today": 2
}
```

### `projects` / `contexts` response
```json
{ "values": ["+Groceries", "+Work", "+Home"] }
```

### Error response
```json
{
  "error": "Task 99 not found",
  "code": 1
}
```

---

## Exit Codes

Consistent exit codes are critical for agent and shell scripting use:

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Task not found (bad ID) |
| 2 | I/O error (file unreadable/unwritable) |
| 3 | Parse error (malformed input) |
| 4 | Config error |
| 5 | Invalid arguments |

---

## Filter/Query Capabilities

The filtering system from the C# reference should map directly:

| Filter Keyword | Behavior | Source |
|---------------|----------|--------|
| `+Project` | Include tasks with project | C# reference |
| `@Context` | Include tasks with context | C# reference |
| `-term` | Exclude tasks containing term | C# reference |
| `DONE` | Only completed (case-sensitive) | C# reference |
| `-DONE` | Exclude completed (case-sensitive) | C# reference |
| `due:today` | Tasks where DueDate == today | C# reference |
| `due:past` | Tasks where DueDate < today | C# reference |
| `due:future` | Tasks where DueDate > today | C# reference |
| `due:active` | Has due date AND not in future | C# reference |
| `h:1` (via `--show-hidden`) | Hidden tasks | C# reference (extension) |
| Threshold (via `--hide-future`) | t: date in future | C# reference (extension) |

Multiple filters = AND logic. All filters from `ls` args, not from a separate config-read path.

---

## Anti-Features

Features to deliberately NOT build in v1.0.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Interactive prompts during mutations | Breaks agent automation; no way to pipe answers | Use `--force` flag or just skip confirmation in CLI (non-GUI mutations are user-initiated) |
| todo.sh compatibility layer (same command aliases, env vars) | Big surface area, not a stated goal; adds complexity without value yet | Deferred to seed planted in PROJECT.md |
| Color output by default in piped contexts | Breaks agent parsing | Auto-detect tty; disable color if not tty (respect NO_COLOR) |
| Task search by fuzzy text (fzf-style) | Requires TUI; not appropriate for CLI; adds large dependency | Defer to TUI milestone |
| Recurrence rules (`rec:` extension) | Not in reference implementation; complex edge cases | Could be added as extension later; not in scope |
| Collaborative/sync features | Different product entirely | Out of scope |
| Undo history / transaction log | Complex state management; file is the source of truth | If needed, deferred to later |
| Task import/export to other formats (iCal, CSV) | Low value in v1; not in reference | Deferred; file format is already portable text |
| REPL / interactive mode | That's the TUI milestone | Deferred to v1.1 TUI |
| Web API / daemon mode | Not what this tool is | Separate concern entirely |

---

## Feature Dependencies

```
config               → (no deps; first thing needed)
add                  → config (needs file path)
ls                   → add (needs tasks to list)
show                 → ls (needs tasks)
do / undo            → ls (needs task IDs)
del                  → ls (needs task IDs)
edit / app           → ls (needs task IDs)
pri / depri          → ls (needs task IDs)
due / due-rm         → ls (needs task IDs)
postpone             → due (same mechanism)
archive              → do (needs completed tasks; needs archive file config)
del-done             → do (needs completed tasks)
stats                → ls (aggregates over task list)
projects             → ls (extracts metadata)
contexts             → ls (extracts metadata)
JSON output          → all commands (cross-cutting flag, not a feature dep)
```

---

## MVP Recommendation

Prioritize in this order for the first shippable version:

**Phase 1 — Core reads (prove the parser and output contract):**
1. `ls` with all filter keywords — proves parser, filter engine, JSON output, exit codes
2. `stats` — proves metadata aggregation; immediately agent-useful
3. `projects` + `contexts` — proves metadata extraction

**Phase 2 — Writes (CRUD):**
4. `add` — prove write path, creation date, relative date parsing
5. `do` / `undo` — prove completion; needed for archive
6. `del` — prove deletion
7. `edit` + `app` — prove update path

**Phase 3 — Task enrichment:**
8. `pri` / `depri` / `inc` / `dec` — priority manipulation
9. `due` / `due-rm` / `postpone` — date manipulation

**Phase 4 — Bulk operations:**
10. `archive` — needs done.txt path config
11. `del-done` — destructive bulk

**Defer:**
- `config` subcommand (config file read/write works via file system first; UI for it is convenience)
- `show` (just `ls | jq` serves same need; add when agent feedback demands it)

---

## Sources

- C# reference implementation: `ToDoLib/Task.cs`, `ToDoLib/TaskList.cs`, `Client/MainWindowViewModel.cs` (filter logic, sort types, date manipulation)
- todo.txt spec: https://github.com/todotxt/todo.txt (format rules, field ordering)
- todo.sh CLI: https://github.com/todotxt/todo.txt-cli (command naming conventions)
- PROJECT.md: `.planning/PROJECT.md` (scope, constraints, out-of-scope items)

**Confidence:**
- Table stakes: HIGH (derived directly from reference implementation + established ecosystem patterns)
- JSON schema: HIGH (designed from first principles for agent use; schema fields derived from C# Task model)
- CLI surface area: HIGH (derived from C# ViewModel public methods + todo.sh conventions)
- Agent consumption patterns: MEDIUM (based on common LLM tool-use patterns; no single authoritative source)

# todotxt — A fast, cross-platform todo.txt CLI tool

A command-line task manager that reads and writes the [todo.txt format](https://github.com/todotxt/todo.txt).
Built in Rust. Works on Linux, macOS, and Windows.

**Audiences:** This documentation is written for both human users and AI agent consumers.
The [JSON Schema Documentation](#4-json-schema-documentation) section uses structured tables
for machine-parseable integration.

---

## Features

- **CLI** — 25+ commands: add, do, list, filter, sort, archive, bulk ops, JSON output, TOML config, shell completions
- **todo.sh compatible** — Drop-in alias support for `add`/`a`, `do`/`x`, `ls`, `lsa`, `lsp`, `rm` and more. Use `--compat` for numbered output.
- **TUI** — Keyboard-driven terminal UI: live filter, sort, grouping (`g`), deferred task toggle (`h`), persistent filter presets
- **Deferred tasks** — Tasks with a future `t:YYYY-MM-DD` threshold date are hidden by default; toggle with `h` in the TUI or `--all` in the CLI
- **todo.txt format** — Strict round-trip: priorities, projects (`+tag`), contexts (`@tag`), due dates (`due:`), threshold dates (`t:`), completion dates
- **Cross-platform** — Windows, Linux, macOS

---

## Table of Contents

1. [Installation](#1-installation)
2. [Quick Start](#2-quick-start)
3. [Full Command Reference](#3-full-command-reference)
4. [JSON Schema Documentation](#4-json-schema-documentation)
5. [Config File Format](#5-config-file-format)
6. [Shell Completion Instructions](#6-shell-completion-instructions)
7. [todo.txt Format Primer](#7-todotxt-format-primer)

---

## 1. Installation

### From source (requires Rust >= 1.75)

```sh
cargo install todotxt
```

### Pre-built binaries

Pre-built binaries for Linux, macOS, and Windows are coming in v1.1. See the
[Releases](../../releases) page.

### Post-install: shell completions

After installing, generate completion scripts for your shell — see
[Shell Completion Instructions](#6-shell-completion-instructions).

---

## 2. Quick Start

Five commands to get started:

```sh
# Add a task with a project and context tag
todotxt add "Buy groceries +shopping @errands"

# List all incomplete tasks
todotxt list

# Mark task 1 as complete
todotxt do 1

# Show completion stats
todotxt stats

# Move completed tasks to done.txt
todotxt archive
```

Each command exits `0` on success, `1` if a task ID is not found, and `2` on validation errors.

---

## 3. Full Command Reference

| Command | Aliases | Description | Key flags |
|---------|---------|-------------|-----------|
| `list [FILTERS]` | `ls` | List incomplete tasks, optionally filtered | `--json`, `--filter/-f` |
| `stats` | — | Show totals: complete, incomplete, overdue, due today | `--json` |
| `projects` | — | List all `+project` tags | `--json` |
| `contexts` | — | List all `@context` tags | `--json` |
| `show <ID>` | — | Show a single task by ID | `--json` |
| `add <TEXT>` | — | Add a new task | `--date`, `--no-date`, `--json` |
| `do <IDS>...` | — | Mark one or more tasks complete | `--json` |
| `undo <IDS>...` | — | Unmark completed tasks | `--json` |
| `del <ID>` | `rm` | Delete a task by ID | `--json` |
| `edit <ID> <TEXT>` | — | Replace task text | `--json` |
| `append <ID> <TEXT>` | — | Append text to a task | `--json` |
| `prepend <ID> <TEXT>` | — | Prepend text to a task | `--json` |
| `pri <PRIORITY> <IDS>...` | — | Set task priority (A-Z) | `--json` |
| `depri <IDS>...` | — | Remove priority from tasks | `--json` |
| `due <ID> <DATE>` | — | Set a due date (`today`, `tomorrow`, `YYYY-MM-DD`, weekday name) | `--json` |
| `postpone <ID> <DAYS>` | — | Advance due date by N days | `--json` |
| `archive` | — | Move completed tasks to `done.txt` | `--json` |
| `del-done` | — | Delete completed tasks from `todo.txt` | `--json` |
| `completions <SHELL>` | — | Output shell completion script | — |

### Global flags (available on all commands)

| Flag | Description |
|------|-------------|
| `--todo-file <PATH>` | Override the `todo.txt` path |
| `--config <PATH>` | Use a specific config file |
| `--json` | Output results as JSON envelope |
| `--no-color` | Disable ANSI color output |
| `--quiet` | Suppress non-error output |

---

## 4. JSON Schema Documentation

All commands accept a global `--json` flag. When set, output is a JSON envelope.

### Envelope format

| Field | Type | Always present | Description |
|-------|------|----------------|-------------|
| `schema_version` | `integer` | yes | Always `1`. Increment indicates breaking change. |
| `data` | `object` or `array` | on success | Command-specific payload. |
| `error` | `object` | on failure | Present instead of `data` when exit code is non-zero. |

**Success example (list):**
```json
{
  "schema_version": 1,
  "data": [
    {
      "id": 1,
      "raw": "(A) Buy groceries +shopping @errands",
      "priority": "A",
      "is_complete": false,
      "projects": ["shopping"],
      "contexts": ["errands"],
      "due_date": null
    }
  ]
}
```

**Error example:**
```json
{
  "schema_version": 1,
  "error": {
    "code": 1,
    "message": "task not found: 99"
  }
}
```

### Task object fields

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `id` | `integer` | 1-based line number in `todo.txt` | `1` |
| `raw` | `string` | Original unmodified task line | `"(A) Buy milk +groceries"` |
| `priority` | `string or null` | Single uppercase letter, or `null` | `"A"` |
| `is_complete` | `boolean` | `true` if task starts with `x ` | `false` |
| `creation_date` | `string or null` | ISO 8601 date (`YYYY-MM-DD`) or `null` | `"2026-01-15"` |
| `completion_date` | `string or null` | ISO 8601 date set when `do` is called, or `null` | `null` |
| `projects` | `array<string>` | All `+tag` values without the `+` | `["shopping"]` |
| `contexts` | `array<string>` | All `@tag` values without the `@` | `["errands"]` |
| `due_date` | `string or null` | `YYYY-MM-DD` from `due:` key-value tag, or `null` | `"2026-05-01"` |
| `tags` | `object` | All `key:value` pairs as a string map | `{"due": "2026-05-01"}` |

### Error object fields

| Field | Type | Description |
|-------|------|-------------|
| `code` | `integer` | `1` = not found / no match; `2` = validation error |
| `message` | `string` | Human-readable error description |

### Agent integration note

> **Parse `schema_version` first.** Reject responses where `schema_version != 1` to detect
> breaking changes before accessing `data`. This field is the stability contract between
> producers and AI agent consumers.

---

## 5. Config File Format

### Platform paths

| OS | Default config location |
|----|------------------------|
| Linux | `~/.config/todotxt/config.toml` |
| macOS | `~/.config/todotxt/config.toml` |
| Windows | `%APPDATA%\todotxt\config.toml` |

### Portable mode

Place `config.toml` in the same directory as the `todotxt` binary to activate portable mode.
When a sidecar `config.toml` is detected at runtime, it takes precedence over the platform path.
This makes the tool fully self-contained for USB/portable deployments.

### Full config example

```toml
[paths]
todo_file = "/home/user/todo.txt"   # Required: path to your todo.txt file
done_file = "/home/user/done.txt"   # Optional: defaults to done.txt beside todo.txt

[display]
color = true           # Default: true. Set false to disable ANSI colors.
preset = "default"     # Output preset: default | minimal | compact

[behavior]
auto_creation_date = false   # Prepend YYYY-MM-DD to new tasks automatically
```

---

## 6. Shell Completion Instructions

Generate and install completion scripts for your shell:

**Bash:**
```bash
todotxt completions bash >> ~/.bashrc && source ~/.bashrc
```

**Zsh:**
```zsh
todotxt completions zsh > "${fpath[1]}/_todotxt" && compinit
```

Or to a local completions dir:
```zsh
mkdir -p ~/.zsh/completions
todotxt completions zsh > ~/.zsh/completions/_todotxt
echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc && source ~/.zshrc
```

**Fish:**
```fish
todotxt completions fish > ~/.config/fish/completions/todotxt.fish
```

**PowerShell:**
```powershell
todotxt completions powershell >> $PROFILE
. $PROFILE
```

---

## 7. todo.txt Format Primer

todo.txt is a plain-text task format. Each line is one task. The format is:

```
(PRIORITY) CREATION-DATE task text +project @context key:value
```

All fields except the task text are optional and positional.

| Token | Meaning | Example |
|-------|---------|---------|
| `(A)` to `(Z)` | Priority — `(A)` is highest | `(A) Buy milk` |
| `YYYY-MM-DD` at start | Creation date | `2026-01-15 Buy milk` |
| `+word` | Project tag | `+groceries` |
| `@word` | Context tag | `@home` |
| `key:value` | Metadata tag | `due:2026-05-01` |
| `x ` prefix | Completed task | `x 2026-01-15 Buy milk` |

**Completed task format:**

```
x COMPLETION-DATE CREATION-DATE task text
```

**Example completed task:**
```
x 2026-04-10 2026-04-01 (A) Submit tax return +finance @office due:2026-04-15
```

**Reference:** [Official todo.txt format specification](https://github.com/todotxt/todo.txt)

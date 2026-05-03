---
status: deferred
planted_during: v1.0
trigger_when: CLI is stable; users want drop-in replacement for todo.sh shell scripts
---

# SEED-003: todo.sh Compatibility Layer

## Idea

Add a `todo.sh`-compatible command interface so users with existing scripts and workflows that call `todo.sh` can drop in the Rust CLI as a replacement without changing their scripts.

## Why This Matters

`todo.sh` is the canonical todo.txt CLI tool with a well-known command interface (`todo.sh add`, `todo.sh do`, `todo.sh list`, etc.). Many users have shell aliases, scripts, and integrations built around this interface. A compatibility layer enables zero-friction migration.

## When to Surface

- v1.0 Core + CLI milestone is complete
- Users report they'd use the tool more if it were a drop-in replacement for `todo.sh`
- CLI command design has stabilized

## Scope Ideas

- `todo.sh`-compatible subcommand aliases (`add`, `do`, `del`, `list`, `listcon`, `listproj`, `archive`, `pri`, `depri`, `append`, `prepend`, `replace`)
- Same positional argument format
- Environment variable support (`TODOTXT_CFG_FILE`, `TODO_DIR`, `TODO_FILE`, `DONE_FILE`)
- `todo.cfg` config file support
- Optional: addon/plugin system compatibility

# Phase 14: Compat Discovery + Spec Lock — Research

## Standard Stack

- **Language:** Rust (edition 2021)
- **Build:** `cargo build`
- **Test:** `cargo test`
- **Affected crates:** `todotxt-core` (filter.rs), `todotxt-cli` (cli.rs, commands/list.rs)
- **External reference:** [todo.sh v2.12+](https://github.com/todotxt/todo.txt-cli) — bash script, the canonical compatibility target

---

## Key Finding: t: Threshold Filtering Already Implemented

`crates/todotxt-core/src/filter.rs` has `suppress_future_threshold: true` as the **default** on `Filter` (lines 26–28, 35–37). Tasks with a future threshold date are already hidden from `list` output at the core library level.

**Edge case confirmed:**
- `t:PAST` (past date) → shown ✅
- `t:TODAY` (today) → shown ✅ (check is `t > today`, so equal dates pass through)
- `t:FUTURE` → hidden ✅ (filtered by `suppress_future_threshold`)

The `t:` filtering semantics match todo.sh convention **without any core library changes**.

**What is still missing for full D-04–D-08 parity:**

| Missing item | Phase | Files |
|---|---|---|
| `--all` flag on `list` (`ListArgs` in `cli.rs`) sets `suppress_future_threshold: false` | 15 | `crates/todotxt-cli/src/cli.rs`, `commands/list.rs` |
| TUI key toggle for deferred task visibility | 17 | `crates/todotxt-tui/src/` |
| Greyed-out styling for deferred tasks when shown | 17 | `crates/todotxt-tui/src/` (use Phase 13 `StyleSheet` system) |

---

## todo.sh Command Surface Audit

### Current CLI commands and existing aliases

From `crates/todotxt-cli/src/cli.rs`:

| Our command | Existing alias | Notes |
|---|---|---|
| `list` | `ls` ✅ | |
| `del` | `delete` | todo.sh uses `rm`; `delete` is not a todo.sh alias |
| `do` | — | todo.sh also accepts `done` |
| `depri` | — | todo.sh also uses `dp` |
| `pri` | — | todo.sh also uses `p` |
| `append` | — | todo.sh also uses `app` |
| `prepend` | — | todo.sh also uses `prep` |
| `contexts` | — | todo.sh uses `listcon`/`lsc` |
| `projects` | — | todo.sh uses `listproj`/`lsprj` |
| `add` | — | todo.sh uses `a` |

### Aliases to add (Phase 15 scope)

These are the Phase 15 compat alias additions to `cli.rs`:

| todo.sh name | todo.sh alias | Our command | Action |
|---|---|---|---|
| add | a | `add` | Add alias `a` |
| append | app | `append` | Add alias `app` |
| del | rm | `del` | Add alias `rm` (and keep `delete`) |
| depri | dp | `depri` | Add alias `dp` |
| do | done | `do` | Add alias `done` |
| prepend | prep | `prepend` | Add alias `prep` |
| pri | p | `pri` | Add alias `p` |
| contexts | lsc | `contexts` | Add alias `lsc` |
| projects | lsprj | `projects` | Add alias `lsprj` |

> **Note on `p` alias:** Clap allows single-char aliases. `p` does not conflict with any global flag since all global flags use `--long` form.

### New commands in scope (Phase 15 scope)

| todo.sh command | Alias | Behavior |
|---|---|---|
| `listpri` | `lsp` | List tasks filtered to PRIORITIES. `lsp A` → priority A only; `lsp A-C` → range A–C. Equivalent to `list` + priority regex filter, but with range parsing. |
| `listall` | `lsa` | List tasks from both `todo.txt` and `done.txt`, merged and sorted by priority. |
| `deduplicate` | — | Remove exact duplicate lines from `todo.txt`. Preserve line numbers (blank line replacement) per `TODOTXT_PRESERVE_LINE_NUMBERS` convention — simplify: always remove blank lines in our implementation. |

### Commands covered with different semantics (document divergence)

| todo.sh command | Our equivalent | Documented divergence |
|---|---|---|
| `replace NR "text"` | `edit NR text` | Different arg style (we don't use quoted full-line replacement); semantically equivalent. Document in compat notes. |
| `shorthelp` | `--help` / `-h` (clap built-in) | Clap provides `--help`; no `shorthelp` subcommand needed. |
| `help [ACTION]` | `--help` / subcommand `--help` | Clap provides per-subcommand help; no `help` subcommand needed. |

### Out-of-scope commands (document as intentional gaps)

| todo.sh command | Reason out of scope |
|---|---|
| `addm` | Multi-line batch add; no interactive use case in our CLI |
| `addto` | Add to arbitrary named file; not in our file model |
| `command` | Extension meta-action; not applicable without shell-script architecture |
| `listfile` / `lf` | List arbitrary file; not in our model |
| `move` / `mv` | Move tasks between files; out of scope for v1.2 |
| `report` | Generate report.txt; not in our model |

### Global flag mapping

| todo.sh flag | Our equivalent | Status |
|---|---|---|
| `-d CONFIG_FILE` | `--config` | ✅ covered |
| `-p` plain mode | `--no-color` | ✅ covered (inverse) |
| `-t` date on add | `add --date` | ✅ covered |
| `-T` no date on add | `add --no-date` | ✅ covered |
| `-v` verbose | inverse of `--quiet` | partial (we suppress, not expand) |
| `-@` hide contexts | no equivalent | intentional gap |
| `-+` hide projects | no equivalent | intentional gap |
| `-P` hide priorities | no equivalent | intentional gap |
| `-f` force | no equivalent | gap (we never prompt) |
| `-a`/`-A` auto-archive | not applicable | intentional gap |
| `-n`/`-N` line numbers | not applicable | intentional gap |
| `-x` disable filter | not applicable | intentional gap |

---

## Compat Mode Output (--compat flag)

Per D-09: a `--compat` flag on `list` (exact name TBD in Phase 15 — `--compat` is the working name) emits todo.sh-style `{N} {task}` numbered output. This is opt-in.

todo.sh `list` output format:
```
1 (A) Buy milk +groceries @home due:2026-04-25
2 Do taxes +finance
3 Call dentist @phone
```

Our current output is formatted via `comfy-table`. With `--compat`, bypass comfy-table and emit the raw `{line_number} {raw_task}` format.

---

## Pitfalls

- `p` as an alias for `pri`: valid for Clap, but short alias `p` could feel ambiguous to users. The alias is added for compat only — document it.
- `rm` alias for `del` on Windows: `rm` has no special meaning in PowerShell (it's `Remove-Item`), so this is safe cross-platform.
- `listpri` priority range parsing (`A-C`): todo.sh accepts `[A-Z]-[A-Z]` range or single `[A-Z]`. We need a small parser; this is not the same as our `Filter::from_query` token format.
- `listall` requires loading both `todo.txt` and `done.txt` — the `done.txt` path comes from `Config::done_file` (if set) or `{todo_dir}/done.txt` by convention.

---

## Validation Architecture

No automated tests for this phase — deliverables are spec documents. Verification is file existence + required section presence.

Downstream phases (15, 17) will add tests for each compat behavior as part of their implementation.

---

## Architecture Responsibility Map

| Component | Phase 14 role | Phase 15 action | Phase 17 action |
|---|---|---|---|
| `crates/todotxt-cli/src/cli.rs` | Audited (source of truth for current aliases) | Add aliases, `listpri`, `listall`, `deduplicate` commands, `--all` flag on `ListArgs` | — |
| `crates/todotxt-cli/src/commands/list.rs` | Audited | Wire `--all` to `suppress_future_threshold: false` | — |
| `crates/todotxt-cli/src/commands/` | Audited | New `listpri.rs`, `listall.rs`, `deduplicate.rs` | — |
| `crates/todotxt-core/src/filter.rs` | Already handles `suppress_future_threshold` | No changes needed | No changes needed |
| `crates/todotxt-tui/src/` | Not audited (TUI scope) | — | Add deferred toggle key + greyed styling |

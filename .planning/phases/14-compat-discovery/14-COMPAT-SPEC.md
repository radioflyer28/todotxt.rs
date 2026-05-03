# Phase 14: Compat Surface Spec
Status: LOCKED
Governs: Phase 15 implementation

---

## Aliases to Add

These 9 aliases are missing from our CLI and must be added to the `Commands` enum in `crates/todotxt-cli/src/cli.rs`. All are confirmed against the todo.sh v2.12 command surface.

| todo.sh command | todo.sh alias | Our command | Clap change |
|---|---|---|---|
| `add` | `a` | `add` | Add `"a"` to the `add` variant's `#[command(alias = ...)]` |
| `append` | `app` | `append` | Add `"app"` alias |
| `del` | `rm` | `del` | Add `"rm"` alias (keep existing `"delete"` alias too) |
| `depri` | `dp` | `depri` | Add `"dp"` alias |
| `do` | `done` | `do` | Add `"done"` alias |
| `prepend` | `prep` | `prepend` | Add `"prep"` alias |
| `pri` | `p` | `pri` | Add `"p"` alias (single-char aliases are valid in Clap 4) |
| `contexts` | `lsc` | `contexts` | Add `"lsc"` alias |
| `projects` | `lsprj` | `projects` | Add `"lsprj"` alias |

**Note on `p` alias:** Clap 4 supports single-character subcommand aliases. `p` does not conflict with any global flag since all global flags use `--long` form. This alias is added for todo.sh compat only — document in help text.

**Note on `rm` alias on Windows:** PowerShell uses `Remove-Item` (not `rm`) as an alias, so `rm` is safe to use as a subcommand alias cross-platform.

---

## New Commands to Implement

Three commands exist in todo.sh but have no equivalent in our CLI. All three are in scope for Phase 15.

| Command | Alias | Behavior spec |
|---|---|---|
| `listpri` | `lsp` | Accept optional `PRIORITIES` arg (single letter `A` or hyphen-delimited range `A-C`; default `A-Z`). Filter tasks to those matching the specified priority or range. Reuse `Filter::from_query` for term filtering, then post-filter on the task's `priority` field. Output in the same format as `list`. Priority range parser: if arg is a single `[A-Z]` letter, match that priority only; if arg is `[A-Z]-[A-Z]`, expand to the inclusive character range. |
| `listall` | `lsa` | Load `todo.txt` AND `done.txt` (path from `Config::done_file` if set, otherwise `{todo_dir}/done.txt`). Merge both lists, sort by priority descending (same sort order as `list`). Display the merged list with line numbers relative to each file (prefix completed tasks with a `done.txt` indicator or separate section — Phase 15 planner decides the exact display format). |
| `deduplicate` | — | Read `todo.txt`, identify exact duplicate raw lines (case-sensitive, byte-for-byte comparison). Remove the second (and subsequent) occurrence of each duplicate. **Simplification vs todo.sh:** do NOT replace duplicates with a blank line — physically remove the line and renumber. Print count of removed duplicates to stdout. Exit 0 whether or not duplicates are found. |

---

## Commands with Different Semantics (document only, no implementation change)

| todo.sh command | Our equivalent | Documented divergence |
|---|---|---|
| `replace NR "text"` | `edit NR text` | Different arg quoting style — todo.sh takes a full quoted replacement string; our `edit` takes unquoted tokens. Semantically equivalent (full line replacement). Document in `--help` output. |
| `shorthelp` | `--help` / `-h` (Clap built-in) | Clap provides per-command `--help`; no `shorthelp` subcommand is needed. |
| `help [ACTION]` | subcommand `--help` | Clap provides per-subcommand `--help`; no `help` subcommand is needed. |

---

## Intentional Gaps (out of scope for v1.2)

| todo.sh command | Reason out of scope |
|---|---|
| `addm` | Multi-line batch add — no interactive use case in our CLI design |
| `addto` | Add to an arbitrary named file — not in our single-file-per-list model |
| `command` | Extension meta-action — not applicable without a shell-script plugin architecture |
| `listfile` / `lf` | List an arbitrary file — not in our model |
| `move` / `mv` | Move tasks between files — out of scope for v1.2 |
| `report` | Generate report.txt — not in our model |

---

## Compat Output Mode (`--compat` flag on `list`)

**Decision D-09:** A `--compat` flag on the `list` command emits todo.sh-style numbered output. This is opt-in — not the default. The flag name `--compat` is locked for Phase 15 implementation.

**todo.sh output format:**
```
1 (A) Buy milk +groceries @home due:2026-04-25
2 Do taxes +finance
3 Call dentist @phone
```

**Implementation note for Phase 15:**
- When `--compat` is passed to `list`, bypass the `Renderer::print_tasks` / comfy-table path entirely.
- Use a simple loop:
  ```rust
  for (id, task) in tasks {
      println!("{} {}", id, task.to_raw());
  }
  ```
- `task.to_raw()` returns the raw todo.txt line (priority prefix, dates, text, tags) without any rendering decoration.
- No comfy-table column alignment, no color markup in compat mode output.

---

## Global Flag Mapping (reference only)

| todo.sh flag | Our equivalent | Status |
|---|---|---|
| `-d CONFIG_FILE` | `--config` | ✅ covered |
| `-p` plain mode | `--no-color` | ✅ covered (inverse semantics — we add color by default; `-p` disables it) |
| `-t` date on add | `add --date` | ✅ covered |
| `-T` no date on add | `add --no-date` | ✅ covered |
| `-v` verbose | inverse of `--quiet` | partial (we suppress output, not expand it) |
| `-@` hide contexts | no equivalent | intentional gap |
| `-+` hide projects | no equivalent | intentional gap |
| `-P` hide priorities | no equivalent | intentional gap |
| `-f` force | no equivalent | gap (we never prompt interactively, so force has no meaning) |
| `-a`/`-A` auto-archive | not applicable | intentional gap |
| `-n`/`-N` line numbers | not applicable | intentional gap |
| `-x` disable filter | not applicable | intentional gap |

---

## Files to Modify in Phase 15

These are the exact file paths Phase 15 executor must touch. No other files require modification for the compat surface.

| File | Change |
|---|---|
| `crates/todotxt-cli/src/cli.rs` | Add 9 aliases to existing Commands enum variants; add `listpri`, `listall`, `deduplicate` command variants; add `--all` flag to `ListArgs`; add `--compat` flag to `ListArgs` |
| `crates/todotxt-cli/src/commands/list.rs` | Wire `--all` → `filter.suppress_future_threshold = false` (and `suppress_hidden = false`); wire `--compat` → bypass comfy-table renderer and use raw numbered output loop |
| `crates/todotxt-cli/src/commands/listpri.rs` | New file — implement `listpri` command (priority range filtering) |
| `crates/todotxt-cli/src/commands/listall.rs` | New file — implement `listall` command (merged todo + done list) |
| `crates/todotxt-cli/src/commands/deduplicate.rs` | New file — implement `deduplicate` command (exact-match line deduplication) |
| `crates/todotxt-cli/src/commands/mod.rs` | Add `pub mod listpri;`, `pub mod listall;`, `pub mod deduplicate;` |

---

*Phase: 14-compat-discovery*
*Spec locked: 2026-04-23*

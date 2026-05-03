# 15-01 Summary — CLI Type Definitions

## Status: COMPLETE

## What was done

Added all CLI surface changes to `crates/todotxt-cli/src/cli.rs`:

### 9 compat aliases added
| Variant | Alias added |
|---------|------------|
| `Add` | `"a"` |
| `Append` | `"app"` |
| `Del` | `"rm"` (added alongside existing `"delete"`) |
| `Depri` | `"dp"` |
| `Do` | `"done"` |
| `Prepend` | `"prep"` |
| `Pri` | `"p"` |
| `Contexts` | `"lsc"` |
| `Projects` | `"lsprj"` |

### 3 new command variants added to `Commands` enum
- `Listpri(ListpriArgs)` — `#[command(name = "listpri", alias = "lsp")]`
- `Listall(ListArgs)` — `#[command(name = "listall", alias = "lsa")]`
- `Deduplicate` — `#[command(name = "deduplicate")]`

### New struct: `ListpriArgs`
- `pub priorities: Option<String>` — accepts `"A"` or `"A-C"` range format

### `ListArgs` new fields
- `pub all: bool` — `#[arg(long)]` — show deferred and hidden tasks
- `pub compat: bool` — `#[arg(long)]` — emit todo.sh-style numbered output

## Deviations
- None. Dispatch arms for new commands were added to `main.rs` in the same pass to allow `cargo check` to pass (otherwise non-exhaustive match error blocks compilation).

## Verification
- `cargo check` → exit 0 ✅

# todotxt-tui

A keyboard-driven terminal UI for [todo.txt](https://github.com/todotxt/todo.txt) files.
No database. No cloud. Just a plain text file — and a fast, full-featured TUI to manage it.

Built in Rust. Ships as a single static binary with no runtime dependencies.

---

## What it looks like

```
┌─ All tasks ─────────────────────────────────────────────────────────────────┐
│ ▶ (A) Fix login bug +api @backend due:2026-05-05                            │
│   (B) Write release notes +docs                                             │
│   Review PR #42 +api @backend                                               │
│   Buy groceries +home @errands                                              │
│   Call dentist @personal due:2026-05-10                                     │
└─────────────────────────────────────────── filter: @backend  sort: priority ┘
  a=add  d=delete  x=done  p=priority  Ctrl+F=filter  ?=help  q=quit
```

---

## Table of Contents

1. [Install](#1-install)
2. [First run](#2-first-run)
3. [Quick start walkthrough](#3-quick-start-walkthrough)
4. [All keybindings](#4-all-keybindings)
5. [Configuration](#5-configuration)
6. [Startup flags](#6-startup-flags)
7. [CLI companion](#7-cli-companion-todotxt)
8. [todo.txt format primer](#8-todotxt-format-primer)
9. [Building from source](#9-building-from-source)

---

## 1. Install

### Download a binary (no Rust required)

Go to the [Releases](../../releases) page and grab the file for your platform:

| Platform | File | Notes |
|----------|------|-------|
| Linux x86_64 | `todotxt-tui-linux-x86_64` | Fully static (musl); no libc dependency |
| macOS (Apple Silicon + Intel) | `todotxt-tui-macos-universal` | Universal binary; runs natively on arm64 and x86_64 |
| Windows x86_64 | `todotxt-tui-windows-x86_64.exe` | Static CRT; no VC++ redistributable required |

**Linux / macOS** — make it executable and put it on your PATH:

```sh
chmod +x todotxt-tui-linux-x86_64
sudo mv todotxt-tui-linux-x86_64 /usr/local/bin/todotxt-tui
```

**Windows** — move the `.exe` somewhere on your `PATH`, e.g.:

```powershell
Move-Item todotxt-tui-windows-x86_64.exe "$env:USERPROFILE\bin\todotxt-tui.exe"
```

---

## 2. First run

Just launch it:

```sh
todotxt-tui
```

On the very first run the TUI:

1. Creates `~/.todotxt.rs/config.toml` with sensible defaults
2. Points `todo_file` at `~/.todotxt.rs/todo.txt`
3. Opens with an empty task list — start adding with **`a`**

No files to create, no config to write. Just run it.

### Where your files live

| OS | Location |
|----|----------|
| Linux | `~/.todotxt.rs/` |
| macOS | `~/.todotxt.rs/` |
| Windows | `%USERPROFILE%\.todotxt.rs\` |

```
~/.todotxt.rs/
  config.toml        ← settings (auto-created on first run)
  todo.txt           ← your tasks
  done.txt           ← completed tasks (created when you first archive)
```

### Portable mode

Drop `config.toml` in the same directory as the binary to activate portable mode.
The TUI runs fully self-contained — `todo.txt` and `done.txt` also default to that
same directory. Nothing is read from or written to your home folder.
Useful for USB drives or shared machines.

---

## 3. Quick start walkthrough

### Step 1 — Add your first tasks

Press **`a`** to open the add dialog, type a task, press **`Enter`**:

```
Buy groceries +shopping @errands
(A) Fix login bug +api due:2026-05-05
Call dentist @personal t:2026-05-08
```

todo.txt format quick reference:

| Syntax | What it does |
|--------|-------------|
| `(A)` at the start | Sets priority — `(A)` highest, `(Z)` lowest |
| `+tag` | Adds a project tag |
| `@tag` | Adds a context tag |
| `due:YYYY-MM-DD` | Sets a due date |
| `t:YYYY-MM-DD` | Deferred threshold — task hidden until this date |

### Step 2 — Navigate and act on tasks

| Key | Action |
|-----|--------|
| `j` / `↓` | Move cursor down |
| `k` / `↑` | Move cursor up |
| `x` | Mark task complete (toggle) |
| `d` | Delete task (asks for confirmation) |
| `Enter` | Edit task text inline |
| `Ctrl+Z` | Undo the last change |

### Step 3 — Filter to focus

Press **`Ctrl+F`** (or **`/`**) to open the filter panel. Type any text, or use these patterns:

| Filter syntax | What it matches |
|---------------|----------------|
| `@errands` | Tasks with the `@errands` context |
| `@work` | All `@work*` tasks — prefix match, so `@work-remote` also matches |
| `+shopping` | Tasks with the `+shopping` project |
| `due:today` | Tasks due today |

Press **`0`** to clear the active filter.
Press **`F1`**–**`F9`** to jump to a saved filter preset (configured in `config.toml`).

### Step 4 — Set priorities and due dates

| Key | Action |
|-----|--------|
| `p` | Set priority (opens A–Z picker) |
| `u` | Remove priority |
| `D` | Set due date — type `today`, `tomorrow`, a weekday name, or `YYYY-MM-DD` |

### Step 5 — Archive completed tasks

When your done list piles up, run the companion CLI:

```sh
todotxt archive
```

This moves completed tasks from `todo.txt` into `done.txt`. Press **`.`** in the TUI
afterward to reload from disk.

---

## 4. All keybindings

### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move cursor down |
| `k` / `↑` | Move cursor up |
| `Tab` | Switch active pane (dual-pane mode) |
| `.` | Reload tasks from disk |

### Task actions

| Key | Action |
|-----|--------|
| `a` | Add new task |
| `Enter` | Edit selected task |
| `x` | Toggle complete |
| `d` | Delete task (with confirmation) |
| `p` | Set priority |
| `u` | Remove priority |
| `D` | Set due date |
| `Ctrl+Z` | Undo last mutation |

### Selection and bulk actions

| Key | Action |
|-----|--------|
| `v` | Enter / exit selection mode |
| `Space` | Toggle selection of current task |
| `D` *(in selection mode)* | Bulk delete selected tasks |
| `T` *(in selection mode)* | Bulk append text to selected tasks |

### View and filter

| Key | Action |
|-----|--------|
| `Ctrl+F` / `/` | Open filter panel |
| `0` | Clear active filter |
| `F1`–`F9` | Apply saved filter preset |
| `g` | Toggle task grouping |
| `s` | Cycle sort order |
| `h` | Toggle deferred task visibility |

### UI

| Key | Action |
|-----|--------|
| `?` | Open keybindings help overlay |
| `!` | Open error / warning log |
| `q` / `Ctrl+C` | Quit |

All bindings can be overridden in `config.toml` — see [Customizing keybindings](#customizing-keybindings).

---

## 5. Configuration

Config file location: see [Where your files live](#where-your-files-live) above.

### Minimal config

```toml
[paths]
todo_file = "~/.todotxt.rs/todo.txt"
```

### Full config reference

```toml
[paths]
todo_file = "~/.todotxt.rs/todo.txt"   # required
done_file = "~/.todotxt.rs/done.txt"   # optional; defaults to done.txt beside todo.txt

[display]
color = true            # set false to disable ANSI colors
preset = "default"      # default | minimal | compact

[behavior]
auto_creation_date = false   # prepend today's date to every new task

[tui]
theme = "default"       # "default" or "light"
```

### Saved filter presets

Assign filters to **`F1`**–**`F9`** for instant view switching:

```toml
[filters]
f1 = "@work"
f2 = "due:today"
f3 = "+personal"
```

### Dual-pane layout

Define named panes shown side-by-side with independent filters and sort orders:

```toml
[[panes]]
label = "Work"
filter = "+work"
sort = "priority"     # file_order | priority | due_date | alphabetical
group = true

[[panes]]
label = "Today"
filter = "due:today"
sort = "due_date"
group = false
```

Switch between panes with **`Tab`**.

### Customizing keybindings

Override any binding in `[keymap]`. Supported formats: single letters (`a`),
`ctrl+<letter>`, named keys (`backspace`, `enter`, `space`, `f1`–`f12`, `up`, `down`,
`left`, `right`, `esc`, `tab`).

```toml
[keymap]
quit       = "ctrl+q"
filter     = "/"
move_down  = "ctrl+n"
move_up    = "ctrl+p"
```

The TUI reports conflicts at startup if two actions share the same key.

---

## 6. Startup flags

```
todotxt-tui [OPTIONS]

Options:
  --todo <PATH>      Override the todo file for this run
  --archive <PATH>   Override the archive (done) file for this run
  --config <PATH>    Load configuration from a specific file
  --version          Print version and exit
  -h, --help         Print help
```

If `--todo` is given without `--archive`, the archive path defaults to `done.txt` in the
same directory as the selected todo file.

---

## 7. CLI companion (`todotxt`)

The `todotxt` binary ships alongside the TUI and covers scripting, shell pipelines,
and quick one-off operations. Both tools read the same `config.toml` and `todo.txt`.

```sh
todotxt add "Review PR #42 +api @backend"
todotxt list @backend
todotxt do 3
todotxt archive
todotxt stats
```

For the full CLI command reference, JSON output format, and shell completion setup,
see [README.cli.md](README.cli.md).

---

## 8. todo.txt format primer

Each line in `todo.txt` is one task:

```
(PRIORITY) CREATION-DATE task text +project @context key:value
```

| Token | Meaning | Example |
|-------|---------|---------|
| `(A)` – `(Z)` | Priority; `(A)` is highest | `(A) Fix the bug` |
| `YYYY-MM-DD` at start | Creation date | `2026-01-15 Write tests` |
| `+word` | Project tag | `+api` |
| `@word` | Context tag | `@backend` |
| `due:YYYY-MM-DD` | Due date | `due:2026-05-10` |
| `t:YYYY-MM-DD` | Threshold (deferred) date — hidden until this date | `t:2026-05-08` |
| `x ` prefix | Completed task | `x 2026-05-01 Buy milk` |

Full spec: [github.com/todotxt/todo.txt](https://github.com/todotxt/todo.txt)

---

## 9. Building from source

Requires Rust ≥ 1.75.

```sh
git clone https://github.com/radioflyer28/todotxt.rs
cd todotxt.rs
cargo build --release -p todotxt-tui    # TUI binary
cargo build --release -p todotxt        # CLI binary
```

Binaries land in `target/release/`.

---

## License

BSD — see [BSD_LICENSE.txt](BSD_LICENSE.txt).
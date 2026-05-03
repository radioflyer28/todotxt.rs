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
2. [Shell aliases](#2-shell-aliases)
3. [First run](#3-first-run)
4. [Quick start walkthrough](#4-quick-start-walkthrough)
5. [All keybindings](#5-all-keybindings)
6. [Configuration](#6-configuration)
7. [Startup flags](#7-startup-flags)
8. [CLI companion](#8-cli-companion-todotxt)
9. [todo.txt format primer](#9-todotxt-format-primer)
10. [Building from source](#10-building-from-source)

---

## 1. Install

### Using the install script (recommended)

**Linux / macOS:**

```sh
# TUI only (default)
curl -fsSL https://raw.githubusercontent.com/radioflyer28/todotxt.rs/master/scripts/install.sh | sh

# CLI only
curl -fsSL https://raw.githubusercontent.com/radioflyer28/todotxt.rs/master/scripts/install.sh | sh -s -- --cli

# Both TUI and CLI
curl -fsSL https://raw.githubusercontent.com/radioflyer28/todotxt.rs/master/scripts/install.sh | sh -s -- --both
```

**Windows (PowerShell):**

```powershell
# TUI only (default)
irm https://raw.githubusercontent.com/radioflyer28/todotxt.rs/master/scripts/install.ps1 | iex

# CLI only (save script first to pass flags)
irm https://raw.githubusercontent.com/radioflyer28/todotxt.rs/master/scripts/install.ps1 -OutFile install.ps1
.\install.ps1 --cli

# Both TUI and CLI (using env var — works with piped iex)
$env:INSTALL='both'; irm https://raw.githubusercontent.com/radioflyer28/todotxt.rs/master/scripts/install.ps1 | iex
```

All scripts download the correct binary for your platform, install to a directory on
your `PATH`, and print the alias setup snippet to add to your shell profile.

### Manual download

Go to the [Releases](../../releases) page and grab the files for your platform:

| Platform | Binary | File |
|----------|--------|------|
| Linux x86_64 | TUI | `todotxt-tui-linux-x86_64` (fully static, musl) |
| Linux x86_64 | CLI | `todotxt-linux-x86_64` (fully static, musl) |
| macOS (Apple Silicon + Intel) | TUI | `todotxt-tui-macos-universal` (universal binary) |
| macOS (Apple Silicon + Intel) | CLI | `todotxt-macos-universal` (universal binary) |
| Windows x86_64 | TUI | `todotxt-tui-windows-x86_64.exe` (static CRT) |
| Windows x86_64 | CLI | `todotxt-windows-x86_64.exe` (static CRT) |

**Linux / macOS:**

```sh
chmod +x todotxt-tui-linux-x86_64 todotxt-linux-x86_64
sudo mv todotxt-tui-linux-x86_64 /usr/local/bin/todotxt-tui
sudo mv todotxt-linux-x86_64 /usr/local/bin/todotxt
```

**Windows:**

```powershell
Move-Item todotxt-tui-windows-x86_64.exe "$env:USERPROFILE\bin\todotxt-tui.exe"
Move-Item todotxt-windows-x86_64.exe     "$env:USERPROFILE\bin\todotxt.exe"
```

---

## 2. Shell aliases

Typing `todotxt-tui` and `todotxt` every time is tedious. Set these up right after installing:

| Alias | Binary | Use it for |
|-------|--------|------------|
| `todo` | `todotxt-tui` | Launch the TUI |
| `td` | `todotxt` | CLI one-liners and scripts |

**Bash / Zsh** — add to `~/.bashrc` or `~/.zshrc`:

```sh
alias todo='todotxt-tui'
alias td='todotxt'
```

**Fish** — add to `~/.config/fish/config.fish`:

```fish
alias todo 'todotxt-tui'
alias td 'todotxt'
```

**PowerShell** — add to your `$PROFILE`:

```powershell
Set-Alias todo todotxt-tui
Set-Alias td   todotxt
```

The rest of this document uses `todo` and `td` in examples.

---

## 3. First run

Just launch it:

```sh
todo
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

## 4. Quick start walkthrough

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
Press **`1`**–**`9`** to jump to a saved filter preset (configured in `config.toml`).

### Step 4 — Set priorities and due dates

| Key | Action |
|-----|--------|
| `p` | Set priority (opens A–Z picker) |
| `u` | Remove priority |
| `D` | Set due date — type `today`, `tomorrow`, a weekday name, or `YYYY-MM-DD` |

### Step 5 — Archive completed tasks

When your done list piles up, run the CLI companion:

```sh
td archive
```

This moves completed tasks from `todo.txt` into `done.txt`. Press **`.`** in the TUI
afterward to reload from disk.

---

## 5. All keybindings

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
| `1`–`9` | Apply saved filter preset (F-key presets from config) |
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

## 6. Configuration

Config file location: see [Where your files live](#where-your-files-live) above.

### Minimal config (what auto-create produces)

```toml
todo_file = "/home/you/.todotxt.rs/todo.txt"
```

### Full config reference

```toml
# Top-level path and behaviour settings
todo_file = "~/.todotxt.rs/todo.txt"
done_file = "~/.todotxt.rs/done.txt"   # optional; defaults to done.txt beside todo.txt
auto_creation_date = false              # prepend today's date to every new task

[tui]
theme = "default"   # "default" (dark) or "light"

# Filter presets — press 1–9 in the TUI to activate.
# Keys must be "f1" through "f9".
[presets.f1]
filter = "@work"

[presets.f2]
filter = "due:today"

[presets.f3]
filter = "+personal"

[keymap]
# Override any TUI action. Supported formats: letter keys, ctrl+<letter>,
# named keys: backspace, enter, space, f1–f12, up, down, left, right, esc, tab.
quit       = "ctrl+q"
filter     = "/"
move_down  = "ctrl+n"
move_up    = "ctrl+p"

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

### Dual-pane layout

Each `[[panes]]` entry creates a named pane shown side-by-side with its own filter and sort.
Switch between panes with **`Tab`**.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `label` | string | `""` | Display name for the pane |
| `filter` | string | `""` | Initial filter query |
| `sort` | string | `file_order` | `file_order`, `priority`, `due_date`, or `alphabetical` |
| `group` | bool | `false` | Enable grouped rendering |

### Customizing keybindings

The TUI reports conflicts at startup if two actions share the same key.

```toml
[keymap]
quit       = "ctrl+q"
filter     = "/"
move_down  = "ctrl+n"
move_up    = "ctrl+p"
```

---

## 7. Startup flags

```
todo [OPTIONS]

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

## 8. CLI companion (`todotxt`)

The `td` alias runs `todotxt`, a full-featured CLI that reads the same `config.toml` and
`todo.txt` as the TUI. Useful for scripting, shell pipelines, and quick one-off operations.

```sh
td add "Review PR #42 +api @backend"
td list @backend
td do 3
td archive
td stats
```

For the full CLI command reference, JSON output format, and shell completion setup,
see [README.cli.md](README.cli.md).

---

## 9. todo.txt format primer

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

## 10. Building from source

Requires Rust ≥ 1.75.

```sh
git clone https://github.com/radioflyer28/todotxt.rs
cd todotxt.rs
cargo build --release -p todotxt-tui    # TUI binary → target/release/todotxt-tui
cargo build --release -p todotxt        # CLI binary → target/release/todotxt
```

---

## License

The Rust source code (`crates/`) is licensed under the
[Apache License 2.0](LICENSE-APACHE).

The original C# code (inherited from
[benrhughes/todotxt.net](https://github.com/benrhughes/todotxt.net)) is covered by the
[BSD 2-Clause License](BSD_LICENSE.txt), copyright 2011 Ben Hughes.
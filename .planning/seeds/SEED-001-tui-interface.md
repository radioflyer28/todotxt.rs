---
planted_during: v1.0
trigger_when: Core library and CLI are stable; users want an interactive terminal experience
---

# SEED-001: TUI Interface (ratatui)

## Idea

Add an interactive terminal UI to the Rust todo.txt app using `ratatui` (or similar). Gives terminal-native users a keyboard-driven, visual task management experience without needing a full desktop GUI.

## Why This Matters

Many developers and power users live in the terminal. A TUI provides the discoverability and interactivity of a GUI with the portability and speed of a CLI. The C# WPF app had a rich interactive UI — a TUI is the terminal-native equivalent for cross-platform users who prefer not to leave their terminal.

## When to Surface

- v1.0 Core + CLI milestone is complete
- Core library is stable and well-tested
- Users/feedback indicates demand for interactive terminal mode

## Scope Ideas

- Full-screen task list view with keyboard navigation
- Inline task editing
- Filter/sort panel
- Status bar (total, filtered, due today, overdue)
- @context and +project autocomplete
- Themeable colors

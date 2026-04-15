---
planted_during: v1.0
trigger_when: Core library and CLI are stable; need native desktop experience for non-terminal users
---

# SEED-002: Native GUI Interface

## Idea

Build a cross-platform native GUI for the Rust todo.txt app, providing feature parity with the original C#/WPF desktop application for users who prefer a graphical interface.

## Why This Matters

The original C# app was primarily a GUI app. Many non-developer users rely on a windowed interface with mouse support, menus, and dialogs. This milestone brings the full desktop experience to Linux and macOS users for the first time, and replaces the Windows-only WPF app.

## When to Surface

- v1.0 Core + CLI milestone is complete
- Core library is stable and well-tested
- Framework decision made (egui, iced, Tauri, or gtk-rs evaluated)

## Scope Ideas

- Task list with filtering, sorting, grouping
- Inline task editing with autocomplete (@context, +project, priorities)
- Dialogs: Add task, Edit task, Filter, Options
- System tray integration (Windows/Linux)
- File watching (reload on external change)
- Font/color customization
- Portable mode (settings beside binary)
- Platform builds: Windows (.exe + installer), Linux (AppImage/deb), macOS (.app)

## Framework Candidates

- `egui` (immediate mode, pure Rust, simple, good cross-platform)
- `iced` (Elm-architecture, reactive, good for complex UIs)
- `Tauri` (web frontend + Rust backend, HTML/CSS/JS UI)
- `gtk-rs` (GTK4 bindings, native Linux look-and-feel)

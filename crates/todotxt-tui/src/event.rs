//! Unified application event type.
//!
//! Both sender threads (crossterm input thread and FileWatcher callback)
//! send `AppEvent` values into the same `mpsc::Sender<AppEvent>`. The main
//! loop calls `recv()` — no polling, no timeout (D-02).

/// All events that can arrive in the main application loop.
#[derive(Debug)]
#[allow(dead_code)]
pub enum AppEvent {
    /// A key press or release event from crossterm.
    Key(crossterm::event::KeyEvent),
    /// Terminal window was resized to (columns, rows).
    Resize(u16, u16),
    /// The watched todo.txt file changed on disk (from FileWatcher callback).
    FileChanged,
    /// A non-fatal error from an event-source thread.
    Error(String),
}

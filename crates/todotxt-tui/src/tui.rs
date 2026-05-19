//! Terminal lifecycle management.
//!
//! `TerminalGuard` is an RAII struct: construction enters raw mode and the
//! alternate screen; `Drop` restores the terminal unconditionally. This ensures
//! cleanup on both normal exit (via `?`) and panics (via color-eyre's hook).

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};

/// Convenience alias for the terminal type used throughout the TUI crate.
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// RAII terminal guard. Entering alternate screen happens in `new()`;
/// leaving happens in `Drop`. Always use this; never call `enable_raw_mode`
/// directly outside this module.
pub struct TerminalGuard {
    pub terminal: Tui,
}

impl TerminalGuard {
    /// Enter raw mode and alternate screen, returning the guard.
    ///
    /// Call `color_eyre::install()` BEFORE calling `TerminalGuard::new()` so
    /// the panic hook is in place before any terminal state is altered.
    pub fn new() -> color_eyre::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(TerminalGuard { terminal })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Best-effort cleanup: ignore errors here so Drop never panics.
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
    }
}

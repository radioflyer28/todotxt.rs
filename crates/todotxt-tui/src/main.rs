#![deny(warnings)]

mod config;
mod tui;

use color_eyre::eyre::eyre;
use config::TuiConfig;

fn main() -> color_eyre::Result<()> {
    // D-08: Install color-eyre FIRST so panic hook restores terminal before printing.
    color_eyre::install()?;

    // Load config (D-07: portable mode path resolution).
    let platform_path = TuiConfig::default_path()
        .ok_or_else(|| eyre!("Cannot determine config directory"))?;
    let config_path = TuiConfig::resolve_path(&platform_path);
    let config = TuiConfig::load(&config_path)?;

    let todo_path = config
        .todo_file
        .ok_or_else(|| eyre!("todo_file is not set in config.toml ({}).\nHint: set todo_file = \"/path/to/todo.txt\"", config_path.display()))?;

    // Verify the file exists before entering the terminal.
    if !todo_path.exists() {
        return Err(eyre!("todo.txt not found at: {}", todo_path.display()));
    }

    // D-09: RAII terminal guard — Drop restores terminal on any exit path.
    let mut guard = tui::TerminalGuard::new()?;
    guard.terminal.clear()?;

    // Placeholder render to confirm the terminal guard works.
    guard.terminal.draw(|f| {
        use ratatui::widgets::Paragraph;
        let msg = Paragraph::new(format!("todotxt-tui: loading {}", todo_path.display()));
        f.render_widget(msg, f.area());
    })?;

    // Keep the screen visible briefly so it can be observed, then exit.
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Guard drops here, restoring terminal.
    Ok(())
}


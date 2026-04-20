//! TUI configuration loaded from the shared TOML config file.
//!
//! `TuiConfig` reads the same `config.toml` as `todotxt-cli`. Both crates
//! use serde to deserialize only the fields they know — unknown fields (like
//! the CLI's `[presets]` table) are silently ignored by each side.

use directories::ProjectDirs;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use todotxt_core::resolve_config_path;

/// Settings from the `[tui]` TOML subsection (D-04, D-05 in 13-CONTEXT.md).
///
/// A `[tui]` block is optional — `#[serde(default)]` on the field in `TuiConfig`
/// means existing configs without `[tui]` continue to work unchanged.
#[derive(Debug, Deserialize, Default)]
pub struct TuiSection {
    /// Theme name: `"default"` (dark) or `"light"`. Empty string → default theme.
    #[serde(default)]
    pub theme: String,
}

/// A named filter preset from the [presets] TOML section.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct TuiPreset {
    pub filter: Option<String>,
}

/// Phase 9 config fields. Mirrors the CLI's top-level TOML fields exactly.
/// A `[tui]` subsection will be added in Phase 13.
#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct TuiConfig {
    /// Path to the user's todo.txt file.
    pub todo_file: Option<PathBuf>,
    /// Path to the user's done.txt file.
    pub done_file: Option<PathBuf>,
    /// Automatically prepend today's date to new tasks.
    #[serde(default)]
    pub auto_creation_date: bool,
    /// Named filter presets. Keys are preset names (e.g. "work", "today").
    #[serde(default)]
    pub presets: HashMap<String, TuiPreset>,
    /// TUI-specific settings from the `[tui]` TOML subsection.
    #[serde(default)]
    pub tui: TuiSection,
}

impl TuiConfig {
    /// Returns the platform-appropriate config file path.
    ///
    /// - Linux:   ~/.config/todotxt/config.toml
    /// - Windows: %APPDATA%\todotxt\config.toml
    /// - macOS:   ~/Library/Application Support/todotxt/config.toml
    pub fn default_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "todotxt")
            .map(|dirs| dirs.config_dir().join("config.toml"))
    }

    /// Resolves the config path, applying portable mode:
    /// if `config.toml` exists beside the binary, use that path instead.
    /// Mirrors `Config::resolve_path` in `todotxt-cli` exactly.
    pub fn resolve_path(platform_path: &Path) -> PathBuf {
        let binary_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        let config_dir = platform_path
            .parent()
            .expect("platform config path must have a parent directory");
        resolve_config_path(&binary_dir, config_dir).join("config.toml")
    }

    /// Load config from `path`. Returns defaults silently if the file does
    /// not exist (first-run UX: TUI starts without requiring a config).
    ///
    /// # Errors
    /// Returns Err if the file exists but contains malformed TOML.
    pub fn load(path: &Path) -> color_eyre::Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)
                .map_err(|e| color_eyre::eyre::eyre!("reading config {}: {}", path.display(), e))?;
            toml::from_str(&content)
                .map_err(|e| color_eyre::eyre::eyre!("parsing config {}: {}", path.display(), e))
        } else {
            Ok(TuiConfig::default())
        }
    }
}

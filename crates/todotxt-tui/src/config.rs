//! TUI configuration loaded from the shared TOML config file.
//!
//! `TuiConfig` reads the same `config.toml` as `todotxt-cli`. Both crates
//! use serde to deserialize only the fields they know — unknown fields (like
//! the CLI's `[presets]` table) are silently ignored by each side.

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use todotxt_core::resolve_config_path;

/// Serde helper: returns `true` as the default value for normalization toggles.
/// Required because `#[serde(default)]` alone defaults to `false` for bool.
fn default_true() -> bool {
    true
}

/// Settings from the `[tui]` TOML subsection (D-04, D-05 in 13-CONTEXT.md).
///
/// A `[tui]` block is optional — `#[serde(default)]` on the field in `TuiConfig`
/// means existing configs without `[tui]` continue to work unchanged.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct TuiSection {
    /// Theme name: `"default"` (dark) or `"light"`. Empty string → default theme.
    #[serde(default)]
    pub theme: String,
}

/// A named filter preset from the [presets] TOML section.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TuiPreset {
    pub filter: Option<String>,
}

/// Phase 9 config fields. Mirrors the CLI's top-level TOML fields exactly.
/// A `[tui]` subsection will be added in Phase 13.
#[derive(Debug, Deserialize, Serialize, Default)]
#[allow(dead_code)]
pub struct TuiConfig {
    /// Path to the user's todo.txt file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub todo_file: Option<PathBuf>,
    /// Path to the user's done.txt file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done_file: Option<PathBuf>,
    /// Automatically prepend today's date to new tasks.
    #[serde(default)]
    pub auto_creation_date: bool,
    /// Normalize token placement when appending text to a task (D-07 in 21-CONTEXT.md).
    /// When true (default), appended priority/project/context/date tokens are merged
    /// into the task's canonical fields instead of raw string concat.
    #[serde(default = "default_true")]
    pub normalize_append: bool,
    /// Normalize token placement when saving an edited task (D-06 in 21-CONTEXT.md).
    /// When true (default), inline priority tokens and metadata in the edited line are
    /// lifted to canonical field positions on save.
    #[serde(default = "default_true")]
    pub normalize_edit: bool,
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
        ProjectDirs::from("", "", "todotxt").map(|dirs| {
            let config_dir = dirs.config_dir();
            // Normalize Windows `%APPDATA%\todotxt\config` to `%APPDATA%\todotxt\config.toml`.
            if config_dir.file_name().map(|n| n == "config").unwrap_or(false) {
                config_dir
                    .parent()
                    .unwrap_or(config_dir)
                    .join("config.toml")
            } else {
                config_dir.join("config.toml")
            }
        })
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

    /// Serialize `self` to TOML and write it atomically to `path`.
    ///
    /// Uses a temp file + rename to prevent partial writes from corrupting
    /// the config file if the process is interrupted (T-16-02-01).
    ///
    /// # Errors
    /// Returns Err if serialization fails or the write/rename fails.
    #[allow(dead_code)] // called by Plan 16-03 handle_filter_defining_key
    pub fn save(&self, path: &Path) -> color_eyre::Result<()> {
        let content = toml::to_string(self)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to serialize config: {e}"))?;
        let tmp_path = path.with_extension("toml.tmp");
        std::fs::write(&tmp_path, &content)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to write config tmp {}: {e}", tmp_path.display()))?;
        std::fs::rename(&tmp_path, path)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to rename config tmp to {}: {e}", path.display()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_normalize_flags_false() {
        // Verify that normalize_append and normalize_edit can be deserialized from TOML
        // and that false values are correctly parsed (T-21-04 mitigation).
        let toml_str = r#"
normalize_append = false
normalize_edit = false
"#;
        let config: TuiConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
        assert_eq!(config.normalize_append, false);
        assert_eq!(config.normalize_edit, false);
    }

    #[test]
    fn deserialize_normalize_flags_true() {
        // Verify that normalize_append and normalize_edit can be deserialized as true.
        let toml_str = r#"
normalize_append = true
normalize_edit = true
"#;
        let config: TuiConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
        assert_eq!(config.normalize_append, true);
        assert_eq!(config.normalize_edit, true);
    }

    #[test]
    fn deserialize_normalize_flags_default() {
        // Verify that when normalize_append and normalize_edit are not specified in TOML,
        // they default to true.
        let toml_str = r#"
auto_creation_date = false
"#;
        let config: TuiConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
        assert_eq!(config.normalize_append, true, "normalize_append should default to true");
        assert_eq!(config.normalize_edit, true, "normalize_edit should default to true");
    }
}

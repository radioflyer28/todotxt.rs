use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use todotxt_core::{resolve_config_path, ArchiveRotationCadence};

/// Named filter preset stored in config under `[presets.name]`.
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct PresetConfig {
    pub filter: Option<String>,
}

/// Application configuration loaded from TOML.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    /// Path to the user's todo.txt file. Required after first run.
    pub todo_file: Option<PathBuf>,
    /// Automatically prepend today's date to new tasks added with `add`.
    #[serde(default)]
    pub auto_creation_date: bool,
    /// Path to the user's done.txt file (defaults to sibling of todo.txt).
    #[serde(default)]
    pub done_file: Option<PathBuf>,
    /// Time-based rotation cadence for the active done.txt archive.
    #[serde(default)]
    pub archive_rotation_cadence: ArchiveRotationCadence,
    /// Named filter presets. Max 9 per CFG-02.
    #[serde(default)]
    pub presets: HashMap<String, PresetConfig>,
}

impl Config {
    /// Returns the unified config file path: `~/.todotxt.rs/config.toml` on all platforms.
    ///
    /// All three files (`config.toml`, `todo.txt`, `done.txt`) live together in
    /// `~/.todotxt.rs/` by default, making the setup self-contained and predictable
    /// regardless of OS.
    pub fn default_path() -> Option<PathBuf> {
        dirs_home().map(|home| home.join(".todotxt.rs").join("config.toml"))
    }

    /// Resolves the config path, applying portable mode:
    /// if config.toml exists beside the binary, use that path instead.
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

    /// Load config from `path`. If the file does not exist, auto-create it
    /// with defaults (todo_file = ~/todo.txt) and return the default config.
    ///
    /// # Errors
    /// Returns Err if the file exists but is malformed TOML, or if filesystem
    /// operations fail.
    pub fn load_or_create(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("reading config: {}", path.display()))?;
            toml::from_str(&content).with_context(|| format!("parsing config: {}", path.display()))
        } else {
            // Auto-create with default todo_file beside config.toml (D-01, D-02).
            // Using the config dir (not a hardcoded home path) means portable mode
            // automatically co-locates todo.txt beside the config and binary.
            let config_dir = path.parent().map(|p| p.to_path_buf());
            let home_todo = config_dir
                .as_ref()
                .map(|d| d.join("todo.txt"))
                .or_else(|| dirs_home().map(|h| h.join(".todotxt.rs").join("todo.txt")));
            let default = Config {
                todo_file: home_todo,
                auto_creation_date: false,
                done_file: None,
                archive_rotation_cadence: ArchiveRotationCadence::Monthly,
                presets: HashMap::new(),
            };
            let toml_str =
                toml::to_string_pretty(&default).context("serializing default config")?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent) // P-08: always create dirs first
                    .with_context(|| format!("creating config dir: {}", parent.display()))?;
            }
            std::fs::write(path, &toml_str)
                .with_context(|| format!("writing default config: {}", path.display()))?;
            Ok(default)
        }
    }

    /// Returns the resolved todo.txt path, or an error if not configured (D-03).
    pub fn resolve_todo_file(&self) -> Result<PathBuf> {
        self.todo_file.clone().ok_or_else(|| {
            anyhow!(
                "todo_file not set in config. \
                 Add `todo_file = \"/path/to/todo.txt\"` to your config or use --todo-file"
            )
        })
    }
}

fn dirs_home() -> Option<PathBuf> {
    directories::BaseDirs::new().map(|b| b.home_dir().to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn load_or_create_auto_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("sub").join("config.toml");
        let config = Config::load_or_create(&config_path).unwrap();
        assert!(config_path.exists(), "config file should be auto-created");
        // Auto-created config should have todo_file set (or None if no home dir)
        let _ = config.todo_file; // just verify it compiled
    }

    #[test]
    fn load_or_create_reads_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "todo_file = \"/home/user/todo.txt\"\n").unwrap();
        let config = Config::load_or_create(&config_path).unwrap();
        assert_eq!(config.todo_file, Some(PathBuf::from("/home/user/todo.txt")));
    }

    #[test]
    fn load_or_create_returns_err_on_malformed_toml() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "not valid toml ][[[").unwrap();
        let result = Config::load_or_create(&config_path);
        assert!(result.is_err(), "malformed TOML should return Err");
    }

    #[test]
    fn resolve_todo_file_returns_err_when_none() {
        let config = Config {
            todo_file: None,
            auto_creation_date: false,
            done_file: None,
            archive_rotation_cadence: ArchiveRotationCadence::Monthly,
            presets: HashMap::new(),
        };
        assert!(config.resolve_todo_file().is_err());
    }

    #[test]
    fn resolve_todo_file_returns_path_when_set() {
        let config = Config {
            todo_file: Some(PathBuf::from("/home/user/todo.txt")),
            auto_creation_date: false,
            done_file: None,
            archive_rotation_cadence: ArchiveRotationCadence::Monthly,
            presets: HashMap::new(),
        };
        assert_eq!(
            config.resolve_todo_file().unwrap(),
            PathBuf::from("/home/user/todo.txt")
        );
    }

    #[test]
    fn presets_can_be_loaded_from_toml() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(
            &config_path,
            "todo_file = \"/home/user/todo.txt\"\n\
             [presets.work]\n\
             filter = \"+work\"\n",
        )
        .unwrap();
        let config = Config::load_or_create(&config_path).unwrap();
        let preset = config.presets.get("work").unwrap();
        assert_eq!(preset.filter.as_deref(), Some("+work"));
    }

    #[test]
    fn archive_rotation_cadence_defaults_to_monthly() {
        let config: Config = toml::from_str("todo_file = \"/home/user/todo.txt\"\n").unwrap();
        assert_eq!(
            config.archive_rotation_cadence,
            ArchiveRotationCadence::Monthly
        );
    }

    #[test]
    fn archive_rotation_cadence_deserializes() {
        let config: Config = toml::from_str(
            "todo_file = \"/home/user/todo.txt\"\narchive_rotation_cadence = \"monthly\"\n",
        )
        .unwrap();
        assert_eq!(
            config.archive_rotation_cadence,
            ArchiveRotationCadence::Monthly
        );
    }
}

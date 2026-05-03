//! TUI configuration loaded from the shared TOML config file.
//!
//! `TuiConfig` reads the same `config.toml` as `todotxt-cli`. Both crates
//! use serde to deserialize only the fields they know — unknown fields (like
//! the CLI's `[presets]` table) are silently ignored by each side.

use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use todotxt_core::SortOrder;
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

/// Persisted sort options for config-defined panes.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PaneSort {
    Priority,
    DueDate,
    Alphabetical,
    #[default]
    FileOrder,
}

impl PaneSort {
    pub fn to_sort_order(self) -> SortOrder {
        match self {
            PaneSort::Priority => SortOrder::Priority,
            PaneSort::DueDate => SortOrder::DueDate,
            PaneSort::Alphabetical => SortOrder::Alphabetical,
            PaneSort::FileOrder => SortOrder::FileOrder,
        }
    }

    #[allow(dead_code)]
    pub fn from_sort_order(sort: SortOrder) -> Self {
        match sort {
            SortOrder::Priority => PaneSort::Priority,
            SortOrder::DueDate => PaneSort::DueDate,
            SortOrder::Alphabetical => PaneSort::Alphabetical,
            _ => PaneSort::FileOrder,
        }
    }
}

/// Persisted pane blueprint loaded from [[panes]] in config.toml.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct PaneConfig {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub filter: String,
    #[serde(default)]
    pub sort: PaneSort,
    #[serde(default)]
    pub group: bool,
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
    /// User-defined key binding overrides from the `[keymap]` TOML section (D-01, Phase 22).
    /// Keys are action names (e.g. "delete"), values are chord strings (e.g. "backspace").
    /// Configs without a `[keymap]` section deserialize to an empty map.
    #[serde(default)]
    pub keymap: HashMap<String, String>,
    /// Config-defined pane blueprints loaded at startup and persisted on quit.
    #[serde(default)]
    pub panes: Vec<PaneConfig>,
}

/// CLI-provided startup path overrides.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CliPathOverrides {
    pub todo: Option<PathBuf>,
    pub archive: Option<PathBuf>,
}

/// Final startup paths after applying config + CLI precedence rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupPaths {
    pub todo_path: PathBuf,
    pub archive_path: PathBuf,
}

fn default_archive_for_todo(todo_path: &Path) -> PathBuf {
    todo_path
        .parent()
        .map(|parent| parent.join("done.txt"))
        .unwrap_or_else(|| PathBuf::from("done.txt"))
}

/// Resolve startup todo/archive paths with deterministic precedence.
///
/// Precedence:
/// - `--todo` overrides `todo_file`
/// - `--archive` overrides `done_file`
/// - When `--todo` is set and `--archive` is omitted, archive defaults to
///   `{todo_dir}/done.txt`
pub fn resolve_startup_paths(config: &TuiConfig, overrides: &CliPathOverrides) -> color_eyre::Result<StartupPaths> {
    let todo_path = overrides.todo.clone().or_else(|| config.todo_file.clone()).ok_or_else(|| {
        color_eyre::eyre::eyre!(
            "todo_file is not set in config.toml. Hint: set todo_file or pass --todo"
        )
    })?;

    let archive_path = if let Some(explicit_archive) = overrides.archive.clone() {
        explicit_archive
    } else if overrides.todo.is_some() {
        default_archive_for_todo(&todo_path)
    } else if let Some(config_archive) = config.done_file.clone() {
        config_archive
    } else {
        default_archive_for_todo(&todo_path)
    };

    Ok(StartupPaths {
        todo_path,
        archive_path,
    })
}

impl TuiConfig {
    /// Returns the unified config file path: `~/.todotxt.rs/config.toml` on all platforms.
    ///
    /// All three files (`config.toml`, `todo.txt`, `done.txt`) live together in
    /// `~/.todotxt.rs/` by default, making the setup self-contained and predictable
    /// regardless of OS.
    pub fn default_path() -> Option<PathBuf> {
        directories::BaseDirs::new()
            .map(|dirs| dirs.home_dir().join(".todotxt.rs").join("config.toml"))
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

            // Parse through toml::Value so malformed [[panes]] entries can be skipped
            // without failing startup for the whole config file.
            let mut root: toml::Value = toml::from_str(&content)
                .map_err(|e| color_eyre::eyre::eyre!("parsing config {}: {}", path.display(), e))?;

            let panes_value = if let toml::Value::Table(table) = &mut root {
                table.remove("panes")
            } else {
                None
            };

            let mut config: TuiConfig = root
                .try_into()
                .map_err(|e| color_eyre::eyre::eyre!("parsing config {}: {}", path.display(), e))?;

            if let Some(value) = panes_value {
                match value {
                    toml::Value::Array(items) => {
                        config.panes.clear();
                        for (idx, item) in items.into_iter().enumerate() {
                            match item.try_into::<PaneConfig>() {
                                Ok(pane) => config.panes.push(pane),
                                Err(e) => eprintln!(
                                    "warning: skipping invalid [[panes]] entry {} in {}: {}",
                                    idx + 1,
                                    path.display(),
                                    e
                                ),
                            }
                        }
                    }
                    _ => {
                        eprintln!(
                            "warning: expected [[panes]] array-of-tables in {}, got non-array value; ignoring panes",
                            path.display()
                        );
                    }
                }
            }

            Ok(config)
        } else {
            // Auto-create with default todo_file = ~/.todotxt.rs/todo.txt (first-run UX).
            // Mirrors CLI's load_or_create so both tools work out-of-the-box without
            // requiring the user to manually write a config file.
            let home_todo = directories::BaseDirs::new()
                .map(|b| b.home_dir().join(".todotxt.rs").join("todo.txt"));
            let default = TuiConfig {
                todo_file: home_todo,
                ..TuiConfig::default()
            };
            let toml_str = toml::to_string_pretty(&default)
                .map_err(|e| color_eyre::eyre::eyre!("Failed to serialize default config: {e}"))?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| color_eyre::eyre::eyre!("creating config dir {}: {e}", parent.display()))?;
            }
            std::fs::write(path, &toml_str)
                .map_err(|e| color_eyre::eyre::eyre!("writing default config {}: {e}", path.display()))?;
            Ok(default)
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

/// Parse a human-readable key chord string into a (KeyCode, KeyModifiers) pair (D-03, Phase 22).
///
/// Supported formats:
/// - Single printable char: `"n"`, `"?"`, `"0"`, `" "` (space as literal char)
/// - Named special key: `"enter"`, `"esc"`, `"backspace"`, `"delete"`, `"up"`, `"down"`,
///   `"left"`, `"right"`, `"tab"`, `"space"`, `"f1"`–`"f12"`
/// - Modifier+key: `"ctrl+d"`, `"shift+f5"`, `"alt+enter"`
///
/// Returns `None` for empty input or unrecognized tokens.
pub(crate) fn parse_key_chord(s: &str) -> Option<(KeyCode, KeyModifiers)> {
    let normalized = s.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }

    let parts: Vec<&str> = normalized.split('+').collect();
    if parts.is_empty() {
        return None;
    }

    // Split into modifier tokens (all but last) and the key token (last).
    let (mod_tokens, key_token) = parts.split_at(parts.len() - 1);
    let key_token = key_token[0];

    if key_token.is_empty() {
        return None;
    }

    // Parse modifier tokens.
    let mut modifiers = KeyModifiers::NONE;
    for &m in mod_tokens {
        match m {
            "ctrl" => modifiers |= KeyModifiers::CONTROL,
            "shift" => modifiers |= KeyModifiers::SHIFT,
            "alt" => modifiers |= KeyModifiers::ALT,
            _ => return None,
        }
    }

    // Parse the key token.
    let key_code = match key_token {
        "enter" => KeyCode::Enter,
        "esc" => KeyCode::Esc,
        "backspace" => KeyCode::Backspace,
        "delete" | "del" => KeyCode::Delete,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "tab" => KeyCode::Tab,
        "space" => KeyCode::Char(' '),
        // F-keys: "f1"–"f12"
        s if s.starts_with('f') && s.len() > 1 => {
            if let Ok(n) = s[1..].parse::<u8>() {
                if n >= 1 && n <= 12 {
                    KeyCode::F(n)
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        // Single printable character
        s if s.chars().count() == 1 => {
            let c = s.chars().next().unwrap();
            KeyCode::Char(c)
        }
        _ => return None,
    };

    Some((key_code, modifiers))
}

/// Returns the 16 default key bindings for overridable Normal-mode actions (D-02, Phase 22).
///
/// These defaults match the hardcoded bindings that existed before Phase 22.
/// Used as the base map by `resolve_keymap`.
pub(crate) fn default_keymap() -> HashMap<String, (KeyCode, KeyModifiers)> {
    let mut m = HashMap::new();
    m.insert("quit".into(),            (KeyCode::Char('q'), KeyModifiers::NONE));
    m.insert("add".into(),             (KeyCode::Char('n'), KeyModifiers::NONE));
    m.insert("edit".into(),            (KeyCode::Char('e'), KeyModifiers::NONE));
    m.insert("delete".into(),          (KeyCode::Char('d'), KeyModifiers::NONE));
    m.insert("bulk_delete".into(),     (KeyCode::Char('D'), KeyModifiers::NONE));
    m.insert("bulk_append".into(),     (KeyCode::Char('T'), KeyModifiers::NONE));
    m.insert("toggle_done".into(),     (KeyCode::Char('x'), KeyModifiers::NONE));
    m.insert("filter_open".into(),     (KeyCode::Char('f'), KeyModifiers::NONE));
    m.insert("filter_define".into(),   (KeyCode::Char('F'), KeyModifiers::NONE));
    m.insert("filter_toggle".into(),   (KeyCode::Char('f'), KeyModifiers::CONTROL));
    m.insert("sort_cycle".into(),      (KeyCode::Char('o'), KeyModifiers::NONE));
    m.insert("group_toggle".into(),    (KeyCode::Char('g'), KeyModifiers::NONE));
    m.insert("deferred_toggle".into(), (KeyCode::Char('h'), KeyModifiers::NONE));
    m.insert("theme_cycle".into(),     (KeyCode::Char('t'), KeyModifiers::NONE));
    m.insert("disjoint_select".into(), (KeyCode::Char('v'), KeyModifiers::NONE));
    m.insert("disjoint_mark".into(),   (KeyCode::Char(' '), KeyModifiers::NONE));
    m.insert("quick_context".into(),   (KeyCode::Char('@'), KeyModifiers::NONE));
    m.insert("quick_project".into(),   (KeyCode::Char('+'), KeyModifiers::NONE));
    // Phase 22 parity hotkeys (D-11)
    m.insert("help".into(),            (KeyCode::Char('?'), KeyModifiers::NONE));
    m.insert("clear_filter".into(),    (KeyCode::Char('0'), KeyModifiers::NONE));
    m.insert("reload".into(),          (KeyCode::Char('.'), KeyModifiers::NONE));
    // Phase 26 pane lifecycle hotkeys (D-17, D-18, D-20)
    m.insert("pane_add".into(),        (KeyCode::Char('n'), KeyModifiers::CONTROL));
    m.insert("pane_delete".into(),     (KeyCode::Char('w'), KeyModifiers::CONTROL));
    m.insert("pane_hide_toggle".into(), (KeyCode::Char('p'), KeyModifiers::CONTROL));
    m
}

/// Build the effective key binding map for this session (D-04, D-05, Phase 22).
///
/// Starts from `default_keymap()`, applies valid user overrides from `config.keymap`,
/// and collects a warning string for every invalid entry (unknown action name or
/// unparseable chord string). When two actions map to the same chord, both are reverted
/// to their defaults and a conflict warning is emitted (D-07, Plan 22-02).
///
/// Returns `(effective_bindings, warnings)`.
pub fn resolve_keymap(config: &TuiConfig) -> (HashMap<String, (KeyCode, KeyModifiers)>, Vec<String>) {
    let mut effective = default_keymap();
    let known_actions: std::collections::HashSet<String> =
        effective.keys().cloned().collect();
    let mut warnings = Vec::new();

    for (action, chord_str) in &config.keymap {
        if !known_actions.contains(action.as_str()) {
            warnings.push(format!(
                "unknown action '{}' in [keymap] — ignored",
                action
            ));
        } else if let Some(binding) = parse_key_chord(chord_str) {
            effective.insert(action.clone(), binding);
        } else {
            warnings.push(format!(
                "invalid key chord '{}' for action '{}' in [keymap] — default used",
                chord_str, action
            ));
        }
    }

    // Conflict detection (D-07, Plan 22-02): if two or more actions share the same
    // chord after overrides are applied, revert all conflicting actions to their defaults.
    let defaults = default_keymap();
    let mut chord_to_actions: HashMap<(KeyCode, KeyModifiers), Vec<String>> = HashMap::new();
    for (action, binding) in &effective {
        chord_to_actions.entry(*binding).or_default().push(action.clone());
    }
    for (chord, actions) in &chord_to_actions {
        if actions.len() > 1 {
            let chord_desc = format!("{:?}+{:?}", chord.0, chord.1);
            warnings.push(format!(
                "conflict: actions [{}] all bound to {} — all reverted to defaults",
                actions.join(", "),
                chord_desc
            ));
            for action in actions {
                if let Some(default_binding) = defaults.get(action.as_str()) {
                    effective.insert(action.clone(), *default_binding);
                }
            }
        }
    }

    (effective, warnings)
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

    // ── Phase 22 keymap tests ─────────────────────────────────────────────────

    #[test]
    fn keymap_field_deserializes_from_toml() {
        let toml_str = r#"
[keymap]
delete = "backspace"
"#;
        let config: TuiConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
        assert_eq!(config.keymap.get("delete").map(|s| s.as_str()), Some("backspace"));
    }

    #[test]
    fn keymap_defaults_to_empty_when_section_absent() {
        let toml_str = r#"
auto_creation_date = false
"#;
        let config: TuiConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
        assert!(config.keymap.is_empty(), "keymap should default to empty map when [keymap] is absent");
    }

    #[test]
    fn parse_key_chord_ctrl_d() {
        let result = parse_key_chord("ctrl+d");
        assert_eq!(result, Some((KeyCode::Char('d'), KeyModifiers::CONTROL)));
    }

    #[test]
    fn parse_key_chord_backspace() {
        let result = parse_key_chord("backspace");
        assert_eq!(result, Some((KeyCode::Backspace, KeyModifiers::NONE)));
    }

    #[test]
    fn parse_key_chord_f5() {
        let result = parse_key_chord("f5");
        assert_eq!(result, Some((KeyCode::F(5), KeyModifiers::NONE)));
    }

    #[test]
    fn parse_key_chord_question_mark() {
        let result = parse_key_chord("?");
        assert_eq!(result, Some((KeyCode::Char('?'), KeyModifiers::NONE)));
    }

    #[test]
    fn parse_key_chord_space_word() {
        let result = parse_key_chord("SPACE");
        assert_eq!(result, Some((KeyCode::Char(' '), KeyModifiers::NONE)));
    }

    #[test]
    fn parse_key_chord_empty_returns_none() {
        assert_eq!(parse_key_chord(""), None);
        assert_eq!(parse_key_chord("  "), None);
    }

    #[test]
    fn parse_key_chord_unknown_key_returns_none() {
        assert_eq!(parse_key_chord("ctrl+bogus_key"), None);
        assert_eq!(parse_key_chord("bogus_key"), None);
    }

    #[test]
    fn resolve_keymap_unknown_action_adds_warning() {
        let mut config = TuiConfig::default();
        config.keymap.insert("nonexistent_action".into(), "x".into());
        let (effective, warnings) = resolve_keymap(&config);
        assert!(
            warnings.iter().any(|w| w.contains("nonexistent_action")),
            "expected warning for unknown action"
        );
        // Default map returned unchanged — still has 16 entries
        assert_eq!(effective.len(), default_keymap().len());
    }

    #[test]
    fn resolve_keymap_invalid_chord_adds_warning_and_keeps_default() {
        let mut config = TuiConfig::default();
        config.keymap.insert("delete".into(), "bogus_chord".into());
        let (effective, warnings) = resolve_keymap(&config);
        assert!(
            warnings.iter().any(|w| w.contains("bogus_chord") && w.contains("delete")),
            "expected warning for invalid chord"
        );
        // Default for "delete" should still be 'd'
        assert_eq!(
            effective.get("delete"),
            Some(&(KeyCode::Char('d'), KeyModifiers::NONE))
        );
    }

    #[test]
    fn resolve_keymap_valid_override_applied() {
        let mut config = TuiConfig::default();
        config.keymap.insert("delete".into(), "backspace".into());
        let (effective, warnings) = resolve_keymap(&config);
        assert!(warnings.is_empty(), "no warnings expected for valid override");
        assert_eq!(
            effective.get("delete"),
            Some(&(KeyCode::Backspace, KeyModifiers::NONE))
        );
    }

    #[test]
    fn resolve_keymap_conflict_detection_reverts_both_actions() {
        // Configure two actions to the same chord — both should revert to defaults.
        let mut config = TuiConfig::default();
        // "delete" default = 'd', override to "x"
        // "toggle_done" default = "x", so this creates a conflict
        config.keymap.insert("delete".into(), "x".into());
        let (effective, warnings) = resolve_keymap(&config);
        assert!(
            warnings.iter().any(|w| w.contains("conflict")),
            "expected a conflict warning"
        );
        // Both "delete" and "toggle_done" should revert to their defaults
        assert_eq!(
            effective.get("delete"),
            Some(&(KeyCode::Char('d'), KeyModifiers::NONE)),
            "delete should revert to default 'd'"
        );
        assert_eq!(
            effective.get("toggle_done"),
            Some(&(KeyCode::Char('x'), KeyModifiers::NONE)),
            "toggle_done should revert to default 'x'"
        );
    }

    #[test]
    fn panes_default_to_empty_when_section_absent() {
        let toml_str = r#"
todo_file = "tasks.txt"
"#;
        let config: TuiConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
        assert!(config.panes.is_empty(), "panes should default to empty vec when [[panes]] is absent");
    }

    #[test]
    fn pane_entry_defaults_missing_fields() {
        let toml_str = r#"
[[panes]]
"#;
        let config: TuiConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
        assert_eq!(config.panes.len(), 1);
        let pane = &config.panes[0];
        assert_eq!(pane.label, "");
        assert_eq!(pane.filter, "");
        assert_eq!(pane.sort, PaneSort::FileOrder);
        assert!(!pane.group);
    }

    #[test]
    fn pane_sort_supports_snake_case_values() {
        let toml_str = r#"
[[panes]]
sort = "due_date"
"#;
        let config: TuiConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
        assert_eq!(config.panes.len(), 1);
        assert_eq!(config.panes[0].sort, PaneSort::DueDate);
    }
}

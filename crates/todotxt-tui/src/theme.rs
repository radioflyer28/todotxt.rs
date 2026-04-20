//! Theme definitions and pre-computed style sheets for the TUI.
//!
//! Styles are computed once at startup in `StyleSheet::from_theme()` and
//! stored on `App`. Render functions read `self.styles.*` — no per-frame theme logic.

use ratatui::style::{Color, Modifier, Style};

/// Built-in color themes.
///
/// `Default` targets dark terminal backgrounds; `Light` targets light backgrounds.
/// Parse a theme name with [`Theme::from_str`] — unknown names fall back to `Default`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Default,
    Light,
}

impl Theme {
    /// Parse a theme name from config. Returns `Theme::Default` for `""`, `"default"`,
    /// or any unrecognized value — never panics (D-03 in 13-CONTEXT.md).
    pub fn from_str(s: &str) -> Self {
        match s {
            "light" => Theme::Light,
            _ => Theme::Default,
        }
    }
}

/// Pre-computed styles for a given theme + NO_COLOR combination.
///
/// Build with [`StyleSheet::from_theme`]. Store on `App` as `pub styles: StyleSheet`.
/// Extend with new fields in future phases (context/project token coloring, status bar).
pub struct StyleSheet {
    /// Style for priority (A) tasks — highest urgency.
    pub priority_a: Style,
    /// Style for priority (B) tasks — medium urgency.
    pub priority_b: Style,
    /// Style for priority (C) tasks — lower urgency.
    pub priority_c: Style,
    /// Style for overdue tasks (due date in past, not completed).
    pub overdue: Style,
}

impl StyleSheet {
    /// Build a `StyleSheet` from a theme and NO_COLOR flag.
    ///
    /// When `no_color` is `true` (NO_COLOR env var present and non-empty),
    /// all `Color::*` fields are stripped — `Modifier::BOLD` is preserved on overdue
    /// because it is a modifier, not a color code (NO_COLOR standard, D-06 in 13-CONTEXT.md).
    pub fn from_theme(theme: Theme, no_color: bool) -> Self {
        if no_color {
            // NO_COLOR: strip all Color::* — preserve modifiers (D-06).
            StyleSheet {
                priority_a: Style::default(),
                priority_b: Style::default(),
                priority_c: Style::default(),
                overdue: Style::default().add_modifier(Modifier::BOLD),
            }
        } else {
            match theme {
                Theme::Default => StyleSheet {
                    // Dark terminal palette — bright variants for contrast (D-02).
                    priority_a: Style::default().fg(Color::LightRed),
                    priority_b: Style::default().fg(Color::Yellow),
                    priority_c: Style::default().fg(Color::Cyan),
                    overdue: Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD),
                },
                Theme::Light => StyleSheet {
                    // Light terminal palette — intentionally distinct from Default.
                    // Some terminals render Red and LightRed similarly; these colors
                    // make theme switching visibly obvious during verification.
                    priority_a: Style::default().fg(Color::Blue),
                    priority_b: Style::default().fg(Color::Magenta),
                    priority_c: Style::default().fg(Color::Green),
                    overdue: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                },
            }
        }
    }
}

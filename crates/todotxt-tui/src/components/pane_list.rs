//! Pane list widget — renders a single pane with its task list

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use std::collections::HashSet;
use crate::state::{Pane, DisplayRow};
use crate::theme::StyleSheet;
use todotxt_core::DueStatus;
use chrono::Local;

#[allow(dead_code)]
pub struct PaneList;

impl PaneList {
    /// Build the pane border title string from label, filter, and active state.
    /// Used by render() and exposed for unit testing.
    pub(crate) fn build_pane_title(pane: &Pane, is_active: bool, label_selected: bool) -> String {
        let mut header_parts: Vec<String> = Vec::new();

        if !pane.label.is_empty() {
            let label_display = if is_active && label_selected {
                format!("✎ {}", pane.label)
            } else if is_active {
                format!("▶ {}", pane.label)
            } else {
                format!("  {}", pane.label)
            };
            header_parts.push(label_display);
        } else if is_active {
            header_parts.push("▶".to_string());
        }

        let trimmed_filter = pane.filter_query.trim();
        if !trimmed_filter.is_empty() {
            let filter_display = if trimmed_filter.chars().count() > 20 {
                let truncated: String = trimmed_filter.chars().take(17).collect();
                format!("{}…", truncated)
            } else {
                trimmed_filter.to_string()
            };
            header_parts.push(filter_display.to_string());
        }

        if header_parts.is_empty() {
            if is_active { "▶".to_string() } else { " ".to_string() }
        } else {
            header_parts.join(" | ")
        }
    }

    /// Truncate text at word boundaries with ellipsis to fit within max_width.
    /// Preserves one-line-per-task visual aesthetic in narrow panes.
    fn truncate_for_width(text: &str, max_width: usize) -> String {
        if text.len() <= max_width {
            return text.to_string();
        }

        if max_width < 4 {
            // Too narrow, just show truncation indicator
            return "…".to_string();
        }

        // Try to truncate at last space before limit
        let truncate_at = max_width.saturating_sub(1);
        if let Some(last_space) = text[..truncate_at.min(text.len())].rfind(' ') {
            if last_space > 0 {
                return format!("{}…", &text[..last_space]);
            }
        }

        // No space found, hard truncate with ellipsis
        let truncated: String = text.chars().take(truncate_at).collect();
        format!("{}…", truncated)
    }

    /// Render a single pane into the given area
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        pane: &Pane,
        is_active: bool,
        label_selected: bool,
        selected_tasks: &HashSet<usize>,
        disjoint_select: bool,
        stylesheet: &StyleSheet,
        task_list: &todotxt_core::TaskList,
        show_deferred: bool,
    ) {
        // Border style depends on active state
        let border_style = if is_active {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::DarkGray)
        };

        let title = Self::build_pane_title(pane, is_active, label_selected);

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        let tasks = task_list.tasks();

        // Build list items from display_rows
        let items: Vec<ListItem> = if pane.display_rows.is_empty() {
            vec![ListItem::new("(no tasks)")]
        } else {
            let usable_width = area.width.saturating_sub(4) as usize; // Account for borders/padding
            pane.display_rows
                .iter()
                .enumerate()
                .map(|(row_idx, row)| {
                    match row {
                        DisplayRow::GroupHeader(label) => {
                            let truncated = if usable_width > 0 {
                                Self::truncate_for_width(&format!(" {}", label), usable_width)
                            } else {
                                label.clone()
                            };
                            ListItem::new(truncated)
                                .style(stylesheet.group_header)
                        }
                        DisplayRow::Task(ci) => {
                            let t = &tasks[*ci];
                            let is_selected = selected_tasks.contains(ci);
                            let is_cursor = row_idx == pane.selected;
                            let prefix = if disjoint_select && is_cursor {
                                "V "
                            } else if is_selected && !is_cursor {
                                "> "
                            } else {
                                ""
                            };
                            let full_content = t.to_raw();
                            let prefixed = format!("{}{}", prefix, full_content);
                            let content = if usable_width > 0 {
                                Self::truncate_for_width(&prefixed, usable_width)
                            } else {
                                prefixed
                            };

                            // Priority and overdue coloring (D-01, D-09 in 13-CONTEXT.md).
                            let style = if t.completed {
                                // Completed tasks: DIM only, no color (D-01, D-06).
                                Style::default().add_modifier(Modifier::DIM)
                            } else if show_deferred
                                && t.threshold_date.is_some_and(|d| d > Local::now().date_naive())
                            {
                                Style::default().add_modifier(Modifier::DIM)
                            } else if t.priority == Some('A') {
                                stylesheet.priority_a
                            } else if t.priority == Some('B') {
                                stylesheet.priority_b
                            } else if t.priority == Some('C') {
                                stylesheet.priority_c
                            } else if t.due_status() == DueStatus::Overdue {
                                stylesheet.overdue
                            } else {
                                Style::default()
                            };

                            let style = if is_selected && !is_cursor {
                                style.add_modifier(Modifier::BOLD)
                            } else {
                                style
                            };

                            ListItem::new(content).style(style)
                        }
                    }
                })
                .collect()
        };

        let cursor_is_selected = pane
            .display_rows
            .get(pane.selected)
            .map(|r| matches!(r, DisplayRow::Task(ci) if selected_tasks.contains(ci)))
            .unwrap_or(false);
        let highlight_modifier = if cursor_is_selected {
            Modifier::REVERSED | Modifier::BOLD
        } else {
            Modifier::REVERSED
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().add_modifier(highlight_modifier));

        let mut list_state = ListState::default();
        if !label_selected && !pane.display_rows.is_empty() {
            list_state = list_state.with_selected(Some(pane.selected));
        }

        frame.render_stateful_widget(list, area, &mut list_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Pane;
    use todotxt_core::SortOrder;

    #[test]
    fn pane_header_no_sort_indicator() {
        // With a non-FileOrder sort, header must NOT contain "sort:"
        let mut p = Pane::new(0, "Pane 3".to_string());
        p.sort_order = SortOrder::CompletedDate;
        let title = PaneList::build_pane_title(&p, true, false);
        assert!(!title.contains("sort:"), "Header must not contain 'sort:': {}", title);
        assert_eq!(title, "▶ Pane 3");
    }

    #[test]
    fn pane_header_filter_no_prefix() {
        // With a filter set, header must NOT contain "filter:" prefix
        let mut p = Pane::new(0, "Pane 3".to_string());
        p.filter_query = "@work +CTRC".to_string();
        p.sort_order = SortOrder::CompletedDate;
        let title = PaneList::build_pane_title(&p, true, false);
        assert!(!title.contains("filter:"), "Header must not contain 'filter:': {}", title);
        assert_eq!(title, "▶ Pane 3 | @work +CTRC");
    }
}

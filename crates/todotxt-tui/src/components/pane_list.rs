//! Pane list widget — renders a single pane with its task list

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use crate::state::{Pane, DisplayRow};
use crate::theme::StyleSheet;
use todotxt_core::DueStatus;
use todotxt_core::SortOrder;
use chrono::Local;

#[allow(dead_code)]
pub struct PaneList;

impl PaneList {
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
    #[allow(dead_code)]
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        pane: &Pane,
        is_active: bool,
        label_selected: bool,
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

        // Build pane header: [label] - [filter] - [sort]
        let mut header_parts = Vec::new();

        // Add label if non-empty
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
            // Show indicator for active pane even when label is empty
            header_parts.push("▶".to_string());
        }

        // Add filter if non-empty
        let trimmed_filter = pane.filter_query.trim();
        if !trimmed_filter.is_empty() {
            let filter_display = if trimmed_filter.len() > 20 {
                format!("{}…", &trimmed_filter[..17])
            } else {
                trimmed_filter.to_string()
            };
            header_parts.push(format!("filter: {}", filter_display));
        }

        // Add sort order if not FileOrder
        if pane.sort_order != SortOrder::FileOrder {
            let sort_name = match pane.sort_order {
                SortOrder::FileOrder => "file",
                SortOrder::Alphabetical => "alpha",
                SortOrder::Priority => "priority",
                SortOrder::DueDate => "due",
                _ => "unknown",
            };
            header_parts.push(format!("sort: {}", sort_name));
        }

        let title = if header_parts.is_empty() {
            if is_active {
                "▶".to_string()
            } else {
                " ".to_string()
            }
        } else {
            header_parts.join(" | ")
        };

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
                .map(|(_row_idx, row)| {
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
                            let full_content = t.to_raw();
                            let content = if usable_width > 0 {
                                Self::truncate_for_width(&full_content, usable_width)
                            } else {
                                full_content.to_string()
                            };

                            // Priority and overdue coloring (D-01, D-09 in 13-CONTEXT.md).
                            let style = if t.completed {
                                // Completed tasks: DIM only, no color (D-01, D-06).
                                Style::default().add_modifier(Modifier::DIM)
                            } else if show_deferred
                                && t.threshold_date.map_or(false, |d| d > Local::now().date_naive())
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

                            ListItem::new(content).style(style)
                        }
                    }
                })
                .collect()
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        let mut list_state = ListState::default();
        if !label_selected && !pane.display_rows.is_empty() {
            list_state = list_state.with_selected(Some(pane.selected));
        }

        frame.render_stateful_widget(list, area, &mut list_state);
    }
}

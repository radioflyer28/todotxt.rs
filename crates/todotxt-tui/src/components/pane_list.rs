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
use chrono::Local;

#[allow(dead_code)]
pub struct PaneList;

impl PaneList {
    /// Render a single pane into the given area
    #[allow(dead_code)]
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        pane: &Pane,
        is_active: bool,
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

        let title = if is_active {
            format!("▶ {}", pane.label)  // Visual indicator for active pane
        } else {
            format!("  {}", pane.label)
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
            pane.display_rows
                .iter()
                .enumerate()
                .map(|(_row_idx, row)| {
                    match row {
                        DisplayRow::GroupHeader(label) => {
                            ListItem::new(format!(" {}", label))
                                .style(stylesheet.group_header)
                        }
                        DisplayRow::Task(ci) => {
                            let t = &tasks[*ci];
                            let content = format!("{}: {}", ci + 1, t.to_raw());

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
        if !pane.display_rows.is_empty() {
            list_state = list_state.with_selected(Some(pane.selected));
        }

        frame.render_stateful_widget(list, area, &mut list_state);
    }
}

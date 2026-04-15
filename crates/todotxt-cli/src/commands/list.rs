use crate::{cli::ListArgs, config::Config, output::Renderer, CliError};
use std::path::Path;
use todotxt_core::{Filter, TaskList};

/// Resolve filter tokens, expanding `:preset` names to their query strings (D-08–D-12).
fn build_filter(args: &ListArgs, cfg: &Config) -> Filter {
    let mut query_parts: Vec<String> = Vec::new();

    for token in &args.filters {
        if let Some(preset_name) = token.strip_prefix(':') {
            // Preset token: look up in config (D-11)
            if let Some(preset) = cfg.presets.get(preset_name) {
                if let Some(q) = &preset.filter {
                    // D-12: preset composes with other filters
                    query_parts.push(q.clone());
                }
            }
            // Unknown presets are silently ignored
        } else {
            // Plain token is always a filter token — never treated as preset (D-12)
            query_parts.push(token.clone());
        }
    }

    // --filter flag tokens append to the composed query (D-09)
    if let Some(fq) = &args.filter_query {
        query_parts.push(fq.clone());
    }

    let combined = query_parts.join(" ");
    if combined.trim().is_empty() {
        Filter::new() // no filter = show all
    } else {
        Filter::from_query(&combined)
    }
}

pub fn run(todo_path: &Path, args: &ListArgs, cfg: &Config, renderer: &Renderer) -> Result<(), CliError> {
    let list = TaskList::load(todo_path)?;
    let filter = build_filter(args, cfg);
    let tasks = list.filter(&filter);
    renderer.print_tasks(&tasks);
    renderer.print_count(tasks.len());
    // P-10: empty result = exit 0, not exit 1
    Ok(())
}


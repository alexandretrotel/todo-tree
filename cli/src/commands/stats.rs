use super::load_config;
use crate::{
    cli,
    parser::TodoParser,
    scanner::{ScanOptions, Scanner},
    utils::display::priority_to_color,
};
use anyhow::{Context, Result};
use colored::Colorize;
use serde_json::json;
use todo_tree_core::Priority;

pub fn run(args: cli::StatsArgs, global: &cli::GlobalOptions) -> Result<()> {
    let path = args
        .path
        .clone()
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let path = path
        .canonicalize()
        .with_context(|| format!("Failed to resolve path: {}", path.display()))?;

    let config = load_config(&path, global.config.as_deref())?;
    let tags = args.tags.clone().unwrap_or(config.tags.clone());

    let parser = TodoParser::new(&tags, false);
    let scanner = Scanner::new(parser, ScanOptions::default());
    let result = scanner.scan(&path)?;

    if args.json {
        let stats = json!({
            "total_items": result.summary.total_count,
            "files_with_todos": result.summary.files_with_todos,
            "files_scanned": result.summary.files_scanned,
            "tag_counts": result.summary.tag_counts,
            "items_per_file": if result.summary.files_with_todos > 0 {
                result.summary.total_count as f64 / result.summary.files_with_todos as f64
            } else {
                0.0
            },
        });
        println!("{}", serde_json::to_string_pretty(&stats)?);
    } else {
        println!("{}", "TODO Statistics".bold().underline());
        println!();
        println!("  Total items:        {}", result.summary.total_count);
        println!("  Files with TODOs:   {}", result.summary.files_with_todos);
        println!("  Files scanned:      {}", result.summary.files_scanned);

        if result.summary.files_with_todos > 0 {
            let avg = result.summary.total_count as f64 / result.summary.files_with_todos as f64;
            println!("  Avg items per file: {:.2}", avg);
        }

        println!();
        println!("{}", "By Tag:".bold());

        let mut tags: Vec<_> = result.summary.tag_counts.iter().collect();
        tags.sort_by(|a, b| b.1.cmp(a.1));

        for (tag, count) in tags {
            let percentage = if result.summary.total_count > 0 {
                (*count as f64 / result.summary.total_count as f64) * 100.0
            } else {
                0.0
            };

            let bar_width = 20;
            let filled = ((percentage / 100.0) * bar_width as f64) as usize;
            let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);

            if global.no_color {
                println!("  {:<8} {:>4} ({:>5.1}%) {}", tag, count, percentage, bar);
            } else {
                let color = priority_to_color(Priority::from_tag(tag));
                println!(
                    "  {:<8} {:>4} ({:>5.1}%) {}",
                    tag.color(color),
                    count,
                    percentage,
                    bar.dimmed()
                );
            }
        }
    }

    Ok(())
}

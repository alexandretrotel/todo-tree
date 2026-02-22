use super::options::PrintOptions;
use super::utils::colorize_tag;
use colored::Colorize;
use std::io::{self, Write};
use todo_tree_core::ScanResult;

pub fn print_summary<W: Write>(
    writer: &mut W,
    result: &ScanResult,
    options: &PrintOptions,
) -> io::Result<()> {
    let summary_line = format!(
        "Found {} TODO items in {} files ({} files scanned)",
        result.summary.total_count, result.summary.files_with_todos, result.summary.files_scanned
    );

    if options.colored {
        writeln!(writer, "{}", summary_line.bold())?;
    } else {
        writeln!(writer, "{}", summary_line)?;
    }

    if !result.summary.tag_counts.is_empty() {
        let mut tags: Vec<_> = result.summary.tag_counts.iter().collect();
        tags.sort_by(|a, b| b.1.cmp(a.1));

        let breakdown: Vec<String> = tags
            .iter()
            .map(|(tag, count)| {
                if options.colored {
                    format!("{}: {}", colorize_tag(tag, options), count)
                } else {
                    format!("{}: {}", tag, count)
                }
            })
            .collect();

        writeln!(writer, "  {}", breakdown.join(", "))?;
    }

    Ok(())
}

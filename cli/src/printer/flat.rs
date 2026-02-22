use super::options::PrintOptions;
use super::utils::{colorize_tag, format_path, make_clickable_link};
use colored::Colorize;
use std::io::{self, Write};
use std::path::Path;
use todo_tree_core::{ScanResult, TodoItem};

pub fn print_flat<W: Write>(
    writer: &mut W,
    result: &ScanResult,
    options: &PrintOptions,
) -> io::Result<()> {
    if result.is_empty() {
        writeln!(writer, "{}", "No TODO items found.".dimmed())?;
        return Ok(());
    }

    let mut all_items = result.all_items();
    all_items.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.line.cmp(&b.1.line)));

    for (path, item) in all_items {
        print_flat_item(writer, &path, &item, options)?;
    }

    Ok(())
}

fn print_flat_item<W: Write>(
    writer: &mut W,
    path: &Path,
    item: &TodoItem,
    options: &PrintOptions,
) -> io::Result<()> {
    let display_path = format_path(path, options);
    let link = make_clickable_link(path, item.line, options);

    let path_str = link.unwrap_or_else(|| {
        if options.colored {
            display_path.bold().to_string()
        } else {
            display_path.to_string()
        }
    });

    let line_col = format!(":{}:{}", item.line, item.column);
    let line_col_display = if options.colored {
        line_col.cyan().to_string()
    } else {
        line_col
    };

    let tag = colorize_tag(&item.tag, options);

    writeln!(
        writer,
        "{}{} [{}] {}",
        path_str, line_col_display, tag, item.message
    )?;
    Ok(())
}

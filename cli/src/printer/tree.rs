use super::options::PrintOptions;
use super::utils::{colorize_tag, format_path, make_clickable_link, make_line_link};
use colored::Colorize;
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
use todo_tree_core::{ScanResult, TodoItem};

pub fn print_tree<W: Write>(
    writer: &mut W,
    result: &ScanResult,
    options: &PrintOptions,
) -> io::Result<()> {
    if result.is_empty() {
        writeln!(writer, "{}", "No TODO items found.".dimmed())?;
        return Ok(());
    }

    if options.group_by_tag {
        print_tree_by_tag(writer, result, options)?;
    } else {
        print_tree_by_file(writer, result, options)?;
    }

    Ok(())
}

fn print_tree_by_file<W: Write>(
    writer: &mut W,
    result: &ScanResult,
    options: &PrintOptions,
) -> io::Result<()> {
    let sorted_files = result.sorted_files();
    let total_files = sorted_files.len();

    for (idx, (path, items)) in sorted_files.iter().enumerate() {
        let is_last_file = idx == total_files - 1;
        print_file_header(writer, path, items.len(), is_last_file, options)?;

        let total_items = items.len();
        for (item_idx, item) in items.iter().enumerate() {
            let is_last_item = item_idx == total_items - 1;
            print_tree_item(writer, item, is_last_file, is_last_item, path, options)?;
        }
    }

    Ok(())
}

fn print_tree_by_tag<W: Write>(
    writer: &mut W,
    result: &ScanResult,
    options: &PrintOptions,
) -> io::Result<()> {
    let mut by_tag: HashMap<String, Vec<(PathBuf, TodoItem)>> = HashMap::new();

    for (path, items) in &result.files_map {
        for item in items {
            by_tag
                .entry(item.tag.clone())
                .or_default()
                .push((path.clone(), item.clone()));
        }
    }

    let mut tags: Vec<_> = by_tag.keys().collect();
    tags.sort();

    let total_tags = tags.len();

    for (idx, tag) in tags.iter().enumerate() {
        let is_last_tag = idx == total_tags - 1;
        let items = by_tag.get(*tag).unwrap();

        let prefix = if is_last_tag {
            "└──"
        } else {
            "├──"
        };
        let colored_tag = colorize_tag(tag, options);
        writeln!(writer, "{} {} ({})", prefix, colored_tag, items.len())?;

        let total_items = items.len();
        for (item_idx, (path, item)) in items.iter().enumerate() {
            let is_last_item = item_idx == total_items - 1;
            let tree_prefix = if is_last_tag { "    " } else { "│   " };
            let item_prefix = if is_last_item {
                "└──"
            } else {
                "├──"
            };

            let display_path = format_path(path, options);
            let link = make_clickable_link(path, item.line, options);

            writeln!(
                writer,
                "{}{} {}:{} - {}",
                tree_prefix,
                item_prefix,
                link.unwrap_or_else(|| display_path.to_string()),
                item.line.to_string().cyan(),
                item.message.dimmed()
            )?;
        }
    }

    Ok(())
}

fn print_file_header<W: Write>(
    writer: &mut W,
    path: &Path,
    item_count: usize,
    is_last: bool,
    options: &PrintOptions,
) -> io::Result<()> {
    let prefix = if is_last { "└──" } else { "├──" };
    let display_path = format_path(path, options);
    let link = make_clickable_link(path, 1, options);

    let path_str = link.unwrap_or_else(|| {
        if options.colored {
            display_path.bold().to_string()
        } else {
            display_path.to_string()
        }
    });

    let count_str = format!("({})", item_count);
    let count_display = if options.colored {
        count_str.dimmed().to_string()
    } else {
        count_str
    };

    writeln!(writer, "{} {} {}", prefix, path_str, count_display)?;
    Ok(())
}

fn print_tree_item<W: Write>(
    writer: &mut W,
    item: &TodoItem,
    is_last_file: bool,
    is_last_item: bool,
    path: &Path,
    options: &PrintOptions,
) -> io::Result<()> {
    let tree_prefix = if is_last_file { "    " } else { "│   " };
    let item_prefix = if is_last_item {
        "└──"
    } else {
        "├──"
    };

    let tag = colorize_tag(&item.tag, options);
    let line_num = if options.colored {
        format!("L{}", item.line).cyan().to_string()
    } else {
        format!("L{}", item.line)
    };

    let line_display = if options.clickable_links {
        make_line_link(path, item.line, options).unwrap_or_else(|| line_num.clone())
    } else {
        line_num
    };

    let author_str = item
        .author
        .as_ref()
        .map(|a| format!("({})", a))
        .unwrap_or_default();

    if author_str.is_empty() {
        writeln!(
            writer,
            "{}{} [{}] {}: {}",
            tree_prefix, item_prefix, line_display, tag, item.message
        )?;
    } else {
        let author_display = if options.colored {
            author_str.yellow().to_string()
        } else {
            author_str
        };
        writeln!(
            writer,
            "{}{} [{}] {} {}: {}",
            tree_prefix, item_prefix, line_display, tag, author_display, item.message
        )?;
    }

    Ok(())
}

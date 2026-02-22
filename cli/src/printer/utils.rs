use super::options::PrintOptions;
use crate::utils::display::priority_to_color;
use colored::Colorize;
use std::path::Path;
use todo_tree_core::Priority;

pub fn format_path(path: &Path, options: &PrintOptions) -> String {
    if options.full_paths {
        path.display().to_string()
    } else if let Some(base) = &options.base_path {
        path.strip_prefix(base)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| path.display().to_string())
    } else {
        path.display().to_string()
    }
}

pub fn make_clickable_link(path: &Path, line: usize, options: &PrintOptions) -> Option<String> {
    if !options.clickable_links || !supports_hyperlinks() {
        return None;
    }

    let display_path = format_path(path, options);
    let abs_path = path.canonicalize().ok()?;
    let file_url = format!("file://{}:{}", abs_path.display(), line);

    let link = format!(
        "\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
        file_url,
        if options.colored {
            display_path.bold().to_string()
        } else {
            display_path
        }
    );

    Some(link)
}

pub fn make_line_link(path: &Path, line: usize, options: &PrintOptions) -> Option<String> {
    if !options.clickable_links || !supports_hyperlinks() {
        return None;
    }

    let abs_path = path.canonicalize().ok()?;
    let file_url = format!("file://{}:{}", abs_path.display(), line);
    let display = format!("L{}", line);

    let link = format!(
        "\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
        file_url,
        if options.colored {
            display.cyan().to_string()
        } else {
            display
        }
    );

    Some(link)
}

pub fn colorize_tag(tag: &str, options: &PrintOptions) -> String {
    if !options.colored {
        return tag.to_string();
    }

    let color = priority_to_color(Priority::from_tag(tag));
    tag.color(color).bold().to_string()
}

fn supports_hyperlinks() -> bool {
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        let supported_terminals = [
            "iTerm.app",
            "WezTerm",
            "Hyper",
            "Tabby",
            "Alacritty",
            "vscode",
            "VSCodium",
            "Ghostty",
        ];
        if supported_terminals.iter().any(|t| term_program.contains(t)) {
            return true;
        }
    }

    if let Ok(colorterm) = std::env::var("COLORTERM")
        && (colorterm == "truecolor" || colorterm == "24bit")
    {
        return true;
    }

    if std::env::var("VTE_VERSION").is_ok() {
        return true;
    }

    if std::env::var("KONSOLE_VERSION").is_ok() {
        return true;
    }

    false
}

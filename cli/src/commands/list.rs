use super::load_config;
use crate::{
    cli,
    parser::TodoParser,
    printer::{OutputFormat, PrintOptions, Printer},
    scanner::{ScanOptions, Scanner},
};
use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn run(args: cli::ListArgs, global: &cli::GlobalOptions) -> Result<()> {
    let path = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let path = path
        .canonicalize()
        .with_context(|| format!("Failed to resolve path: {}", path.display()))?;

    let mut config = load_config(&path, global.config.as_deref())?;
    config.merge_with_cli(crate::config::CliOptions {
        tags: args.tags.clone(),
        include: args.include.clone(),
        exclude: args.exclude.clone(),
        json: args.json,
        flat: true,
        no_color: global.no_color,
        ignore_case: args.ignore_case,
        no_require_colon: args.no_require_colon,
    });

    let case_sensitive = !args.ignore_case && !config.ignore_case;
    let require_colon = if args.no_require_colon {
        false
    } else {
        config.require_colon
    };

    let parser = TodoParser::with_options(
        &config.tags,
        case_sensitive,
        require_colon,
        config.custom_pattern.as_deref(),
    );

    let scan_options = ScanOptions {
        include: config.include.clone(),
        exclude: config.exclude.clone(),
        ..Default::default()
    };

    let scanner = Scanner::new(parser, scan_options);
    let mut result = scanner.scan(&path)?;

    if let Some(filter_tag) = &args.filter {
        result = result.filter_by_tag(filter_tag);
    }

    let print_options = PrintOptions {
        format: if args.json {
            OutputFormat::Json
        } else {
            OutputFormat::Flat
        },
        colored: !global.no_color,
        show_line_numbers: true,
        full_paths: false,
        clickable_links: !global.no_color,
        base_path: Some(path),
        show_summary: !args.json,
        group_by_tag: false,
    };

    let printer = Printer::new(print_options);
    printer.print(&result)?;

    Ok(())
}

use super::{load_config, sort_results};
use crate::{
    cli,
    config::CliOptions,
    parser::TodoParser,
    printer::{OutputFormat, PrintOptions, Printer},
    scanner::{ScanOptions, Scanner},
};
use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn run(args: cli::ScanArgs, global: &cli::GlobalOptions) -> Result<()> {
    let path = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let path = path
        .canonicalize()
        .with_context(|| format!("Failed to resolve path: {}", path.display()))?;

    let mut config = load_config(&path, global.config.as_deref())?;
    config.merge_with_cli(CliOptions {
        tags: args.tags.clone(),
        include: args.include.clone(),
        exclude: args.exclude.clone(),
        json: args.json,
        flat: args.flat,
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
        max_depth: args.depth,
        follow_links: args.follow_links,
        hidden: args.hidden,
        threads: 0,
        respect_gitignore: true,
    };

    let scanner = Scanner::new(parser, scan_options);
    let mut result = scanner.scan(&path)?;

    sort_results(&mut result, args.sort);

    let print_options = PrintOptions {
        format: if args.json {
            OutputFormat::Json
        } else if args.flat {
            OutputFormat::Flat
        } else {
            OutputFormat::Tree
        },
        colored: !global.no_color,
        show_line_numbers: true,
        full_paths: false,
        clickable_links: !global.no_color,
        base_path: Some(path),
        show_summary: !args.json,
        group_by_tag: args.group_by_tag,
    };

    let printer = Printer::new(print_options);
    printer.print(&result)?;

    Ok(())
}

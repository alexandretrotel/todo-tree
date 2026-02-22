pub mod cli;
pub mod commands;
pub mod config;
pub mod parser;
pub mod printer;
pub mod scanner;
pub mod utils;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use commands::{init, list, scan, stats, tags as cli_tags};
pub use todo_tree_core::{Priority, ScanResult, ScanSummary, TodoItem};

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.global.no_color || std::env::var("NO_COLOR").is_ok() {
        colored::control::set_override(false);
    }

    match cli.get_command() {
        Commands::Scan(args) => scan::run(args, &cli.global),
        Commands::List(args) => list::run(args, &cli.global),
        Commands::Tags(args) => cli_tags::run(args, &cli.global),
        Commands::Init(args) => init::run(args),
        Commands::Stats(args) => stats::run(args, &cli.global),
    }
}

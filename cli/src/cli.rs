use clap::{Args, Parser, Subcommand, ValueHint};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "todo-tree",
    author,
    version,
    about,
    long_about = None,
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOptions,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Args, Debug, Clone)]
pub struct GlobalOptions {
    #[arg(long, global = true, env = "NO_COLOR", help = "Disable colored output")]
    pub no_color: bool,

    #[arg(short, long, global = true, help = "Enable verbose logging")]
    pub verbose: bool,

    #[arg(
        long,
        global = true,
        value_hint = ValueHint::FilePath,
        help = "Path to config file"
    )]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    #[command(visible_alias = "s", about = "Scan files and print TODO matches")]
    Scan(ScanArgs),
    #[command(visible_alias = "l", visible_alias = "ls", about = "List TODO matches")]
    List(ListArgs),
    #[command(visible_alias = "t", about = "Manage configured TODO tags")]
    Tags(TagsArgs),
    #[command(about = "Create a default configuration file")]
    Init(InitArgs),
    #[command(about = "Show summary stats for TODO matches")]
    Stats(StatsArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ScanArgs {
    #[arg(value_hint = ValueHint::AnyPath, help = "Path to scan (defaults to current directory)")]
    pub path: Option<PathBuf>,
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "Tags to search for (comma-separated)"
    )]
    pub tags: Option<Vec<String>>,
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "File patterns to include (glob patterns, comma-separated)"
    )]
    pub include: Option<Vec<String>>,
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "File patterns to exclude (glob patterns, comma-separated)"
    )]
    pub exclude: Option<Vec<String>>,
    #[arg(long, help = "Output results in JSON format")]
    pub json: bool,
    #[arg(long, help = "Print flat output without grouping by file")]
    pub flat: bool,
    #[arg(
        short,
        long,
        default_value = "0",
        help = "Limit directory traversal depth"
    )]
    pub depth: usize,
    #[arg(long, help = "Follow symlinks when scanning")]
    pub follow_links: bool,
    #[arg(long, help = "Include hidden files and directories")]
    pub hidden: bool,
    #[arg(long, help = "Ignore case when matching tags")]
    pub ignore_case: bool,
    #[arg(long, help = "Allow tags without a trailing colon")]
    pub no_require_colon: bool,
    #[arg(long, default_value = "file", help = "Sort order for results")]
    pub sort: SortOrder,
    #[arg(long, help = "Group output by tag")]
    pub group_by_tag: bool,
}

impl Default for ScanArgs {
    fn default() -> Self {
        Self {
            path: None,
            tags: None,
            include: None,
            exclude: None,
            json: false,
            flat: false,
            depth: 0,
            follow_links: false,
            hidden: false,
            ignore_case: false,
            no_require_colon: false,
            sort: SortOrder::File,
            group_by_tag: false,
        }
    }
}

#[derive(Args, Debug, Clone, Default)]
pub struct ListArgs {
    #[arg(value_hint = ValueHint::AnyPath, help = "Path to scan (defaults to current directory)")]
    pub path: Option<PathBuf>,
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "Tags to search for (comma-separated)"
    )]
    pub tags: Option<Vec<String>>,
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "File patterns to include (glob patterns, comma-separated)"
    )]
    pub include: Option<Vec<String>>,
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "File patterns to exclude (glob patterns, comma-separated)"
    )]
    pub exclude: Option<Vec<String>>,
    #[arg(long, help = "Output results in JSON format")]
    pub json: bool,
    #[arg(long, help = "Filter results by a specific tag")]
    pub filter: Option<String>,
    #[arg(long, help = "Ignore case when matching tags")]
    pub ignore_case: bool,
    #[arg(long, help = "Allow tags without a trailing colon")]
    pub no_require_colon: bool,
}

#[derive(Args, Debug, Clone)]
pub struct TagsArgs {
    #[arg(long, help = "Show tags in JSON format")]
    pub json: bool,
    #[arg(long, help = "Add a new tag to the configuration")]
    pub add: Option<String>,
    #[arg(long, help = "Remove a tag from the configuration")]
    pub remove: Option<String>,
    #[arg(long, help = "Reset tags to defaults")]
    pub reset: bool,
}

#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    #[arg(
        long,
        default_value = "json",
        help = "Configuration format: json or yaml"
    )]
    pub format: ConfigFormat,
    #[arg(short, long, help = "Overwrite the config file if it exists")]
    pub force: bool,
}

#[derive(Args, Debug, Clone)]
pub struct StatsArgs {
    #[arg(value_hint = ValueHint::AnyPath, help = "Path to scan (defaults to current directory)")]
    pub path: Option<PathBuf>,
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "Tags to search for (comma-separated)"
    )]
    pub tags: Option<Vec<String>>,
    #[arg(long, help = "Output results in JSON format")]
    pub json: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum SortOrder {
    #[value(name = "file", help = "Sort by file path")]
    #[default]
    File,
    #[value(name = "line", help = "Sort by line number")]
    Line,
    #[value(name = "priority", help = "Sort by tag priority")]
    Priority,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum ConfigFormat {
    #[default]
    #[value(name = "json", help = "Generate JSON config")]
    Json,
    #[value(name = "yaml", help = "Generate YAML config")]
    Yaml,
}

impl Cli {
    pub fn get_command(&self) -> Commands {
        self.command
            .clone()
            .unwrap_or_else(|| Commands::Scan(ScanArgs::default()))
    }
}

impl From<ScanArgs> for ListArgs {
    fn from(scan: ScanArgs) -> Self {
        Self {
            path: scan.path,
            tags: scan.tags,
            include: scan.include,
            exclude: scan.exclude,
            json: scan.json,
            filter: None,
            ignore_case: scan.ignore_case,
            no_require_colon: scan.no_require_colon,
        }
    }
}

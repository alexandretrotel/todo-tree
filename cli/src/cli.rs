use clap::{Args, Parser, Subcommand, ValueHint};
use std::path::PathBuf;

/// A CLI tool to find and display TODO-style comments in your codebase
///
/// Similar to the VS Code "Todo Tree" extension, this tool recursively scans
/// directories for comments containing TODO-style tags and displays them
/// in a tree view grouped by file.
#[derive(Parser, Debug)]
#[command(
    name = "todo-tree",
    author,
    version,
    about,
    long_about = None,
)]
pub struct Cli {
    /// The command to execute
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Global options that apply to all commands
    #[command(flatten)]
    pub global: GlobalOptions,
}

/// Global options available for all commands
#[derive(Args, Debug, Clone)]
pub struct GlobalOptions {
    /// Disable colored output
    #[arg(long, global = true, env = "NO_COLOR")]
    pub no_color: bool,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Path to a custom config file
    #[arg(long, global = true, value_hint = ValueHint::FilePath)]
    pub config: Option<PathBuf>,
}

/// Available commands for the todo-tree CLI
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Scan directories for TODO-style comments (default command)
    #[command(visible_alias = "s")]
    Scan(ScanArgs),

    /// List all TODO-style comments in a flat format
    #[command(visible_alias = "l", visible_alias = "ls")]
    List(ListArgs),

    /// Show or manage configured tags
    #[command(visible_alias = "t")]
    Tags(TagsArgs),

    /// Initialize a new .todorc config file
    Init(InitArgs),

    /// Show statistics about TODOs in the codebase
    Stats(StatsArgs),
}

/// Arguments for the scan command
#[derive(Args, Debug, Clone)]
pub struct ScanArgs {
    /// Directory or file to scan (defaults to current directory)
    #[arg(value_hint = ValueHint::AnyPath)]
    pub path: Option<PathBuf>,

    /// Tags to search for (comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    pub tags: Option<Vec<String>>,

    /// File patterns to include (glob patterns, comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    pub include: Option<Vec<String>>,

    /// File patterns to exclude (glob patterns, comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    pub exclude: Option<Vec<String>>,

    /// Output results in JSON format
    #[arg(long)]
    pub json: bool,

    /// Output results in flat format (no tree structure)
    #[arg(long)]
    pub flat: bool,

    /// Maximum depth to scan (0 = unlimited)
    #[arg(short, long, default_value = "0")]
    pub depth: usize,

    /// Follow symbolic links
    #[arg(long)]
    pub follow_links: bool,

    /// Include hidden files and directories
    #[arg(long)]
    pub hidden: bool,

    /// Ignore case when matching tags (matches TODO, todo, Todo, etc.)
    #[arg(long)]
    pub ignore_case: bool,

    /// Don't require colon after tag (allow "TODO something" without colon)
    #[arg(long)]
    pub no_require_colon: bool,

    /// Sort results by: file, tag, line
    #[arg(long, default_value = "file")]
    pub sort: SortOrder,

    /// Group results by tag instead of by file
    #[arg(long)]
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

/// Arguments for the list command
#[derive(Args, Debug, Clone, Default)]
pub struct ListArgs {
    /// Directory or file to scan (defaults to current directory)
    #[arg(value_hint = ValueHint::AnyPath)]
    pub path: Option<PathBuf>,

    /// Tags to search for (comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    pub tags: Option<Vec<String>>,

    /// File patterns to include (glob patterns, comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    pub include: Option<Vec<String>>,

    /// File patterns to exclude (glob patterns, comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    pub exclude: Option<Vec<String>>,

    /// Output results in JSON format
    #[arg(long)]
    pub json: bool,

    /// Filter by specific tag
    #[arg(long)]
    pub filter: Option<String>,

    /// Ignore case when matching tags (matches TODO, todo, Todo, etc.)
    #[arg(long)]
    pub ignore_case: bool,

    /// Don't require colon after tag (allow "TODO something" without colon)
    #[arg(long)]
    pub no_require_colon: bool,
}

/// Arguments for the tags command
#[derive(Args, Debug, Clone)]
pub struct TagsArgs {
    /// Show tags in JSON format
    #[arg(long)]
    pub json: bool,

    /// Add a new tag to the configuration
    #[arg(long)]
    pub add: Option<String>,

    /// Remove a tag from the configuration
    #[arg(long)]
    pub remove: Option<String>,

    /// Reset tags to defaults
    #[arg(long)]
    pub reset: bool,
}

/// Arguments for the init command
#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    /// Configuration format: json or yaml
    #[arg(long, default_value = "json")]
    pub format: ConfigFormat,

    /// Force overwrite if config file exists
    #[arg(short, long)]
    pub force: bool,
}

/// Arguments for the stats command
#[derive(Args, Debug, Clone)]
pub struct StatsArgs {
    /// Directory or file to scan (defaults to current directory)
    #[arg(value_hint = ValueHint::AnyPath)]
    pub path: Option<PathBuf>,

    /// Tags to search for (comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    pub tags: Option<Vec<String>>,

    /// Output results in JSON format
    #[arg(long)]
    pub json: bool,
}

/// Sort order for results
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum SortOrder {
    /// Sort by file path
    #[default]
    File,
    /// Sort by line number
    Line,
    /// Sort by priority (based on tag type)
    Priority,
}

/// Configuration format for init command
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum ConfigFormat {
    #[default]
    Json,
    Yaml,
}

impl Cli {
    /// Parse CLI arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Get the effective command, defaulting to Scan if none specified
    pub fn get_command(&self) -> Commands {
        self.command
            .clone()
            .unwrap_or_else(|| Commands::Scan(ScanArgs::default()))
    }
}

/// Convert ScanArgs to ListArgs for the list command
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scan_command() {
        let cli = Cli::parse_from(["todo-tree", "scan", "--tags", "TODO,FIXME"]);

        match cli.command {
            Some(Commands::Scan(args)) => {
                assert_eq!(
                    args.tags,
                    Some(vec!["TODO".to_string(), "FIXME".to_string()])
                );
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn test_parse_scan_with_path() {
        let cli = Cli::parse_from(["todo-tree", "scan", "./src"]);

        match cli.command {
            Some(Commands::Scan(args)) => {
                assert_eq!(args.path, Some(PathBuf::from("./src")));
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn test_parse_list_command() {
        let cli = Cli::parse_from(["todo-tree", "list", "--json"]);

        match cli.command {
            Some(Commands::List(args)) => {
                assert!(args.json);
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_parse_tags_command() {
        let cli = Cli::parse_from(["todo-tree", "tags"]);

        assert!(matches!(cli.command, Some(Commands::Tags(_))));
    }

    #[test]
    fn test_parse_no_color() {
        let cli = Cli::parse_from(["todo-tree", "--no-color", "scan"]);

        assert!(cli.global.no_color);
    }

    #[test]
    fn test_parse_include_exclude() {
        let cli = Cli::parse_from([
            "todo-tree",
            "scan",
            "--include",
            "*.rs,*.py",
            "--exclude",
            "target/**,node_modules/**",
        ]);

        match cli.command {
            Some(Commands::Scan(args)) => {
                assert_eq!(
                    args.include,
                    Some(vec!["*.rs".to_string(), "*.py".to_string()])
                );
                assert_eq!(
                    args.exclude,
                    Some(vec!["target/**".to_string(), "node_modules/**".to_string()])
                );
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn test_default_command_is_scan() {
        let cli = Cli::parse_from(["todo-tree"]);

        match cli.get_command() {
            Commands::Scan(_) => {}
            _ => panic!("Expected default to be Scan command"),
        }
    }

    #[test]
    fn test_parse_init_command() {
        let cli = Cli::parse_from(["todo-tree", "init", "--format", "yaml", "--force"]);

        match cli.command {
            Some(Commands::Init(args)) => {
                assert_eq!(args.format, ConfigFormat::Yaml);
                assert!(args.force);
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_sort_order() {
        let cli = Cli::parse_from(["todo-tree", "scan", "--sort", "priority"]);

        match cli.command {
            Some(Commands::Scan(args)) => {
                assert_eq!(args.sort, SortOrder::Priority);
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn test_scan_args_from_list_args() {
        let scan = ScanArgs {
            path: Some(PathBuf::from("./src")),
            tags: Some(vec!["TODO".to_string()]),
            json: true,
            ..Default::default()
        };

        let list: ListArgs = scan.into();
        assert_eq!(list.path, Some(PathBuf::from("./src")));
        assert_eq!(list.tags, Some(vec!["TODO".to_string()]));
        assert!(list.json);
    }

    #[test]
    fn test_parse_verbose_flag() {
        let cli = Cli::parse_from(["todo-tree", "-v", "scan"]);
        assert!(cli.global.verbose);
    }

    #[test]
    fn test_parse_config_path() {
        let cli = Cli::parse_from(["todo-tree", "--config", "/path/to/config.json", "scan"]);
        assert_eq!(
            cli.global.config,
            Some(PathBuf::from("/path/to/config.json"))
        );
    }

    #[test]
    fn test_parse_list_with_filter() {
        let cli = Cli::parse_from(["todo-tree", "list", "--filter", "TODO"]);

        match cli.command {
            Some(Commands::List(args)) => {
                assert_eq!(args.filter, Some("TODO".to_string()));
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_parse_stats_command() {
        let cli = Cli::parse_from(["todo-tree", "stats", "--json"]);

        match cli.command {
            Some(Commands::Stats(args)) => {
                assert!(args.json);
            }
            _ => panic!("Expected Stats command"),
        }
    }

    #[test]
    fn test_parse_stats_with_path() {
        let cli = Cli::parse_from(["todo-tree", "stats", "./src"]);

        match cli.command {
            Some(Commands::Stats(args)) => {
                assert_eq!(args.path, Some(PathBuf::from("./src")));
            }
            _ => panic!("Expected Stats command"),
        }
    }

    #[test]
    fn test_parse_tags_add() {
        let cli = Cli::parse_from(["todo-tree", "tags", "--add", "CUSTOM"]);

        match cli.command {
            Some(Commands::Tags(args)) => {
                assert_eq!(args.add, Some("CUSTOM".to_string()));
            }
            _ => panic!("Expected Tags command"),
        }
    }

    #[test]
    fn test_parse_tags_remove() {
        let cli = Cli::parse_from(["todo-tree", "tags", "--remove", "NOTE"]);

        match cli.command {
            Some(Commands::Tags(args)) => {
                assert_eq!(args.remove, Some("NOTE".to_string()));
            }
            _ => panic!("Expected Tags command"),
        }
    }

    #[test]
    fn test_parse_tags_reset() {
        let cli = Cli::parse_from(["todo-tree", "tags", "--reset"]);

        match cli.command {
            Some(Commands::Tags(args)) => {
                assert!(args.reset);
            }
            _ => panic!("Expected Tags command"),
        }
    }

    #[test]
    fn test_parse_scan_depth() {
        let cli = Cli::parse_from(["todo-tree", "scan", "--depth", "3"]);

        match cli.command {
            Some(Commands::Scan(args)) => {
                assert_eq!(args.depth, 3);
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn test_parse_scan_follow_links() {
        let cli = Cli::parse_from(["todo-tree", "scan", "--follow-links"]);

        match cli.command {
            Some(Commands::Scan(args)) => {
                assert!(args.follow_links);
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn test_parse_scan_hidden() {
        let cli = Cli::parse_from(["todo-tree", "scan", "--hidden"]);

        match cli.command {
            Some(Commands::Scan(args)) => {
                assert!(args.hidden);
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn test_parse_scan_ignore_case() {
        let cli = Cli::parse_from(["todo-tree", "scan", "--ignore-case"]);

        match cli.command {
            Some(Commands::Scan(args)) => {
                assert!(args.ignore_case);
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn test_parse_scan_flat() {
        let cli = Cli::parse_from(["todo-tree", "scan", "--flat"]);

        match cli.command {
            Some(Commands::Scan(args)) => {
                assert!(args.flat);
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn test_sort_order_line() {
        let cli = Cli::parse_from(["todo-tree", "scan", "--sort", "line"]);

        match cli.command {
            Some(Commands::Scan(args)) => {
                assert_eq!(args.sort, SortOrder::Line);
            }
            _ => panic!("Expected Scan command"),
        }
    }

    #[test]
    fn test_config_format_default() {
        assert_eq!(ConfigFormat::default(), ConfigFormat::Json);
    }

    #[test]
    fn test_sort_order_default() {
        assert_eq!(SortOrder::default(), SortOrder::File);
    }

    #[test]
    fn test_scan_args_default() {
        let args = ScanArgs::default();
        assert!(args.path.is_none());
        assert!(args.tags.is_none());
        assert!(args.include.is_none());
        assert!(args.exclude.is_none());
        assert!(!args.json);
        assert!(!args.flat);
        assert_eq!(args.depth, 0);
        assert!(!args.follow_links);
        assert!(!args.hidden);
        assert!(!args.ignore_case);
        assert_eq!(args.sort, SortOrder::File);
    }

    #[test]
    fn test_list_args_default() {
        let args = ListArgs::default();
        assert!(args.path.is_none());
        assert!(args.tags.is_none());
        assert!(args.include.is_none());
        assert!(args.exclude.is_none());
        assert!(!args.json);
        assert!(args.filter.is_none());
        assert!(!args.ignore_case);
    }

    #[test]
    fn test_scan_args_to_list_args_preserves_ignore_case() {
        let scan = ScanArgs {
            ignore_case: true,
            ..Default::default()
        };

        let list: ListArgs = scan.into();
        assert!(list.ignore_case);
    }

    #[test]
    fn test_scan_args_to_list_args_preserves_include_exclude() {
        let scan = ScanArgs {
            include: Some(vec!["*.rs".to_string()]),
            exclude: Some(vec!["target/**".to_string()]),
            ..Default::default()
        };

        let list: ListArgs = scan.into();
        assert_eq!(list.include, Some(vec!["*.rs".to_string()]));
        assert_eq!(list.exclude, Some(vec!["target/**".to_string()]));
    }
}

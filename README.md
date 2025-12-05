# Todo Tree

A command-line tool to find and display TODO-style comments in your codebase, similar to the VS Code "Todo Tree" extension.

## Features

- 🔍 **Recursive directory scanning** - Respects `.gitignore` rules automatically
- 🏷️ **Configurable tags** - TODO, FIXME, BUG, NOTE, HACK, XXX, WARN, PERF (and custom tags)
- 🌳 **Tree view output** - Beautiful hierarchical display grouped by file
- 📋 **Multiple output formats** - Tree, flat list, and JSON
- ⚙️ **Configuration file support** - `.todorc` in JSON or YAML format
- 🎨 **Colored output** - Priority-based coloring for different tag types
- 🔗 **Clickable links** - Terminal hyperlinks to file locations (where supported)
- 🚀 **Fast** - Parallel file scanning using the `ignore` crate

## Installation

### Using Cargo (Recommended)

```bash
cargo install todo-tree
```

### From Source

```bash
# Clone the repository
git clone https://github.com/alexandretrotel/todo-tree.git
cd todo-tree

# Build and install
cargo install --path .
```

## Usage

The tool provides two binary names: `todo-tree` and `tt` (alias for quick access).

### Basic Commands

```bash
# Scan current directory (default command)
tt

# Scan a specific directory
tt scan ./src

# Scan with specific tags
tt scan --tags TODO,FIXME,BUG

# List all TODOs in flat format
tt list

# Show configured tags
tt tags

# Show statistics
tt stats
```

### Command Reference

#### `scan` (default)

Scan directories for TODO-style comments and display in tree format.

```bash
tt scan [PATH] [OPTIONS]

Options:
  -t, --tags <TAGS>        Tags to search for (comma-separated)
  -i, --include <PATTERN>  File patterns to include (glob)
  -e, --exclude <PATTERN>  File patterns to exclude (glob)
  -d, --depth <N>          Maximum depth to scan (0 = unlimited)
      --json               Output in JSON format
      --flat               Output in flat format (no tree)
      --hidden             Include hidden files
      --follow-links       Follow symbolic links
      --case-sensitive     Case-sensitive tag matching
      --sort <ORDER>       Sort by: file, tag, line, priority
```

#### `list`

List all TODO items in a flat format.

```bash
tt list [PATH] [OPTIONS]

Options:
  -t, --tags <TAGS>        Tags to search for
  -i, --include <PATTERN>  File patterns to include
  -e, --exclude <PATTERN>  File patterns to exclude
      --filter <TAG>       Filter by specific tag
      --json               Output in JSON format
```

#### `tags`

Show or manage configured tags.

```bash
tt tags [OPTIONS]

Options:
      --json               Output in JSON format
      --add <TAG>          Add a new tag
      --remove <TAG>       Remove a tag
      --reset              Reset to default tags
```

#### `init`

Create a new configuration file.

```bash
tt init [OPTIONS]

Options:
      --format <FORMAT>    Config format: json or yaml [default: json]
  -f, --force              Overwrite existing config file
```

#### `stats`

Show statistics about TODOs in the codebase.

```bash
tt stats [PATH] [OPTIONS]

Options:
  -t, --tags <TAGS>        Tags to search for
      --json               Output in JSON format
```

### Global Options

These options apply to all commands:

```bash
      --no-color           Disable colored output
  -v, --verbose            Enable verbose output
      --config <FILE>      Path to custom config file
```

## Configuration

Create a `.todorc.json` or `.todorc.yaml` file in your project root:

### JSON Format (`.todorc.json`)

```json
{
  "tags": ["TODO", "FIXME", "BUG", "NOTE", "HACK", "XXX", "WARN", "PERF"],
  "include": ["*.rs", "*.py", "*.js", "*.ts"],
  "exclude": ["target/**", "node_modules/**", "dist/**"],
  "json": false,
  "flat": false,
  "no_color": false,
  "case_sensitive": false
}
```

### YAML Format (`.todorc.yaml`)

```yaml
tags:
  - TODO
  - FIXME
  - BUG
  - NOTE
  - HACK

include:
  - "*.rs"
  - "*.py"

exclude:
  - "target/**"
  - "node_modules/**"

json: false
flat: false
no_color: false
```

### Configuration Search Order

1. `.todorc` in the current directory
2. `.todorc.json` in the current directory
3. `.todorc.yaml` or `.todorc.yml` in the current directory
4. Parent directories (recursive)
5. `~/.config/todo-tree/config.json` (global config)

## Examples

### Example 1: Basic Scan

```bash
$ tt scan ./src

├── src/main.rs (2)
│   ├── [L10] TODO: Implement error handling
│   └── [L25] FIXME: This needs refactoring
├── src/lib.rs (3)
│   ├── [L5] NOTE: Public API
│   ├── [L42] TODO(alice): Add documentation
│   └── [L78] BUG: Memory leak in this function
└── src/utils.rs (1)
    └── [L15] HACK: Temporary workaround

Found 6 TODO items in 3 files (15 files scanned)
  TODO: 2, FIXME: 1, NOTE: 1, BUG: 1, HACK: 1
```

### Example 2: JSON Output

```bash
$ tt scan --json

{
  "files": [
    {
      "path": "src/main.rs",
      "items": [
        {
          "tag": "TODO",
          "message": "Implement error handling",
          "line": 10,
          "column": 5,
          "priority": "Medium"
        }
      ]
    }
  ],
  "summary": {
    "total_count": 6,
    "files_with_todos": 3,
    "files_scanned": 15,
    "tag_counts": {
      "TODO": 2,
      "FIXME": 1,
      "NOTE": 1,
      "BUG": 1,
      "HACK": 1
    }
  }
}
```

### Example 3: Filter by Tag

```bash
$ tt list --filter BUG

src/lib.rs:78:5 [BUG] Memory leak in this function
src/database.rs:142:3 [BUG] Connection not closed properly

Found 2 TODO items in 2 files (15 files scanned)
  BUG: 2
```

### Example 4: Include/Exclude Patterns

```bash
# Only scan Rust files
tt scan --include "*.rs"

# Exclude test files
tt scan --exclude "*_test.rs,tests/**"

# Combine patterns
tt scan --include "*.rs,*.py" --exclude "target/**,__pycache__/**"
```

### Example 5: Statistics

```bash
$ tt stats

TODO Statistics

  Total items:        42
  Files with TODOs:   12
  Files scanned:      156
  Avg items per file: 3.50

By Tag:
  TODO       18 ( 42.9%) ████████░░░░░░░░░░░░
  FIXME      10 ( 23.8%) █
████░░░░░░░░░░░░░░░
  BUG         6 ( 14.3%) ███░░░░░░░░░░░░░░░░░
  NOTE        5 ( 11.9%) ██░░░░░░░░░░░░░░░░░░
  HACK        3 (  7.1%) █░░░░░░░░░░░░░░░░░░░
```

## Supported Comment Styles

The tool recognizes TODO-style tags in various comment formats:

| Language | Comment Styles |
|----------|---------------|
| C, C++, Rust, Go, Java, JavaScript, TypeScript | `//`, `/* */` |
| Python, Ruby, Shell, YAML | `#` |
| HTML, XML | `<!-- -->` |
| SQL | `--`, `/* */` |
| Lisp, Clojure | `;` |
| Lua | `--` |

### Tag Formats Recognized

```rust
// TODO: Simple tag with colon
// TODO Simple tag without colon
// TODO(author): Tag with author
// todo: Case insensitive (by default)
```

## Priority Levels

Tags are assigned priority levels for sorting and coloring:

| Priority | Tags | Color |
|----------|------|-------|
| Critical | BUG, FIXME, XXX | Red |
| High | HACK, WARN, WARNING | Yellow |
| Medium | TODO, PERF | Cyan |
| Low | NOTE, INFO, IDEA | Green |

## Terminal Support

### Clickable Links

The tool generates clickable hyperlinks (OSC 8) in supported terminals:

- iTerm2
- WezTerm
- Hyper
- VS Code Terminal
- GNOME Terminal (VTE 0.50+)
- Konsole
- Alacritty

### Color Support

Colors are automatically enabled when outputting to a terminal. Use `--no-color` or set the `NO_COLOR` environment variable to disable.

## Development

### Building

```bash
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Project Structure

```
src/
├── lib.rs       # Library entry point with command implementations
├── main.rs      # Binary entry point (todo-tree)
├── bin/
│   └── tt.rs    # Binary entry point (tt alias)
├── cli.rs       # Command-line argument parsing (clap)
├── config.rs    # Configuration file handling
├── parser.rs    # Regex-based tag detection
├── printer.rs   # Output formatting (tree, flat, JSON)
└── scanner.rs   # Directory traversal (ignore crate)
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feat/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feat/amazing-feature`)
5. Open a Pull Request

## Acknowledgments

- Inspired by the [Todo Tree](https://marketplace.visualstudio.com/items?itemName=Gruntfuggly.todo-tree) VS Code extension
- Built with [clap](https://github.com/clap-rs/clap), [ignore](https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore), and [regex](https://github.com/rust-lang/regex)

## License

MIT License - see [LICENSE](LICENSE) for details.

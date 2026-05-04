# Todo Tree

A command-line tool to find and display TODO-style comments in your codebase, similar to the VS Code "Todo Tree" extension.

![Demo of Todo Tree](./assets/todo-tree.gif)

## Features

- 🔍 **Recursive directory scanning** - Respects `.gitignore` rules automatically
- 🏷️ **Configurable tags** - TODO, FIXME, BUG, NOTE, HACK, WARN, PERF, and more (and custom tags)
- 🌳 **Tree view output** - Beautiful hierarchical display grouped by file
- 📋 **Multiple output formats** - Tree, flat list, and JSON
- ⚙️ **Configuration file support** - `.todorc` in JSON or YAML format
- 🎨 **Colored output** - Priority-based coloring for different tag types
- 🔗 **Clickable links** - Terminal hyperlinks to file locations (where supported)
- 🤖 **GitHub Action** - Automatically scan PRs and post TODO summaries as comments

## Installation

### Using Cargo (Recommended)

```bash
cargo install todo-tree
```

### From Source

```bash
# Clone the repository
git clone https://github.com/atrtde/todo-tree.git
cd todo-tree

# Build and install
cargo install --path .
```

### Using Homebrew (macOS/Linux)

```bash
brew tap atrtde/todo-tree
brew install todo-tree
```

### NixOS (Flakes)

#### Try before you install

```bash
# runs the default todo-tree command
nix run github:atrtde/todo-tree

# create a shell with the command available (with nix-output-monitor)
nom shell github:atrtde/todo-tree
tt tags

# or, just normal nix
nix shell github:atrtde/todo-tree
tt scan ~/projects/todo-tree --tags FIXME
```

**Note:** If you haven't enabled the experimental Nix command and flakes features, you need to pass `--extra-experimental-features "nix-command flakes"` to the command. See the [Nix command wiki](https://nixos.wiki/wiki/Nix_command) for more details.

#### Install for your system

```nix
# flake.nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    todo-tree.url = "github:atrtde/todo-tree";
  };

  outputs = { self, nixpkgs, todo-tree, ... }: {
    nixosConfigurations.my-host = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [ ./configuration.nix ];
      specialArgs = { inherit todo-tree; };
    };
  };
}

# configuration.nix
{ pkgs, todo-tree, ... }:

{
  environment.systemPackages = [
    todo-tree.packages.${pkgs.stdenv.hostPlatform.system}.todo-tree
  ];
}
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

# Create a GitHub Actions workflow
tt workflow init
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
  "ignore_case": false,
  "require_colon": true
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

## Tag Matching Rules

By default, todo-tree requires tags to be **UPPERCASE** and followed by a **colon**:

```rust
// TODO: This will be found ✓
// FIXME: This will be found ✓
// BUG: This will be found ✓

// todo: This will NOT be found (lowercase) ✗
// TODO This will NOT be found (no colon) ✗
// Todo: This will NOT be found (mixed case) ✗
```

**Optional author/assignee syntax** (still works with colon):
```rust
// TODO(john): Assigned to john ✓
// FIXME(team): Needs team review ✓
```

### Flexible Matching Options

You can customize the matching behavior with CLI flags:

```bash
# Ignore case when matching (matches TODO, todo, Todo, etc.)
tt scan --ignore-case

# Allow tags without colon (matches "TODO something")
tt scan --no-require-colon

# Use both options together (most flexible, like v0.2.x behavior)
tt scan --ignore-case --no-require-colon
```

Or set these options in your `.todorc.json`:

```json
{
  "ignore_case": true,
  "require_colon": false
}
```

### Why These Defaults?

The strict defaults (uppercase + colon required) significantly reduce false positives.

These defaults align with most coding conventions and help you find **intentional TODO comments**, not accidental matches.

## GitHub Actions

Generate a workflow file at `.github/workflows/todo-tree.yml`:

```bash
tt workflow init
```

This creates a pull request workflow that checks out the repository and runs `atrtde/todo-tree-action@v1.0.3` by default.

Use `--force` to overwrite an existing workflow, `--path` to write the template elsewhere, or `--action` to override the generated action ref:

```bash
tt workflow init --force
tt workflow init --path .github/workflows/custom-todo-tree.yml
tt workflow init --action atrtde/todo-tree-action@main
```

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
- Ghostty

### Color Support

Colors are automatically enabled when outputting to a terminal. Use `--no-color` or set the `NO_COLOR` environment variable to disable.

## Related Projects

### [todo-tree-action](https://github.com/atrtde/todo-tree-action)

A GitHub Action that automatically scans your pull requests for TODO comments and posts a summary as a PR comment. Features include:
- Scan only changed files in PRs
- Filter to show only NEW TODOs (not in base branch)
- Automatic PR comment with formatted results
- Full configuration support

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

Inspired by the [Todo Tree](https://marketplace.visualstudio.com/items?itemName=Gruntfuggly.todo-tree) VS Code extension

## License

GPL-3.0-or-later

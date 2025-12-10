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
- 🧩 **Editor extensions** - Integrates with Zed via slash commands

## Installation

### Using Cargo (Recommended)

```bash
cargo install todo-tree
```

### NixOS (Flakes)

#### Try before you install

```bash
# runs the default todo-tree command
nix run github:alexandretrotel/todo-tree

# create a shell with the command available (with nix-output-monitor)
nom shell github:alexandretrotel/todo-tree
tt tags

# or, just normal nix
nix shell github:alexandretrotel/todo-tree
tt scan ~/projects/todo-tree --tags FIXME
```

**Note:** If you haven't enabled the experimental Nix command and flakes features, you need to pass `--extra-experimental-features "nix-command flakes"` to the command. See the [Nix command wiki](https://nixos.wiki/wiki/Nix_command) for more details.

#### Install for your system

```nix
# flake.nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    todo-tree.url = "github:alexandretrotel/todo-tree";
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
| Critical | BUG, FIXME, ERROR | Red |
| High | HACK, WARN, WARNING, FIX | Yellow |
| Medium | TODO, WIP, MAYBE | Cyan |
| Low | NOTE, XXX, INFO, DOCS, PERF, TEST, IDEA | Green |

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

## Extensions

### Zed Editor

The [zed-todo-tree](https://github.com/alexandretrotel/zed-todo-tree) extension integrates TODO scanning directly into Zed Assistant using slash commands.

See the [zed-todo-tree repository](https://github.com/alexandretrotel/zed-todo-tree) for installation instructions, usage details, and required capabilities.

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

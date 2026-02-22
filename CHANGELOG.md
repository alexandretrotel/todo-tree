# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## todo-tree-0.4.0

### Breaking Changes
- **Core API rename**: `Summary` is now `ScanSummary` and `ScanResult.summary` now uses `ScanSummary`.
- **License change**: Project license changed to GPLv3.

### Added
- **Core parser module** with exported `DEFAULT_REGEX`.
- **CLI display utilities** for color handling.

### Changed
- Refactored CLI scanner, parser, commands, and printer into smaller modules.
- Switched config handling dependencies to `yaml_serde` and `directories-next`.
- Updated CI/release workflows and README metadata/links.

### Removed
- Removed example `.todorc` files from the repository.
- Removed unused test dependencies and test files.

## todo-tree-0.3.0

### Breaking Changes

- **Default scanning now requires uppercase tags with colon**: By default, only `TODO:` format matches, not `todo:` or `TODO ` (without colon). This significantly reduces false positives in real-world codebases.
- **Case-sensitive matching is now the default**: Tags must be uppercase (TODO, FIXME, BUG) to match. Use `--ignore-case` to restore the old behavior.
- **Removed `::` from default comment markers**: Prevents false positives in Rust, C++, and other languages where `::` is used as a scope resolution operator (e.g., `std::io::Error` no longer matches the ERROR tag).

### Added

- **New `--ignore-case` flag**: Ignore case when matching tags (matches TODO, todo, Todo, etc.)
- **New `--no-require-colon` flag**: Allow tags without colon (e.g., `TODO something` without `:`)
- **New `require_colon` and `ignore_case` config options**: Control matching behavior in `.todorc` configuration files
- **Enhanced configuration**: Options can be set in `.todorc.json` or `.todorc.yaml` files
- **Comprehensive test suite**: Added 15+ new tests to prevent false positives

### Fixed

- **False positive**: `std::io::Error` in Rust/C++ code no longer matches ERROR tag
- **False positive**: `std::error` in C++ namespace no longer matches ERROR tag
- **False positive**: Variable names like `ERROR_CODE` no longer match ERROR tag
- **False positive**: Prose like "this is an error" no longer matches ERROR tag
- **False positive**: `Result<T, Error>` in Rust type definitions no longer matches ERROR tag

### Documentation

- Updated README with new scanning behavior and examples
- Added CHANGELOG to track version history
- Added migration guide for users upgrading from 0.2.x
- Improved documentation of default regex pattern

### Tests

- Added test for Rust scope resolution operator (`std::io::Error`)
- Added test for scope resolution with `::` operator
- Added test for C++ namespace resolution  
- Added tests for require-colon behavior
- Added tests for case-sensitive default behavior
- Added tests for variable names containing tag words
- Added tests for mixed-case tag matching

### Migration from 0.2.x to 0.3.0

If you want the old behavior (case-insensitive, no colon required), you have two options:

**Option 1: Command-line flags**
```bash
tt scan --ignore-case --no-require-colon
```

**Option 2: Configuration file** (`.todorc.json`)
```json
{
  "ignore_case": true,
  "require_colon": false
}
```

**Option 3: Configuration file** (`.todorc.yaml`)
```yaml
ignore_case: true
require_colon: false
```

## todo-tree-0.2.1

### Fixed
- Minor bug fixes and performance improvements
- Updated dependencies

### Changed
- Improved error messages
- Better handling of edge cases

## todo-tree-0.2.0

### Added
- Initial public release
- Tree and list output formats
- JSON output support
- Configuration file support (`.todorc.json`, `.todorc.yaml`)
- Configurable tags (TODO, FIXME, BUG, NOTE, HACK, etc.)
- Recursive directory scanning
- `.gitignore` respect
- Priority-based coloring
- Clickable terminal links (OSC 8)
- Multiple comment style support
- Statistics command
- Tags management commands
- Homebrew installation support
- Cargo installation support
- NixOS Flakes support

### Features
- Recursive directory scanning with `.gitignore` support
- Configurable tags with priority levels
- Beautiful tree view output
- Multiple output formats (tree, flat, JSON)
- Configuration file support
- Colored output with priority-based coloring
- Clickable links in supported terminals
- Fast parallel scanning
- Statistics and summary views

---

## Core Library Changelog

### todo-tree-core-0.3.0

#### Breaking Changes

- Changed default tag matching to require uppercase + colon
- Removed `::` from default comment markers to prevent false positives
- Config now uses `ignore_case` instead of `case_sensitive` for clearer semantics

#### Added

- New `require_colon` and `ignore_case` parameters in parser configuration
- Enhanced regex pattern builder with colon requirement option
- New `TodoParser::with_options()` method for full configuration control

#### Deprecated

- `TodoParser::with_regex()` is deprecated in favor of `with_options()`

### todo-tree-core-0.2.1

#### Fixed
- Minor type definition improvements
- Better priority handling

### todo-tree-core-0.2.0

#### Added
- Core types: `TodoItem`, `FileResult`, `ScanResult`, `Summary`
- Priority levels: Critical, High, Medium, Low
- Tag definitions with 17 default tags
- Extensible tag system
- Serialization support with serde

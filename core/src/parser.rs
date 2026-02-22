/// Default regex pattern for matching TODO-style tags in comments.
///
/// This pattern is inspired by the VSCode Todo Tree extension and matches tags
/// that appear after common comment markers.
///
/// Pattern breakdown:
/// - `(//|#|<!--|;|/\*|\*|--)`  - Comment markers for most languages
/// - `\s*`                       - Optional whitespace after comment marker
/// - `($TAGS)`                   - The tag to match (placeholder, replaced at runtime)
/// - `(?:\(([^)]+)\))?`          - Optional author in parentheses
/// - `:`                         - Required colon after tag
/// - `(.*)`                      - The message
///
/// Supported comment syntaxes:
///   //    - C, C++, Java, JavaScript, TypeScript, Rust, Go, Swift, Kotlin
///   #     - Python, Ruby, Shell, YAML, TOML
///   /*    - C-style block comments
///   *     - Block comment continuation lines
///   <!--  - HTML, XML, Markdown comments
///   --    - SQL, Lua, Haskell, Ada
///   ;     - Lisp, Clojure, Assembly, INI files
///   %     - LaTeX, Erlang, MATLAB, Prolog
///   """   - Python docstrings
///   '''   - Python docstrings
///   REM   - Batch files
///
/// Note: `::` was removed from default comment markers to prevent false positives
/// in Rust, C++, and other languages where `::` is used as a scope resolution operator
/// (e.g., `std::io::Error`).
pub const DEFAULT_REGEX: &str =
    r#"(//|#|<!--|;|/\*|\*|--|%|"""|'''|REM\s)\s*($TAGS)(?:\(([^)]+)\))?:(.*)"#;

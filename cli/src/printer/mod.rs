pub mod flat;
pub mod json;
pub mod options;
pub mod summary;
pub mod tree;
pub mod utils;

use flat::print_flat;
use json::print_json;
pub use options::{OutputFormat, PrintOptions};
use std::io::{self, Write};
use summary::print_summary;
use todo_tree_core::ScanResult;
use tree::print_tree;

pub struct Printer {
    options: PrintOptions,
}

impl Printer {
    pub fn new(options: PrintOptions) -> Self {
        if !options.colored {
            colored::control::set_override(false);
        }
        Self { options }
    }

    pub fn print(&self, result: &ScanResult) -> io::Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        self.print_to(&mut handle, result)
    }

    pub fn print_to<W: Write>(&self, writer: &mut W, result: &ScanResult) -> io::Result<()> {
        match self.options.format {
            OutputFormat::Tree => print_tree(writer, result, &self.options)?,
            OutputFormat::Flat => print_flat(writer, result, &self.options)?,
            OutputFormat::Json => print_json(writer, result, &self.options)?,
        }

        if self.options.show_summary && self.options.format != OutputFormat::Json {
            writeln!(writer)?;
            print_summary(writer, result, &self.options)?;
        }

        Ok(())
    }
}

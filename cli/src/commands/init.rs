use crate::cli::{self, ConfigFormat};
use crate::config::Config;
use anyhow::Result;
use std::path::PathBuf;

pub fn run(args: cli::InitArgs) -> Result<()> {
    let filename = match args.format {
        ConfigFormat::Json => ".todorc.json",
        ConfigFormat::Yaml => ".todorc.yaml",
    };

    let path = PathBuf::from(filename);

    if path.exists() && !args.force {
        anyhow::bail!(
            "Config file {} already exists. Use --force to overwrite.",
            filename
        );
    }

    let config = Config::new();
    config.save(&path)?;

    println!("Created configuration file: {}", filename);
    println!("\nYou can customize the following settings:");
    println!("  - tags: List of tags to search for");
    println!("  - include: File patterns to include");
    println!("  - exclude: File patterns to exclude");
    println!("  - json: Default to JSON output");
    println!("  - flat: Default to flat output");

    Ok(())
}

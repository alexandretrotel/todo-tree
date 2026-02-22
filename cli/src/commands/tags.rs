use super::load_config;
use crate::{cli, parser::priority_to_color};
use anyhow::Result;
use todo_tree_core::Priority;
use todo_tree_core::tags::default_tag_names;

pub fn run(args: cli::TagsArgs, global: &cli::GlobalOptions) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let mut config = load_config(&current_dir, global.config.as_deref())?;

    if let Some(new_tag) = &args.add {
        if !config.tags.iter().any(|t| t.eq_ignore_ascii_case(new_tag)) {
            config.tags.push(new_tag.to_uppercase());
            super::save_config(&config)?;
            println!("Added tag: {}", new_tag.to_uppercase());
        } else {
            println!("Tag already exists: {}", new_tag);
        }
        return Ok(());
    }

    if let Some(remove_tag) = &args.remove {
        let original_len = config.tags.len();
        config.tags.retain(|t| !t.eq_ignore_ascii_case(remove_tag));
        if config.tags.len() < original_len {
            super::save_config(&config)?;
            println!("Removed tag: {}", remove_tag);
        } else {
            println!("Tag not found: {}", remove_tag);
        }
        return Ok(());
    }

    if args.reset {
        config.tags = default_tag_names();
        super::save_config(&config)?;
        println!("Tags reset to defaults");
        return Ok(());
    }

    if args.json {
        let json = serde_json::json!({
            "tags": config.tags,
            "default_tags": default_tag_names(),
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        use colored::Colorize;
        println!("{}", "Configured tags:".bold());
        for tag in &config.tags {
            if global.no_color {
                println!("  - {}", tag);
            } else {
                let color = priority_to_color(Priority::from_tag(tag));
                println!("  - {}", tag.color(color));
            }
        }
    }

    Ok(())
}

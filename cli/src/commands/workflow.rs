use crate::cli::{WorkflowArgs, WorkflowCommands, WorkflowInitArgs};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

const DEFAULT_WORKFLOW_PATH: &str = ".github/workflows/todo-tree.yml";
const ACTION_VERSION: &str = "v1.0.3";

pub fn run(args: WorkflowArgs) -> Result<()> {
    match args.command {
        WorkflowCommands::Init(args) => init(args),
    }
}

fn init(args: WorkflowInitArgs) -> Result<()> {
    let action = args.action.unwrap_or_else(default_action_ref);
    let path = args
        .path
        .unwrap_or_else(|| PathBuf::from(DEFAULT_WORKFLOW_PATH));

    validate_action_ref(&action)?;
    write_workflow_template(&path, args.force, &action)?;

    println!("Created workflow file: {}", path.display());
    println!("The workflow will run on pull requests using {action}.");

    Ok(())
}

fn default_action_ref() -> String {
    format!("alexandretrotel/todo-tree-action@{ACTION_VERSION}")
}

fn validate_action_ref(action: &str) -> Result<()> {
    let Some((repo, reference)) = action.split_once('@') else {
        anyhow::bail!(
            "Invalid action reference {:?}. Expected format: owner/repo@ref",
            action
        );
    };

    let mut repo_parts = repo.split('/');
    let owner = repo_parts.next().unwrap_or_default();
    let name = repo_parts.next().unwrap_or_default();

    if owner.is_empty()
        || name.is_empty()
        || reference.is_empty()
        || repo_parts.next().is_some()
        || action.contains('\n')
        || action.contains('\r')
    {
        anyhow::bail!(
            "Invalid action reference {:?}. Expected format: owner/repo@ref",
            action
        );
    }

    Ok(())
}

fn workflow_template(action: &str) -> String {
    format!(
        r#"name: todo-tree

on:
  pull_request:

permissions:
  contents: read
  pull-requests: write

jobs:
  todo-tree:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v6

      - name: Scan TODOs
        uses: {action}
        with:
          github-token: ${{{{ secrets.GITHUB_TOKEN }}}}
          changed-only: true
          new-only: true
"#
    )
}

fn write_workflow_template(path: &Path, force: bool, action: &str) -> Result<()> {
    if path.exists() && !force {
        anyhow::bail!(
            "Workflow file {} already exists. Use --force to overwrite.",
            path.display()
        );
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    std::fs::write(path, workflow_template(action))
        .with_context(|| format!("Failed to write workflow file: {}", path.display()))?;

    Ok(())
}

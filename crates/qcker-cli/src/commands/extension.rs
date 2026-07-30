use clap::{Args, Subcommand};
use std::path::Path;

use qcker_engine::extension::manager::ExtensionManager;

#[derive(Args)]
pub struct ExtensionArgs {
    #[command(subcommand)]
    command: ExtensionCommand,
}

#[derive(Subcommand)]
pub enum ExtensionCommand {
    Ls,
    Install {
        path: String,
    },
    Uninstall {
        id: String,
    },
    Enable {
        id: String,
    },
    Disable {
        id: String,
    },
    Info {
        id: String,
    },
}

pub fn execute(args: ExtensionArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let mut manager = ExtensionManager::new(data_dir.to_path_buf());
    manager.init()?;

    match args.command {
        ExtensionCommand::Ls => {
            let extensions = manager.list();

            if format == "json" {
                let output: Vec<serde_json::Value> = extensions
                    .iter()
                    .map(|ext| {
                        serde_json::json!({
                            "id": ext.metadata.id,
                            "name": ext.metadata.name,
                            "version": ext.metadata.version,
                            "status": format!("{:?}", ext.status),
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                if extensions.is_empty() {
                    println!("No extensions installed.");
                    return Ok(());
                }

                println!("{:<30} {:<15} {:<10} {:<15}", "ID", "NAME", "VERSION", "STATUS");
                for ext in &extensions {
                    println!(
                        "{:<30} {:<15} {:<10} {:<15}",
                        ext.metadata.id,
                        ext.metadata.name,
                        ext.metadata.version,
                        format!("{:?}", ext.status)
                    );
                }
            }
        }
        ExtensionCommand::Install { path } => {
            manager.install(&path)?;

            if format == "json" {
                let output = serde_json::json!({
                    "path": path,
                    "installed": true,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Extension installed from {}", path);
            }
        }
        ExtensionCommand::Uninstall { id } => {
            manager.uninstall(&id)?;

            if format == "json" {
                let output = serde_json::json!({
                    "id": id,
                    "uninstalled": true,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Extension {} uninstalled", id);
            }
        }
        ExtensionCommand::Enable { id } => {
            manager.enable(&id)?;

            if format == "json" {
                let output = serde_json::json!({
                    "id": id,
                    "enabled": true,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Extension {} enabled", id);
            }
        }
        ExtensionCommand::Disable { id } => {
            manager.disable(&id)?;

            if format == "json" {
                let output = serde_json::json!({
                    "id": id,
                    "disabled": true,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Extension {} disabled", id);
            }
        }
        ExtensionCommand::Info { id } => {
            if let Some(ext) = manager.get(&id) {
                if format == "json" {
                    println!("{}", serde_json::to_string_pretty(ext)?);
                } else {
                    println!("ID:          {}", ext.metadata.id);
                    println!("Name:        {}", ext.metadata.name);
                    println!("Version:     {}", ext.metadata.version);
                    println!("Author:      {}", ext.metadata.author);
                    println!("Description: {}", ext.metadata.description);
                    println!("Status:      {:?}", ext.status);
                    println!("Path:        {}", ext.path);
                }
            } else {
                return Err(anyhow::anyhow!("Extension not found: {}", id));
            }
        }
    }

    Ok(())
}

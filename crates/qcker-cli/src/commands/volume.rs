use clap::{Args, Subcommand};
use std::path::Path;

use qcker_engine::volume::local::VolumeManager;

#[derive(Args)]
pub struct VolumeArgs {
    #[command(subcommand)]
    command: VolumeCommand,
}

#[derive(Subcommand)]
pub enum VolumeCommand {
    Create {
        name: String,

        #[arg(short, long, default_value = "local")]
        driver: String,
    },
    Rm {
        name: String,
    },
    Ls,
    Inspect {
        name: String,
    },
}

pub fn execute(args: VolumeArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let manager = VolumeManager::new(data_dir.to_path_buf());
    manager.init()?;

    match args.command {
        VolumeCommand::Create { name, driver } => {
            let volume = manager.create(&name, &driver)?;

            if format == "json" {
                let output = serde_json::json!({
                    "name": volume.name,
                    "driver": volume.driver,
                    "mountpoint": volume.mountpoint,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Volume {} created", name);
            }
        }
        VolumeCommand::Rm { name } => {
            manager.remove(&name)?;

            if format == "json" {
                let output = serde_json::json!({
                    "name": name,
                    "removed": true,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Volume {} removed", name);
            }
        }
        VolumeCommand::Ls => {
            let volumes = manager.list()?;

            if format == "json" {
                let output: Vec<serde_json::Value> = volumes
                    .iter()
                    .map(|v| {
                        serde_json::json!({
                            "name": v.name,
                            "driver": v.driver,
                            "mountpoint": v.mountpoint,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("{:<20} {:<15} {:<40}", "VOLUME NAME", "DRIVER", "MOUNTPOINT");
                for v in &volumes {
                    println!(
                        "{:<20} {:<15} {:<40}",
                        v.name,
                        v.driver,
                        v.mountpoint.display()
                    );
                }
            }
        }
        VolumeCommand::Inspect { name } => {
            let volume = manager.get(&name)?;

            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&volume)?);
            } else {
                println!("Name:       {}", volume.name);
                println!("Driver:     {}", volume.driver);
                println!("Mountpoint: {}", volume.mountpoint.display());
                println!("Created:    {}", volume.created_at);
            }
        }
    }

    Ok(())
}

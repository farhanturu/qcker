use clap::{Args, Subcommand};
use std::path::{Path, PathBuf};

use qcker_runtime::snapshot::{
    check_criu_requirements, delete_snapshot, is_criu_available, list_snapshots,
    restore_snapshot, checkpoint_container, CheckpointOptions,
};

#[derive(Args)]
pub struct SnapshotArgs {
    #[command(subcommand)]
    command: SnapshotCommand,
}

#[derive(Subcommand)]
pub enum SnapshotCommand {
    Check,
    List,
    Checkpoint {
        container_id: String,
        #[arg(long, help = "Leave container running after checkpoint")]
        leave_running: bool,
    },
    Restore {
        snapshot_path: String,
        #[arg(long, help = "New container ID (auto-generated if not specified)")]
        name: Option<String>,
    },
    Delete {
        snapshot_id: String,
    },
}

pub fn execute(args: SnapshotArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    match args.command {
        SnapshotCommand::Check => {
            println!("Checking CRIU availability...");
            let available = is_criu_available();
            println!("CRIU available: {}", available);

            if available {
                match check_criu_requirements() {
                    Ok(issues) => {
                        if issues.is_empty() {
                            println!("All CRIU requirements satisfied.");
                        } else {
                            println!("Warnings:");
                            for issue in issues {
                                println!("  - {}", issue);
                            }
                        }
                    }
                    Err(e) => eprintln!("Error checking requirements: {}", e),
                }
            } else {
                println!("Please install CRIU: apt-get install criu");
            }
        }
        SnapshotCommand::List => {
            let snapshots = list_snapshots(data_dir)?;
            if snapshots.is_empty() {
                println!("No snapshots found.");
                return Ok(());
            }

            if format == "json" {
                let output: Vec<serde_json::Value> = snapshots.iter().map(|s| {
                    serde_json::json!({
                        "id": s.id,
                        "container_id": s.container_id,
                        "created_at": s.created_at,
                        "path": s.path.to_string_lossy(),
                        "pid": s.pid,
                        "image": s.image,
                    })
                }).collect();
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("{:<40} {:<20} {:<20} {}", "SNAPSHOT ID", "CONTAINER", "CREATED", "IMAGE");
                println!("{}", "-".repeat(95));
                for s in &snapshots {
                    println!(
                        "{:<40} {:<20} {:<20} {}",
                        s.id, s.container_id, s.created_at[..19].to_string(), s.image
                    );
                }
            }
        }
        SnapshotCommand::Checkpoint { container_id, leave_running } => {
            if !is_criu_available() {
                return Err(anyhow::anyhow!("CRIU is not installed. Please install CRIU to use checkpoint."));
            }

            let options = CheckpointOptions {
                leave_running,
                ..Default::default()
            };

            let snapshot = checkpoint_container(&container_id, data_dir, &options)?;

            if format == "json" {
                let output = serde_json::json!({
                    "id": snapshot.id,
                    "container_id": snapshot.container_id,
                    "created_at": snapshot.created_at,
                    "path": snapshot.path.to_string_lossy(),
                    "pid": snapshot.pid,
                    "image": snapshot.image,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Checkpoint created: {}", snapshot.id);
                println!("Path: {}", snapshot.path.display());
                println!("Container: {}", snapshot.container_id);
            }
        }
        SnapshotCommand::Restore { snapshot_path, name } => {
            if !is_criu_available() {
                return Err(anyhow::anyhow!("CRIU is not installed. Please install CRIU to use restore."));
            }

            let path = PathBuf::from(&snapshot_path);
            if !path.exists() {
                return Err(anyhow::anyhow!("Snapshot path not found: {}", snapshot_path));
            }

            let new_id = restore_snapshot(&path, name.as_deref(), data_dir)?;

            if format == "json" {
                let output = serde_json::json!({
                    "new_container_id": new_id,
                    "snapshot_path": snapshot_path,
                    "status": "restored",
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Container restored as: {}", new_id);
                println!("From snapshot: {}", snapshot_path);
            }
        }
        SnapshotCommand::Delete { snapshot_id } => {
            delete_snapshot(&snapshot_id, data_dir)?;
            println!("Deleted snapshot: {}", snapshot_id);
        }
    }

    Ok(())
}

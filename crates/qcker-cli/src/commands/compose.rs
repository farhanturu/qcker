use clap::{Args, Subcommand};
use std::path::{Path, PathBuf};

use qcker_engine::compose::parser::ComposeFile;
use qcker_engine::compose::project::ComposeProject;

/// Manage multi-container applications with Compose
#[derive(Args)]
pub struct ComposeArgs {
    #[command(subcommand)]
    command: ComposeCommand,

    /// Compose file path
    #[arg(short = 'f', long, default_value = "docker-compose.yml")]
    file: PathBuf,

    /// Project name
    #[arg(short = 'p', long)]
    project_name: Option<String>,
}

#[derive(Subcommand)]
pub enum ComposeCommand {
    /// Create and start services
    Up {
        /// Services to start
        services: Vec<String>,

        /// Run in detached mode
        #[arg(short, long)]
        detach: bool,
    },
    /// Stop and remove services
    Down {
        /// Remove volumes
        #[arg(short, long)]
        volumes: bool,
    },
    /// List services
    Ps,
    /// Build or rebuild services
    Build {
        /// Services to build
        services: Vec<String>,
    },
    /// Pull service images
    Pull {
        /// Services to pull
        services: Vec<String>,
    },
    /// Show logs
    Logs {
        /// Services to show logs for
        services: Vec<String>,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },
}

pub fn execute(args: ComposeArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    // Parse compose file
    let compose_file = ComposeFile::parse_file(&args.file)?;

    // Get project name
    let project_name = args.project_name.unwrap_or_else(|| {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "default".to_string())
    });

    let project = ComposeProject::new(
        &project_name,
        compose_file,
        std::env::current_dir().unwrap_or_default(),
        data_dir.to_path_buf(),
    );

    match args.command {
        ComposeCommand::Up { services, detach: _ } => {
            let services_ref = if services.is_empty() {
                None
            } else {
                Some(services.as_slice())
            };
            project.up(services_ref)?;

            if format == "json" {
                let output = serde_json::json!({
                    "project": project_name,
                    "status": "started",
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Project {} started", project_name);
            }
        }
        ComposeCommand::Down { volumes } => {
            project.down(volumes)?;

            if format == "json" {
                let output = serde_json::json!({
                    "project": project_name,
                    "status": "stopped",
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Project {} stopped", project_name);
            }
        }
        ComposeCommand::Ps => {
            let statuses = project.ps()?;

            if format == "json" {
                let output: Vec<serde_json::Value> = statuses
                    .iter()
                    .map(|s| {
                        serde_json::json!({
                            "name": s.name,
                            "state": s.state,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("{:<20} {:<15}", "SERVICE", "STATE");
                for s in &statuses {
                    println!("{:<20} {:<15}", s.name, s.state);
                }
            }
        }
        ComposeCommand::Build { services } => {
            println!("Building services: {:?}", services);
            // TODO: Implement build
        }
        ComposeCommand::Pull { services } => {
            println!("Pulling services: {:?}", services);
            // TODO: Implement pull
        }
        ComposeCommand::Logs { services, follow } => {
            println!("Showing logs for: {:?}", services);
            if follow {
                println!("Following logs...");
            }
            // TODO: Implement logs
        }
    }

    Ok(())
}

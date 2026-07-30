use clap::{Args, Subcommand};
use std::path::{Path, PathBuf};

use qcker_engine::compose::parser::ComposeFile;
use qcker_engine::compose::project::ComposeProject;

#[derive(Args)]
pub struct ComposeArgs {
    #[command(subcommand)]
    command: ComposeCommand,

    #[arg(short = 'f', long, default_value = "docker-compose.yml")]
    file: PathBuf,

    #[arg(short = 'p', long)]
    project_name: Option<String>,
}

#[derive(Subcommand)]
pub enum ComposeCommand {
    Up {
        services: Vec<String>,

        #[arg(short, long)]
        detach: bool,
    },
    Down {
        #[arg(short, long)]
        volumes: bool,
    },
    Ps,
    Build {
        services: Vec<String>,
    },
    Pull {
        services: Vec<String>,
    },
    Logs {
        services: Vec<String>,

        #[arg(short, long)]
        follow: bool,
    },
}

pub fn execute(args: ComposeArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let compose_file = ComposeFile::parse_file(&args.file)?;

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
        }
        ComposeCommand::Pull { services } => {
            println!("Pulling services: {:?}", services);
        }
        ComposeCommand::Logs { services, follow } => {
            println!("Showing logs for: {:?}", services);
            if follow {
                println!("Following logs...");
            }
        }
    }

    Ok(())
}

use clap::{Args, Subcommand};
use std::fs;
use std::path::Path;


#[derive(Args)]
pub struct SystemArgs {
    #[command(subcommand)]
    command: SystemCommand,
}

#[derive(Subcommand)]
pub enum SystemCommand {
    Info,
    Prune {
        #[arg(long)]
        all: bool,
    },
}

pub fn execute(args: SystemArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    match args.command {
        SystemCommand::Info => show_info(data_dir, format),
        SystemCommand::Prune { all } => prune(data_dir, all),
    }
}

fn show_info(data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let containers_dir = data_dir.join("containers");
    let images_dir = data_dir.join("images");
    let volumes_dir = data_dir.join("volumes");

    let container_count = if containers_dir.exists() {
        std::fs::read_dir(&containers_dir)?.count()
    } else {
        0
    };

    let image_count = if images_dir.exists() {
        std::fs::read_dir(&images_dir)?.count()
    } else {
        0
    };

    let volume_count = if volumes_dir.exists() {
        std::fs::read_dir(&volumes_dir)?.count()
    } else {
        0
    };

    if format == "json" {
        let info = serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "data_dir": data_dir.display().to_string(),
            "containers": container_count,
            "images": image_count,
            "volumes": volume_count,
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
        });
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        println!("Qcker System Information");
        println!("========================");
        println!("Version:    {}", env!("CARGO_PKG_VERSION"));
        println!("Data Dir:   {}", data_dir.display());
        println!("OS:         {}", std::env::consts::OS);
        println!("Arch:       {}", std::env::consts::ARCH);
        println!("Containers: {}", container_count);
        println!("Images:     {}", image_count);
        println!("Volumes:    {}", volume_count);
    }

    Ok(())
}

fn prune(data_dir: &Path, all: bool) -> anyhow::Result<()> {
    let containers_dir = data_dir.join("containers");

    if containers_dir.exists() {
        for entry in fs::read_dir(&containers_dir)? {
            let entry = entry?;
            let state_path = entry.path().join("state.json");

            if state_path.exists() {
                let content = std::fs::read_to_string(&state_path)?;
                let state: serde_json::Value = serde_json::from_str(&content)?;

                let status = state["state"].as_str().unwrap_or("");
                if status == "Stopped" || all {
                    std::fs::remove_dir_all(entry.path())?;
                    println!("Removed container: {}", entry.file_name().to_string_lossy());
                }
            }
        }
    }

    println!("Prune complete");
    Ok(())
}

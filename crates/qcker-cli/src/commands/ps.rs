use clap::Args;
use std::fs;
use std::path::Path;

use crate::output;

#[derive(Args)]
pub struct PsArgs {
    #[arg(short, long)]
    all: bool,
}

pub fn execute(args: PsArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let containers_dir = data_dir.join("containers");

    if !containers_dir.exists() {
        output::print_container_list(&[], format);
        return Ok(());
    }

    let mut containers = Vec::new();

    for entry in fs::read_dir(&containers_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let state_path = path.join("state.json");
        if !state_path.exists() {
            continue;
        }

        let state_json = fs::read_to_string(&state_path)?;
        let container: serde_json::Value = serde_json::from_str(&state_json)?;

        let id = container["id"].as_str().unwrap_or("unknown").to_string();
        let state = container["state"].as_str().unwrap_or("unknown").to_string();
        let pid = container["pid"].as_i64().map(|p| p as i32);

        if !args.all && state != "running" {
            continue;
        }

        containers.push((id, state, pid));
    }

    output::print_container_list(&containers, format);

    Ok(())
}

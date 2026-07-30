use clap::Args;
use std::path::Path;

use crate::output;

#[derive(Args)]
pub struct StopArgs {
    pub container_id: String,

    #[arg(short, long, default_value = "10")]
    pub timeout: u32,
}

pub fn execute(args: StopArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let state_path = data_dir
        .join("containers")
        .join(&args.container_id)
        .join("state.json");

    if !state_path.exists() {
        return Err(anyhow::anyhow!("Container not found: {}", args.container_id));
    }

    let content = std::fs::read_to_string(&state_path)?;
    let state: serde_json::Value = serde_json::from_str(&content)?;

    let pid = state["pid"].as_i64().ok_or_else(|| anyhow::anyhow!("Container has no PID"))?;

    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }

    std::thread::sleep(std::time::Duration::from_secs(args.timeout as u64));

    unsafe {
        libc::kill(pid as i32, libc::SIGKILL);
    }

    let mut state: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&state_path)?)?;
    state["state"] = serde_json::json!("Stopped");
    std::fs::write(&state_path, serde_json::to_string_pretty(&state)?)?;

    if format == "json" {
        let output = serde_json::json!({
            "container": args.container_id,
            "status": "stopped",
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        output::print_success(&format!("Container {} stopped", args.container_id));
    }

    Ok(())
}

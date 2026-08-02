use clap::Args;
use std::path::Path;

use qcker_runtime::process::ContainerProcess;

#[derive(Args)]
pub struct ExecArgs {
    container_id: String,

    #[arg(trailing_var_arg = true)]
    command: Vec<String>,

    #[arg(short = 't', long)]
    terminal: bool,

    #[arg(short = 'i', long)]
    interactive: bool,
}

pub fn execute(args: ExecArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let container_state = ContainerProcess::load_state(data_dir, &args.container_id)?;

    if args.command.is_empty() {
        return Err(anyhow::anyhow!("No command specified"));
    }

    let process = ContainerProcess {
        container: container_state,
        data_dir: data_dir.to_path_buf(),
    };

    process.exec(&args.command, args.terminal, args.interactive)?;

    if format == "json" {
        let output = serde_json::json!({
            "container": args.container_id,
            "command": args.command,
            "status": "executed",
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    }

    Ok(())
}

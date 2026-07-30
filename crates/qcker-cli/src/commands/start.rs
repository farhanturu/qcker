use clap::Args;
use std::path::Path;

use crate::output;
use qcker_runtime::process::ContainerProcess;

#[derive(Args)]
pub struct StartArgs {
    container_id: String,
}

pub fn execute(args: StartArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let container = ContainerProcess::load_state(data_dir, &args.container_id)?;
    let mut process = ContainerProcess {
        container,
        data_dir: data_dir.to_path_buf(),
    };

    process.start()?;

    output::print_success(&format!("Container {} started", args.container_id));
    output::print_container_state(
        &args.container_id,
        "running",
        process.container.pid,
        format,
    );

    Ok(())
}

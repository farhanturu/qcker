use clap::Args;
use std::path::Path;

use crate::output;
use qcker_runtime::process::ContainerProcess;

#[derive(Args)]
pub struct StateArgs {
    container_id: String,
}

pub fn execute(args: StateArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let container = ContainerProcess::load_state(data_dir, &args.container_id)?;

    output::print_container_state(
        &container.id,
        &format!("{:?}", container.state).to_lowercase(),
        container.pid,
        format,
    );

    Ok(())
}

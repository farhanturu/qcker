use clap::Args;
use std::path::Path;

use crate::output;
use qcker_runtime::process::ContainerProcess;

#[derive(Args)]
pub struct DeleteArgs {
    container_id: String,

    #[arg(short, long)]
    force: bool,
}

pub fn execute(args: DeleteArgs, data_dir: &Path, _format: &str) -> anyhow::Result<()> {
    let container = ContainerProcess::load_state(data_dir, &args.container_id)?;
    let mut process = ContainerProcess {
        container,
        data_dir: data_dir.to_path_buf(),
        log_path: None,
    };

    if args.force && process.container.state == qcker_runtime::spec::ContainerState::Running {
        process.kill(nix::sys::signal::Signal::SIGKILL)?;
    }

    process.delete()?;

    output::print_success(&format!("Container {} deleted", args.container_id));

    Ok(())
}

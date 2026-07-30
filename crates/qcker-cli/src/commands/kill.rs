use clap::Args;
use nix::sys::signal::Signal;
use std::path::Path;

use crate::output;
use qcker_runtime::process::ContainerProcess;

#[derive(Args)]
pub struct KillArgs {
    container_id: String,

    #[arg(short, long, default_value = "SIGKILL")]
    signal: String,
}

pub fn execute(args: KillArgs, data_dir: &Path, _format: &str) -> anyhow::Result<()> {
    let container = ContainerProcess::load_state(data_dir, &args.container_id)?;
    let process = ContainerProcess {
        container,
        data_dir: data_dir.to_path_buf(),
    };

    let signal = match args.signal.as_str() {
        "SIGKILL" => Signal::SIGKILL,
        "SIGTERM" => Signal::SIGTERM,
        "SIGINT" => Signal::SIGINT,
        "SIGHUP" => Signal::SIGHUP,
        "SIGSTOP" => Signal::SIGSTOP,
        "SIGCONT" => Signal::SIGCONT,
        _ => return Err(anyhow::anyhow!("Unknown signal: {}", args.signal)),
    };

    process.kill(signal)?;

    output::print_success(&format!(
        "Container {} killed with {}",
        args.container_id, args.signal
    ));

    Ok(())
}

use clap::Args;
use std::path::{Path, PathBuf};

use crate::output;
use qcker_common::id;
use qcker_runtime::process::{ContainerProcess, ResourceLimits};
use qcker_runtime::spec::{LinuxConfig, NamespaceConfig, NamespaceType, OciConfig, ProcessConfig, RootConfig, UserConfig};

#[derive(Args)]
pub struct RunArgs {
    #[arg(long)]
    rootfs: PathBuf,

    #[arg(trailing_var_arg = true)]
    command: Vec<String>,

    #[arg(long)]
    name: Option<String>,

    #[arg(short = 't', long)]
    terminal: bool,

    #[arg(short = 'w', long, default_value = "/")]
    workdir: String,

    #[arg(short = 'e', long)]
    env: Vec<String>,

    #[arg(long)]
    hostname: Option<String>,

    #[arg(short, long)]
    detach: bool,

    #[arg(long, help = "CPU cores (e.g., 1.5 for 1.5 cores)")]
    cpus: Option<f64>,

    #[arg(long, help = "CPU shares (relative weight, default 1024)")]
    cpu_shares: Option<u64>,

    #[arg(short = 'm', long, help = "Memory limit in MB (e.g., 512)")]
    memory: Option<u64>,

    #[arg(long, help = "Memory + swap limit in MB")]
    memory_swap: Option<u64>,

    #[arg(long, help = "Max number of processes (default 256)")]
    pids_limit: Option<i64>,

    #[arg(long, help = "Enable GPU access")]
    gpu: bool,

    #[arg(long, help = "GPU devices to expose (e.g., /dev/nvidia0)")]
    gpu_device: Vec<String>,

    #[arg(long, help = "VRAM limit in MB")]
    vram: Option<u64>,

    #[arg(long, help = "Read-only rootfs")]
    read_only: bool,

    #[arg(long, help = "Run with root privileges (disable user namespace)")]
    privileged: bool,
}

pub fn execute(args: RunArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let container_id = args.name.unwrap_or_else(id::generate_container_id);

    let config = OciConfig {
        oci_version: "1.0.0".to_string(),
        root: RootConfig {
            path: PathBuf::from("."),
            readonly: args.read_only,
        },
        process: Some(ProcessConfig {
            terminal: args.terminal,
            user: UserConfig { uid: 0, gid: 0 },
            args: if args.command.is_empty() {
                vec!["/bin/sh".to_string()]
            } else {
                args.command
            },
            env: args.env,
            cwd: args.workdir,
            capabilities: None,
            rlimits: vec![],
            no_new_privileges: !args.privileged,
        }),
        hostname: args.hostname,
        mounts: vec![],
        linux: Some(LinuxConfig {
            namespaces: vec![
                NamespaceConfig { r#type: NamespaceType::Pid, path: None },
                NamespaceConfig { r#type: NamespaceType::Network, path: None },
                NamespaceConfig { r#type: NamespaceType::Mount, path: None },
                NamespaceConfig { r#type: NamespaceType::Uts, path: None },
                NamespaceConfig { r#type: NamespaceType::Ipc, path: None },
                NamespaceConfig { r#type: NamespaceType::Cgroup, path: None },
            ],
            resources: None,
            uid_mappings: vec![],
            gid_mappings: vec![],
            seccomp: None,
        }),
    };

    let resource_limits = ResourceLimits {
        cpu_cores: args.cpus,
        cpu_shares: args.cpu_shares,
        memory_mb: args.memory,
        memory_swap_mb: args.memory_swap,
        pids_limit: args.pids_limit,
        gpu_enabled: args.gpu,
        gpu_devices: args.gpu_device,
        vram_mb: args.vram,
    };

    let mut container = ContainerProcess::new(
        &container_id,
        &args.rootfs,
        config,
        data_dir.to_path_buf(),
    )?;

    container.create()?;

    if resource_limits.cpu_cores.is_some()
        || resource_limits.memory_mb.is_some()
        || resource_limits.pids_limit.is_some()
        || resource_limits.gpu_enabled
    {
        container.apply_resource_limits(&resource_limits)?;
    }

    container.start()?;

    output::print_container_state(
        &container_id,
        "running",
        container.container.pid,
        format,
    );

    if !args.detach {
        let exit_code = container.wait()?;
        output::print_success(&format!(
            "Container {} exited with code {}",
            container_id, exit_code
        ));
    }

    Ok(())
}

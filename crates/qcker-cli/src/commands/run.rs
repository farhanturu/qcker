use clap::Args;
use std::path::{Path, PathBuf};

use crate::output;
use qcker_common::id;
use qcker_runtime::process::{ContainerProcess, ResourceLimits};
use qcker_runtime::spec::{LinuxConfig, NamespaceConfig, NamespaceType, OciConfig, ProcessConfig, RootConfig, UserConfig};

#[derive(Args)]
pub struct RunArgs {
    #[arg(long)]
    pub rootfs: Option<PathBuf>,

    #[arg(short, long)]
    pub image: Option<String>,

    #[arg(trailing_var_arg = true)]
    pub command: Vec<String>,

    #[arg(long)]
    pub name: Option<String>,

    #[arg(short = 't', long)]
    pub terminal: bool,

    #[arg(short = 'w', long, default_value = "/")]
    pub workdir: String,

    #[arg(short = 'e', long)]
    pub env: Vec<String>,

    #[arg(long)]
    pub hostname: Option<String>,

    #[arg(short, long)]
    pub detach: bool,

    #[arg(short = 'p', long)]
    pub publish: Vec<String>,

    #[arg(short = 'v', long)]
    pub volume: Vec<String>,

    #[arg(long, default_value = "bridge")]
    pub network: String,

    #[arg(long)]
    pub dns: Vec<String>,

    #[arg(long)]
    pub cpus: Option<f64>,

    #[arg(long)]
    pub cpu_shares: Option<u64>,

    #[arg(short = 'm', long)]
    pub memory: Option<u64>,

    #[arg(long)]
    pub memory_swap: Option<u64>,

    #[arg(long)]
    pub pids_limit: Option<i64>,

    #[arg(long)]
    pub gpu: bool,

    #[arg(long)]
    pub gpu_device: Vec<String>,

    #[arg(long)]
    pub vram: Option<u64>,

    #[arg(long)]
    pub read_only: bool,

    #[arg(long)]
    pub privileged: bool,

    #[arg(long)]
    pub rm: bool,

    #[arg(long)]
    pub init: bool,

    #[arg(long)]
    pub cap_add: Vec<String>,

    #[arg(long)]
    pub cap_drop: Vec<String>,

    #[arg(long)]
    pub user: Option<String>,
}

pub fn execute(args: RunArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    if args.rootfs.is_none() && args.image.is_none() {
        return Err(anyhow::anyhow!("Either --rootfs or --image must be specified"));
    }

    let container_id = args.name.unwrap_or_else(id::generate_container_id);

    let rootfs_path = if let Some(rootfs) = args.rootfs {
        rootfs
    } else if let Some(_image) = &args.image {
        return Err(anyhow::anyhow!("--image support not yet implemented, use --rootfs"));
    } else {
        return Err(anyhow::anyhow!("Either --rootfs or --image must be specified"));
    };

    let mut env_vars = args.env.clone();
    env_vars.push("PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string());

    let _dns_servers = if args.dns.is_empty() {
        vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()]
    } else {
        args.dns.clone()
    };

    let port_mappings: Vec<String> = args.publish.to_vec();

    let volume_mounts: Vec<String> = args.volume.to_vec();

    let hostname = args.hostname.unwrap_or_else(|| container_id[..12.min(container_id.len())].to_string());

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
            env: env_vars,
            cwd: args.workdir,
            capabilities: None,
            rlimits: vec![],
            no_new_privileges: !args.privileged,
        }),
        hostname: Some(hostname),
        mounts: vec![],
        linux: Some(LinuxConfig {
            namespaces: build_namespaces(&args.network),
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
        &rootfs_path,
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

    if !port_mappings.is_empty() {
        tracing::info!("Port mappings: {:?}", port_mappings);
    }

    if !volume_mounts.is_empty() {
        tracing::info!("Volume mounts: {:?}", volume_mounts);
    }

    if !args.detach {
        let exit_code = container.wait()?;
        output::print_success(&format!(
            "Container {} exited with code {}",
            container_id, exit_code
        ));

        if args.rm {
            container.delete()?;
            tracing::info!("Container {} removed", container_id);
        }
    }

    Ok(())
}

fn build_namespaces(network: &str) -> Vec<NamespaceConfig> {
    let mut namespaces = vec![
        NamespaceConfig { r#type: NamespaceType::Pid, path: None },
        NamespaceConfig { r#type: NamespaceType::Mount, path: None },
        NamespaceConfig { r#type: NamespaceType::Uts, path: None },
        NamespaceConfig { r#type: NamespaceType::Ipc, path: None },
        NamespaceConfig { r#type: NamespaceType::Cgroup, path: None },
    ];

    if network != "host" {
        namespaces.push(NamespaceConfig { r#type: NamespaceType::Network, path: None });
    }

    namespaces
}

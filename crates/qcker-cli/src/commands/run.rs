use clap::Args;
use std::fs;
use std::path::{Path, PathBuf};

use crate::output;
use qcker_common::id;
use qcker_backend::types::{MountSpec, MountType, PortForwardSpec, Protocol};
use qcker_runtime::process::{ContainerProcess, ResourceLimits};
use qcker_runtime::spec::{CapabilitiesConfig, LinuxConfig, MountConfig, NamespaceConfig, NamespaceType, OciConfig, ProcessConfig, RootConfig, UserConfig};

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

pub async fn execute(args: RunArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    if args.rootfs.is_none() && args.image.is_none() {
        return Err(anyhow::anyhow!("Either --rootfs or --image must be specified"));
    }

    let container_id = args.name.unwrap_or_else(id::generate_container_id);

    let mut image_env: Vec<String> = Vec::new();
    let mut image_cmd: Vec<String> = Vec::new();
    let mut image_workdir: Option<String> = None;

    let rootfs_path = if let Some(rootfs) = args.rootfs {
        rootfs
    } else if let Some(image) = &args.image {
        use qcker_engine::image::store::ImageStore;
        use qcker_engine::registry::client::RegistryClient;
        use qcker_runtime::rootfs::{create_rootfs, RootfsConfig};

        let store = ImageStore::new(data_dir.to_path_buf());
        store.init().map_err(|e| anyhow::anyhow!("Failed to init image store: {}", e))?;

        let image_meta = if store.image_exists(image) {
            store.get_image(image).map_err(|e| anyhow::anyhow!("{}", e))?
        } else {
            let client = RegistryClient::new("registry-1.docker.io");
            client.pull_image(image, data_dir.to_path_buf()).await
                .map_err(|e| anyhow::anyhow!("Failed to pull image {}: {}", image, e))?
        };

        if let Some(ref image_cfg) = image_meta.config.config {
            if let Some(ref env_list) = image_cfg.env {
                image_env = env_list.clone();
            }
            if let Some(ref cmd_list) = image_cfg.cmd {
                image_cmd = cmd_list.clone();
            }
            if let Some(ref wd) = image_cfg.working_dir {
                image_workdir = Some(wd.clone());
            }
        }

        let container_dir = data_dir.join("containers").join(&container_id);
        fs::create_dir_all(&container_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create container dir: {}", e))?;

        let layer_paths: Vec<PathBuf> = image_meta.layers.iter().map(|digest| {
            let hash = digest.strip_prefix("sha256:").unwrap_or(digest);
            data_dir.join("layers").join(hash).join("layer")
        }).collect();

        let dns_list = if args.dns.is_empty() {
            vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()]
        } else {
            args.dns.clone()
        };

        let hostname_val = args.hostname.clone().unwrap_or_else(|| {
            container_id[..12.min(container_id.len())].to_string()
        });

        let rootfs_config = RootfsConfig {
            container_dir: container_dir.clone(),
            layers: layer_paths.clone(),
            rootless: !args.privileged,
            skip_mounts: false,
            hostname: Some(hostname_val.clone()),
            dns_servers: dns_list.clone(),
        };

        let rootfs = create_rootfs(&rootfs_config)
            .map_err(|e| anyhow::anyhow!("Failed to create rootfs from image: {}", e))?;

        for layer_path in &layer_paths {
            if layer_path.exists() {
                qcker_common::fs::copy_dir_all(layer_path, &rootfs)
                    .map_err(|e| anyhow::anyhow!("Failed to copy layer {}: {}", layer_path.display(), e))?;
            }
        }

        rootfs
    } else {
        return Err(anyhow::anyhow!("Either --rootfs or --image must be specified"));
    };

    let mut env_vars = args.env.clone();
    for e in &image_env {
        if !env_vars.iter().any(|existing| existing.starts_with(&e.split('=').next().unwrap_or(""))) {
            env_vars.push(e.clone());
        }
    }
    env_vars.push("PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string());

    let dns_servers = if args.dns.is_empty() {
        vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()]
    } else {
        args.dns.clone()
    };
    env_vars.push(format!("DNS_SERVERS={}", dns_servers.join(",")));

    let port_mappings: Vec<PortForwardSpec> = args.publish.iter().filter_map(|p| {
        let parts: Vec<&str> = p.splitn(2, ':').collect();
        if parts.len() == 2 {
            let host_port: u16 = parts[0].parse().ok()?;
            let guest_port: u16 = parts[1].parse().ok()?;
            Some(PortForwardSpec {
                host_port,
                guest_port,
                protocol: Protocol::Tcp,
            })
        } else {
            None
        }
    }).collect();

    let volume_mounts: Vec<MountSpec> = args.volume.iter().filter_map(|v| {
        let parts: Vec<&str> = v.splitn(2, ':').collect();
        if parts.len() == 2 {
            Some(MountSpec {
                source: parts[0].to_string(),
                target: parts[1].to_string(),
                mount_type: MountType::Bind,
                read_only: false,
            })
        } else {
            None
        }
    }).collect();

    let hostname = args.hostname.unwrap_or_else(|| container_id[..12.min(container_id.len())].to_string());

    let user_opt = args.user.clone();
    let (uid, gid) = if let Some(user) = &user_opt {
        let parts: Vec<&str> = user.splitn(2, ':').collect();
        let uid = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
        let gid = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        (uid, gid)
    } else {
        (0, 0)
    };

    let mut namespaces = build_namespaces(&args.network);
    if user_opt.is_some() {
        namespaces.push(NamespaceConfig { r#type: NamespaceType::User, path: None });
    }

    let default_caps = vec!["CAP_CHOWN", "CAP_DAC_OVERRIDE", "CAP_FOWNER", "CAP_FSETID", "CAP_KILL", "CAP_SETGID", "CAP_SETUID", "CAP_SETPCAP", "CAP_NET_BIND_SERVICE", "CAP_SYS_CHROOT", "CAP_MKNOD", "CAP_AUDIT_WRITE"];
    let mut effective_caps: Vec<String> = if args.privileged {
        vec!["ALL".to_string()]
    } else {
        default_caps.iter().map(|s| s.to_string()).collect()
    };
    for cap in &args.cap_drop {
        effective_caps.retain(|c| c != cap);
    }
    for cap in &args.cap_add {
        if !effective_caps.contains(cap) {
            effective_caps.push(cap.clone());
        }
    }

    let capabilities = if args.cap_add.is_empty() && args.cap_drop.is_empty() && !args.privileged {
        None
    } else {
        Some(CapabilitiesConfig {
            bounding: effective_caps.clone(),
            effective: effective_caps.clone(),
            inheritable: effective_caps.clone(),
            permitted: effective_caps,
            ambient: vec![],
        })
    };

    let config = OciConfig {
        oci_version: "1.0.0".to_string(),
        root: RootConfig {
            path: PathBuf::from("."),
            readonly: args.read_only,
        },
        process: Some(ProcessConfig {
            terminal: args.terminal,
            user: UserConfig { uid, gid },
            args: if args.init {
                let mut cmd = vec!["/sbin/init".to_string()];
                if !args.command.is_empty() {
                    cmd.extend(args.command);
                }
                cmd
            } else if !args.command.is_empty() {
                args.command
            } else if !image_cmd.is_empty() {
                image_cmd
            } else {
                vec!["/bin/sh".to_string()]
            },
            env: env_vars,
            cwd: if args.workdir != "/" {
                args.workdir
            } else {
                image_workdir.unwrap_or_else(|| "/".to_string())
            },
            capabilities,
            rlimits: vec![],
            no_new_privileges: !args.privileged,
        }),
        hostname: Some(hostname),
        mounts: volume_mounts.iter().map(|m| MountConfig {
            destination: PathBuf::from(&m.target),
            source: Some(PathBuf::from(&m.source)),
            r#type: Some("bind".to_string()),
            options: vec![if m.read_only { "ro".to_string() } else { "rw".to_string() }],
        }).collect(),
        linux: Some(LinuxConfig {
            namespaces,
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

# Qcker

A daemonless, rootless container engine written in Rust.

Qcker is a lightweight alternative to Docker that runs containers without a background daemon. It starts faster, uses less memory, and has a smaller binary footprint.

## Why Qcker?

| Metric | Docker | Qcker |
|--------|--------|-------|
| Binary size | ~200MB | ~7MB |
| Daemon memory | 100-300MB | 0 (no daemon) |
| Container startup | ~1.2s | <200ms |
| Rootless by default | No | Yes |
| Written in | Go | Rust |

## Quick Start

```bash
# Build from source
git clone https://github.com/qcker/qcker.git
cd qcker
cargo build --release

# Run a container
sudo ./target/release/qcker run --rootfs /path/to/rootfs -- /bin/echo "Hello from Qcker"

# Open TUI
./target/release/qcker
```

## Features

- Daemonless architecture (no background process)
- Rootless by default (no root required for most operations)
- OCI-compatible (runs standard Linux containers)
- Resource limits (CPU, memory, PIDs, GPU)
- Container isolation (namespaces, cgroups, seccomp)
- Built-in TUI for container management
- Extension system for custom networking, storage, and security
- Docker-compatible CLI

## CLI Commands

```
qcker run       Run a container
qcker create    Create a container
qcker start     Start a container
qcker stop      Stop a container
qcker kill      Kill a container
qcker delete    Delete a container
qcker ps        List containers
qcker exec      Execute command in container
qcker images    List images
qcker build     Build image from Dockerfile
qcker pull      Pull image from registry
qcker network   Manage networks
qcker volume    Manage volumes
qcker compose   Manage compose applications
qcker extension Manage extensions
qcker tui       Open terminal UI
```

## Resource Limits

```bash
qcker run --rootfs /path/to/rootfs \
    --cpus 2 \
    --memory 512 \
    --pids-limit 256 \
    --gpu \
    -- /bin/sh
```

Options:
- `--cpus <cores>` - CPU cores (e.g., 1.5)
- `--cpu-shares <weight>` - CPU shares (default 1024)
- `--memory <MB>` - Memory limit
- `--memory-swap <MB>` - Memory + swap limit
- `--pids-limit <n>` - Max processes (default 256)
- `--gpu` - Enable GPU access
- `--vram <MB>` - VRAM limit
- `--read-only` - Read-only rootfs
- `--privileged` - Root privileges mode

## Container Isolation

Each container runs in its own:
- PID namespace (process isolation)
- Network namespace (network isolation)
- Mount namespace (filesystem isolation)
- UTS namespace (hostname isolation)
- IPC namespace (IPC isolation)
- Cgroup namespace (resource isolation)

Security features:
- Seccomp syscall filtering
- Capability dropping
- User namespace mapping (rootless)
- Read-only rootfs option

## TUI

Run `qcker` without arguments to open the Terminal UI:

- Tab navigation between Containers, Images, Networks, Volumes, Files, Editor, Logs
- Browse container files
- Edit files with built-in editor
- Create/delete files and directories
- Container lifecycle management

## Architecture

```
qcker/
├── crates/
│   ├── qcker-cli/          # CLI binary + TUI
│   ├── qcker-runtime/      # OCI runtime (namespaces, cgroups, seccomp)
│   ├── qcker-backend/      # Runtime backend abstraction
│   ├── qcker-engine/       # Image, build, network, volume, compose
│   ├── qcker-common/       # Shared utilities
│   └── qcker-ext-api/      # Extension SDK
└── extensions/             # Built-in extensions
```

## Extension System

Qcker supports extensions for custom networking, storage, security scanning, and more.

```bash
# List extensions
qcker extension ls

# Install extension
qcker extension install /path/to/extension

# Enable/disable
qcker extension enable <name>
qcker extension disable <name>
```

Extension types:
- Network drivers (bridge, macvlan, cilium)
- Storage drivers (overlayfs, zfs, btrfs)
- Security scanners (trivy, grype)
- Log drivers (json-file, syslog, loki)
- Build strategies (dockerfile, buildkit)
- Custom CLI commands

## Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run linter
cargo clippy
```

## Requirements

- Linux kernel 5.3+
- Rust 1.70+
- Root or user namespace support

## License

Apache 2.0

## Tags

docker, docker-alternative, container, container-engine, container-runtime, rust, oci, rootless, daemonless, linux, namespaces, cgroups, podman-alternative, containerization, devops

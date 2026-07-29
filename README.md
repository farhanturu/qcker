# Qcker

A daemonless, rootless container engine written in Rust.

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-50%20passed-brightgreen.svg)](#testing)

Qcker is a lightweight alternative to Docker that runs containers without a background daemon. It starts faster, uses less memory, and has a smaller binary footprint.

**Production by [PaongLabs](https://github.com/farhanturu)**

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
# Clone and build
git clone https://github.com/farhanturu/qcker.git
cd qcker
cargo build --release

# Run a container
sudo ./target/release/qcker run --rootfs /path/to/rootfs -- /bin/echo "Hello from Qcker"

# Open TUI
./target/release/qcker
```

## Features

- **Daemonless** - No background process, fork-exec model
- **Rootless** - Runs without root by default
- **OCI-compatible** - Runs standard Linux containers
- **Resource limits** - CPU, memory, PIDs, GPU/VRAM
- **Container isolation** - Namespaces, cgroups, seccomp
- **Built-in TUI** - Terminal UI with file management
- **Extension system** - Custom networking, storage, security
- **Docker CLI compatible** - Drop-in replacement for most commands

## CLI Commands

```
qcker run           Run a container
qcker create        Create a container
qcker start         Start a container
qcker stop          Stop a container
qcker kill          Kill a container
qcker delete        Delete a container
qcker ps            List containers
qcker exec          Execute command in container
qcker images        List images
qcker build         Build image from Dockerfile
qcker pull          Pull image from registry
qcker network       Manage networks
qcker volume        Manage volumes
qcker compose       Manage compose applications
qcker extension     Manage extensions
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

| Option | Description |
|--------|-------------|
| `--cpus <cores>` | CPU cores (e.g., 1.5) |
| `--cpu-shares <weight>` | CPU shares (default 1024) |
| `--memory <MB>` | Memory limit |
| `--memory-swap <MB>` | Memory + swap limit |
| `--pids-limit <n>` | Max processes (default 256) |
| `--gpu` | Enable GPU access |
| `--vram <MB>` | VRAM limit |
| `--read-only` | Read-only rootfs |
| `--privileged` | Root privileges mode |

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

Run `qcker` without arguments to open the Terminal UI.

**Tabs:** Containers, Images, Networks, Volumes, Files, Editor, Marketplace, Logs

**Navigation:**
- `Tab/Shift+Tab` - Switch tabs
- `Up/Down` - Navigate items
- `Enter` - Select/open
- `r` - Refresh
- `h` - Help
- `q` - Quit

**File Browser:**
- `Enter` - Open file/directory
- `e` - Edit file
- `d` - Delete
- `n` - New file
- `m` - New directory

**Marketplace:**
- `u/Enter` - Uninstall extension
- Request: [GitHub Issues](https://github.com/farhanturu/qcker-extensions/issues)

## Extensions

Browse and manage extensions in the TUI Marketplace tab.

**Available Extensions:**
- bridge (network) - Built-in
- overlayfs (storage) - Built-in
- trivy (security) - Vulnerability scanning
- cilium (network) - eBPF networking
- zfs (storage) - ZFS backend
- loki (logging) - Grafana Loki
- buildkit (build) - BuildKit builder

**Install:**
```bash
qcker extension install /path/to/extension.so
```

**Request new:** [Open an issue](https://github.com/farhanturu/qcker-extensions/issues/new?template=extension_request.yml)

## Architecture

```
qcker/
├── crates/
│   ├── qcker-cli/          # CLI binary + TUI
│   ├── qcker-runtime/      # OCI runtime
│   ├── qcker-backend/      # Backend abstraction
│   ├── qcker-engine/       # Image, build, network, volume, compose, extension
│   ├── qcker-common/       # Shared utilities
│   └── qcker-ext-api/      # Extension SDK
└── target/release/qcker    # Binary (~7.5MB)
```

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

## Documentation

- [Getting Started](docs/guides/GETTING_STARTED.md)
- [CLI Reference](docs/api/CLI.md)
- [Benchmarks](docs/benchmark/BENCHMARK.md)
- [Extensions](https://github.com/farhanturu/qcker-extensions)

## License

Apache 2.0

## Author

**PaongLabs** - [GitHub](https://github.com/farhanturu)

## Tags

```
docker, docker-alternative, container, container-engine, container-runtime,
rust, oci, rootless, daemonless, linux, namespaces, cgroups, podman-alternative,
containerization, devops, lightweight, fast, secure, kubernetes, k8s, crio
```

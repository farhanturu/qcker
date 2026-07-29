<p align="center">
  <img src="logo.png" alt="Qcker Logo" width="600">
</p>

<h1 align="center">Qcker</h1>

<p align="center">
  <strong>A daemonless, rootless container engine written in Rust</strong>
</p>

<p align="center">
  <a href="https://opensource.org/licenses/Apache-2.0"><img src="https://img.shields.io/badge/License-Apache_2.0-blue.svg" alt="License"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.70%2B-orange.svg" alt="Rust"></a>
  <a href="#"><img src="https://img.shields.io/badge/tests-50%20passed-brightgreen.svg" alt="Tests"></a>
  <a href="#"><img src="https://img.shields.io/badge/version-0.1.0-blue.svg" alt="Version"></a>
</p>

<p align="center">
  Qcker is a lightweight, high-performance alternative to Docker.
  <br>
  No daemon. No bloat. Just containers.
</p>

---

## What is Qcker?

Qcker is a container engine that runs Linux containers without a background daemon. It is written in Rust for safety and speed, and is fully OCI-compliant so it can run existing Docker images.

**Key differences from Docker:**

| | Docker | Qcker |
|---|---|---|
| Daemon | Yes (dockerd, 100-300MB RAM) | None |
| Binary size | ~200 MB | ~7 MB |
| Container startup | ~1.2s | <200ms |
| Rootless by default | No | Yes |
| Language | Go | Rust |
| Built-in TUI | No | Yes |
| GPU support | Manual | Built-in |

---

## Quick Start

### Build

```bash
git clone https://github.com/farhanturu/qcker.git
cd qcker
cargo build --release
```

### Run a container

```bash
# Simple command
sudo ./target/release/qcker run --rootfs /path/to/rootfs -- /bin/echo "Hello"

# Interactive shell
sudo ./target/release/qcker run --rootfs /path/to/rootfs -t -- /bin/sh

# With resource limits
sudo ./target/release/qcker run --rootfs /path/to/rootfs \
    --cpus 2 --memory 512 --pids-limit 256 \
    -- /bin/sh

# With GPU
sudo ./target/release/qcker run --rootfs /path/to/rootfs \
    --gpu --vram 1024 \
    -- /bin/sh
```

### Open TUI

```bash
./target/release/qcker
```

---

## Features

**Container Management**
- Create, start, stop, kill, delete containers
- List running and stopped containers
- Execute commands inside running containers
- Container logs and state inspection

**Resource Limits**
- `--cpus <cores>` - CPU cores (e.g., 1.5)
- `--memory <MB>` - Memory limit
- `--pids-limit <n>` - Max processes
- `--gpu` - Enable GPU access
- `--vram <MB>` - VRAM limit

**Security**
- Rootless by default
- PID, network, mount, UTS, IPC, cgroup namespace isolation
- Seccomp syscall filtering
- Capability dropping
- Read-only rootfs option

**TUI (Terminal UI)**
- 8 tabs: Containers, Images, Networks, Volumes, Files, Editor, Marketplace, Logs
- Browse and edit files inside containers
- Manage extensions

**Docker-Compatible CLI**
- `qcker run`, `qcker ps`, `qcker images`, `qcker build`, `qcker exec`
- `qcker network`, `qcker volume`, `qcker compose`, `qcker extension`

---

## CLI Commands

| Command | Description |
|---------|-------------|
| `qcker run` | Run a container |
| `qcker create` | Create a container |
| `qcker start` | Start a container |
| `qcker stop` | Stop a container |
| `qcker kill` | Kill a container |
| `qcker delete` | Delete a container |
| `qcker ps` | List containers |
| `qcker exec` | Execute in container |
| `qcker images` | List images |
| `qcker build` | Build from Dockerfile |
| `qcker pull` | Pull image from registry |
| `qcker network` | Manage networks |
| `qcker volume` | Manage volumes |
| `qcker compose` | Manage compose apps |
| `qcker extension` | Manage extensions |

---

## Architecture

```
qcker
├── qcker-cli          CLI binary + TUI
├── qcker-runtime      OCI runtime (namespaces, cgroups, seccomp)
├── qcker-backend      Runtime backend abstraction
├── qcker-engine       Image, build, network, volume, compose, extension
├── qcker-common       Shared utilities
└── qcker-ext-api      Extension SDK
```

How it works:

1. CLI parses commands and builds OCI runtime specs
2. Engine manages images, networks, volumes, extensions
3. Runtime creates containers using Linux kernel primitives:
   - `clone()` with namespace flags for isolation
   - `chroot()` for filesystem isolation
   - cgroups v2 for resource limits
   - seccomp for syscall filtering

---

## Extensions

Qcker has a first-class extension system for custom networking, storage, security, and more.

| Extension | Category | Status |
|-----------|----------|--------|
| bridge | network | Built-in |
| overlayfs | storage | Built-in |
| trivy | security | Available |
| cilium | network | Available |
| zfs | storage | Available |
| loki | logging | Available |
| buildkit | build | Available |

Browse and request extensions: https://github.com/farhanturu/qcker-extensions

---

## Comparison

| Feature | Docker | Podman | Qcker |
|---------|--------|--------|-------|
| Daemon | Yes | No | No |
| Rootless | Optional | Yes | Yes |
| Language | Go | Go | Rust |
| Binary size | 200 MB | 100 MB | 7.5 MB |
| TUI | No | No | Yes |
| GPU support | Manual | Manual | Built-in |
| Extensions | Limited | Limited | Full SDK |

---

## Requirements

- Linux kernel 5.3+
- Rust 1.70+ (for building)
- Root access or user namespace support

---

## Build

```bash
cargo build --release    # Release build
cargo test               # Run tests
cargo clippy             # Lint
```

---

## License

Apache License 2.0 - see [LICENSE](LICENSE)

---

## Author

**PaongLabs**
- GitHub: https://github.com/farhanturu
- Email: paongtech@gmail.com

---

## Tags

docker, docker-alternative, container, container-engine, container-runtime, rust, oci, rootless, daemonless, linux, namespaces, cgroups, podman, podman-alternative, kubernetes, k8s, devops, gpu, tui, lightweight-containers, fast-containers, container-tools, docker-replacement, docker-desktop-alternative

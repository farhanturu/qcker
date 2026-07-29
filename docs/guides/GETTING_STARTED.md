# Getting Started with Qcker

## Installation

### Build from Source

```bash
# Prerequisites
sudo apt-get update
sudo apt-get install -y build-essential libssl-dev pkg-config

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/qcker/qcker.git
cd qcker
cargo build --release

# Binary location
ls -la target/release/qcker
```

### Quick Test

```bash
# Create a test rootfs
mkdir -p /tmp/test-rootfs/{bin,etc,proc,sys,dev,tmp}
cp /bin/sh /tmp/test-rootfs/bin/

# Run a container
sudo ./target/release/qcker run --rootfs /tmp/test-rootfs -- /bin/sh -c "echo Hello from Qcker!"
```

## Basic Usage

### Running Containers

```bash
# Run with command
qcker run --rootfs /path/to/rootfs -- /bin/echo "Hello"

# Run interactively
qcker run --rootfs /path/to/rootfs -t -- /bin/sh

# Run in background
qcker run --rootfs /path/to/rootfs -d --name myapp -- /bin/sh

# Run with environment variables
qcker run --rootfs /path/to/rootfs -e "APP=test" -e "DEBUG=true" -- /bin/sh

# Run with resource limits
qcker run --rootfs /path/to/rootfs --cpus 2 --memory 512 -- /bin/sh
```

### Managing Containers

```bash
# List running containers
qcker ps

# List all containers
qcker ps --all

# Show container state
qcker state <container-id>

# Stop container
qcker stop <container-id>

# Delete container
qcker delete <container-id>

# Force delete running container
qcker delete -f <container-id>
```

### Executing Commands

```bash
# Execute command in running container
qcker exec <container-id> /bin/ls

# Execute interactively
qcker exec -it <container-id> /bin/sh
```

## Resource Management

### CPU Limits

```bash
# Limit to 1.5 CPU cores
qcker run --rootfs /path/to/rootfs --cpus 1.5 -- /bin/sh

# Set CPU shares (relative weight)
qcker run --rootfs /path/to/rootfs --cpu-shares 512 -- /bin/sh
```

### Memory Limits

```bash
# Limit to 256MB RAM
qcker run --rootfs /path/to/rootfs --memory 256 -- /bin/sh

# Limit to 512MB RAM + 1GB swap
qcker run --rootfs /path/to/rootfs --memory 512 --memory-swap 1024 -- /bin/sh
```

### Process Limits

```bash
# Limit to 100 processes
qcker run --rootfs /path/to/rootfs --pids-limit 100 -- /bin/sh
```

### GPU Access

```bash
# Enable GPU access
qcker run --rootfs /path/to/rootfs --gpu -- /bin/sh

# Specify GPU device
qcker run --rootfs /path/to/rootfs --gpu --gpu-device /dev/nvidia0 -- /bin/sh

# Set VRAM limit
qcker run --rootfs /path/to/rootfs --gpu --vram 2048 -- /bin/sh
```

## TUI (Terminal UI)

```bash
# Open TUI
qcker
```

TUI Navigation:
- Tab/Shift+Tab - Switch tabs
- Up/Down - Navigate items
- Enter - Select/open
- r - Refresh
- h - Help
- q - Quit

Container Files:
- Enter - Open file/directory
- e - Edit file
- d - Delete
- n - New file
- m - New directory

## Networking

```bash
# Create network
qcker network create mynet

# List networks
qcker network ls

# Run container on network
qcker run --rootfs /path/to/rootfs --network mynet -- /bin/sh

# Remove network
qcker network rm mynet
```

## Volumes

```bash
# Create volume
qcker volume create mydata

# List volumes
qcker volume ls

# Run with volume
qcker run --rootfs /path/to/rootfs -v mydata:/data -- /bin/sh

# Remove volume
qcker volume rm mydata
```

## Compose

Create a `docker-compose.yml`:

```yaml
version: "3"
services:
  web:
    image: nginx:latest
    ports:
      - "8080:80"
  app:
    image: node:18
    depends_on:
      - web
```

```bash
# Start services
qcker compose up -d

# List services
qcker compose ps

# View logs
qcker compose logs

# Stop services
qcker compose down
```

## Extensions

```bash
# List extensions
qcker extension ls

# Install extension
qcker extension install /path/to/extension

# Enable extension
qcker extension enable <extension-id>

# Disable extension
qcker extension disable <extension-id>
```

## Troubleshooting

### Permission Denied

If you get permission errors:
```bash
# Use sudo for root operations
sudo qcker run --rootfs /path/to/rootfs -- /bin/sh

# Or enable user namespaces
qcker run --rootfs /path/to/rootfs --privileged -- /bin/sh
```

### Container Won't Start

Check:
1. Rootfs path exists and contains valid filesystem
2. Command exists in rootfs
3. Required libraries are available in rootfs

### Resource Limits Not Working

Resource limits require:
1. Cgroups v2 mounted at /sys/fs/cgroup
2. Root privileges or delegated cgroup access

## Configuration

Qcker configuration is stored in `~/.config/qcker/config.toml`:

```toml
[default]
data_dir = "~/.local/share/qcker"
log_level = "info"

[network]
default_driver = "bridge"

[security]
default_seccomp = true
default_capabilities = false
```

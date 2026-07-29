# Qcker API Reference

## CLI Commands

### Container Lifecycle

#### qcker run

Run a container.

```
qcker run [OPTIONS] --rootfs <ROOTFS> [COMMAND]...
```

Options:
- `--rootfs <PATH>` - Root filesystem path (required)
- `--name <NAME>` - Container name
- `-t, --terminal` - Enable terminal
- `-w, --workdir <DIR>` - Working directory (default: /)
- `-e, --env <KEY=VALUE>` - Environment variables
- `--hostname <NAME>` - Container hostname
- `-d, --detach` - Run in background
- `--cpus <CORES>` - CPU cores (e.g., 1.5)
- `--cpu-shares <WEIGHT>` - CPU shares (default: 1024)
- `-m, --memory <MB>` - Memory limit
- `--memory-swap <MB>` - Memory + swap limit
- `--pids-limit <N>` - Max processes (default: 256)
- `--gpu` - Enable GPU access
- `--gpu-device <PATH>` - GPU device path
- `--vram <MB>` - VRAM limit
- `--read-only` - Read-only rootfs
- `--privileged` - Root privileges mode

#### qcker create

Create a container without starting it.

```
qcker create [OPTIONS] --rootfs <ROOTFS> [COMMAND]...
```

Same options as `qcker run`.

#### qcker start

Start a created container.

```
qcker start <CONTAINER_ID>
```

#### qcker stop

Stop a running container.

```
qcker stop [OPTIONS] <CONTAINER_ID>
```

Options:
- `-t, --time <SECONDS>` - Seconds to wait before killing (default: 10)

#### qcker kill

Send signal to container.

```
qcker kill [OPTIONS] <CONTAINER_ID>
```

Options:
- `-s, --signal <SIGNAL>` - Signal to send (default: SIGKILL)

#### qcker delete

Delete a container.

```
qcker delete [OPTIONS] <CONTAINER_ID>
```

Options:
- `-f, --force` - Force delete running container

#### qcker ps

List containers.

```
qcker ps [OPTIONS]
```

Options:
- `-a, --all` - Show all containers (default: running only)
- `--format <FORMAT>` - Output format (text, json)

#### qcker state

Show container state.

```
qcker state <CONTAINER_ID>
```

#### qcker exec

Execute command in running container.

```
qcker exec [OPTIONS] <CONTAINER_ID> <COMMAND>...
```

Options:
- `-t, --terminal` - Enable terminal
- `-i, --interactive` - Keep stdin open

### Image Management

#### qcker images

List local images.

```
qcker images [OPTIONS]
```

Options:
- `-a, --all` - Show all images
- `--format <FORMAT>` - Output format (text, json)

#### qcker pull

Pull image from registry.

```
qcker pull [OPTIONS] <IMAGE>
```

Options:
- `--registry <URL>` - Registry URL (default: registry-1.docker.io)

#### qcker build

Build image from Dockerfile.

```
qcker build [OPTIONS] [PATH]
```

Options:
- `-t, --tag <TAG>` - Image tag
- `-f, --file <FILE>` - Dockerfile path (default: Dockerfile)
- `--build-arg <KEY=VALUE>` - Build arguments
- `--no-cache` - Disable cache

### Network Management

#### qcker network create

Create a network.

```
qcker network create [OPTIONS] <NAME>
```

Options:
- `-d, --driver <DRIVER>` - Network driver (default: bridge)
- `--subnet <SUBNET>` - Subnet (e.g., 172.20.0.0/16)

#### qcker network ls

List networks.

```
qcker network ls
```

#### qcker network rm

Remove a network.

```
qcker network rm <NAME>
```

#### qcker network inspect

Inspect a network.

```
qcker network inspect <NAME>
```

### Volume Management

#### qcker volume create

Create a volume.

```
qcker volume create [OPTIONS] <NAME>
```

Options:
- `-d, --driver <DRIVER>` - Volume driver (default: local)

#### qcker volume ls

List volumes.

```
qcker volume ls
```

#### qcker volume rm

Remove a volume.

```
qcker volume rm <NAME>
```

#### qcker volume inspect

Inspect a volume.

```
qcker volume inspect <NAME>
```

### Compose

#### qcker compose up

Start services.

```
qcker compose up [OPTIONS] [SERVICES...]
```

Options:
- `-f, --file <FILE>` - Compose file (default: docker-compose.yml)
- `-p, --name <NAME>` - Project name
- `-d, --detach` - Run in background

#### qcker compose down

Stop services.

```
qcker compose down [OPTIONS]
```

Options:
- `-v, --volumes` - Remove volumes

#### qcker compose ps

List services.

```
qcker compose ps
```

### Extensions

#### qcker extension ls

List installed extensions.

```
qcker extension ls
```

#### qcker extension install

Install an extension.

```
qcker extension install <PATH>
```

#### qcker extension uninstall

Uninstall an extension.

```
qcker extension uninstall <ID>
```

#### qcker extension enable

Enable an extension.

```
qcker extension enable <ID>
```

#### qcker extension disable

Disable an extension.

```
qcker extension disable <ID>
```

#### qcker extension info

Show extension details.

```
qcker extension info <ID>
```

## Output Formats

All commands support `--format` option:

- `text` - Human-readable output (default)
- `json` - JSON output for scripting

Example:
```bash
qcker ps --format json
```

## Environment Variables

- `QCKER_DATA_DIR` - Data directory (default: ~/.local/share/qcker)
- `QCKER_LOG_LEVEL` - Log level (debug, info, warn, error)

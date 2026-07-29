# Qcker Extension API Reference

## Overview

Qcker extensions are dynamic libraries (.so on Linux, .dylib on macOS) that extend the container engine with custom networking, storage, security, logging, and build capabilities.

Extensions communicate with Qcker via JSON-RPC over Unix sockets, allowing them to be written in any language.

## Extension Types

| Type | Trait | Description |
|------|-------|-------------|
| Network Driver | `NetworkDriver` | Custom container networking |
| Storage Driver | `StorageDriver` | Custom storage backends |
| Security Scanner | `SecurityScanner` | Vulnerability scanning |
| Log Driver | `LogDriver` | Custom log capture |
| Build Strategy | `BuildStrategy` | Alternative build methods |
| Hook Extension | `HookExtension` | Container lifecycle hooks |
| Command Extension | `CommandExtension` | Custom CLI commands |

## Rust SDK

### Installation

```toml
[dependencies]
qcker-ext-api = "0.1"
```

### Base Extension Trait

Every extension must implement `Extension`:

```rust
use qcker_ext_api::prelude::*;
use std::collections::HashMap;

#[derive(Default)]
struct MyExtension;

#[async_trait]
impl Extension for MyExtension {
    fn id(&self) -> &str {
        "com.example.my-ext"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn name(&self) -> &str {
        "My Extension"
    }

    fn description(&self) -> &str {
        "Does something useful"
    }

    fn author(&self) -> &str {
        "Your Name"
    }

    fn api_version(&self) -> &str {
        "1.0.0"
    }

    fn capabilities(&self) -> Vec<ExtensionCapability> {
        vec![
            ExtensionCapability::NetworkAccess,
            ExtensionCapability::ContainerLifecycle,
        ]
    }

    async fn on_load(&mut self, ctx: &ExtensionContext) -> Result<(), String> {
        tracing::info!("Extension loaded");
        Ok(())
    }

    async fn on_unload(&mut self) -> Result<(), String> {
        tracing::info!("Extension unloaded");
        Ok(())
    }

    async fn health_check(&self) -> Result<(), String> {
        Ok(())
    }
}
```

### Export Macro

```rust
qcker_extension!(MyExtension);
```

This generates the required C FFI entry points for dynamic loading.

### Network Driver

```rust
#[async_trait]
impl NetworkDriver for MyExtension {
    fn driver_name(&self) -> &str {
        "my-driver"
    }

    async fn init(&mut self, config: &DriverConfig) -> Result<(), String> {
        Ok(())
    }

    async fn create_network(&mut self, spec: &NetworkSpec) -> Result<NetworkInfo, String> {
        Ok(NetworkInfo {
            id: "net-123".to_string(),
            name: spec.name.clone(),
            driver: "my-driver".to_string(),
            subnet: Some("172.20.0.0/16".to_string()),
            gateway: Some("172.20.0.1".to_string()),
        })
    }

    async fn delete_network(&mut self, network_id: &str) -> Result<(), String> {
        Ok(())
    }

    async fn connect(
        &mut self,
        network_id: &str,
        container_id: &str,
        opts: &ConnectOpts,
    ) -> Result<(), String> {
        Ok(())
    }

    async fn disconnect(
        &mut self,
        network_id: &str,
        container_id: &str,
    ) -> Result<(), String> {
        Ok(())
    }

    async fn list_networks(&self) -> Result<Vec<NetworkInfo>, String> {
        Ok(vec![])
    }

    async fn inspect_network(&self, network_id: &str) -> Result<NetworkDetail, String> {
        Err("Not implemented".to_string())
    }
}
```

### Storage Driver

```rust
#[async_trait]
impl StorageDriver for MyExtension {
    fn driver_name(&self) -> &str {
        "my-storage"
    }

    async fn init(&mut self, config: &DriverConfig) -> Result<(), String> {
        Ok(())
    }

    async fn prepare(&mut self, key: &str, parent: &str) -> Result<Mount, String> {
        Ok(Mount {
            source: format!("/data/{}", key),
            destination: "/".to_string(),
            mount_type: "bind".to_string(),
            options: vec![],
        })
    }

    async fn commit(&mut self, key: &str, name: &str) -> Result<(), String> {
        Ok(())
    }

    async fn remove(&mut self, key: &str) -> Result<(), String> {
        Ok(())
    }

    async fn mount(&mut self, key: &str) -> Result<PathBuf, String> {
        Ok(PathBuf::from(format!("/data/{}", key)))
    }

    async fn unmount(&mut self, key: &str) -> Result<(), String> {
        Ok(())
    }

    async fn list(&self) -> Result<Vec<SnapshotInfo>, String> {
        Ok(vec![])
    }
}
```

### Security Scanner

```rust
#[async_trait]
impl SecurityScanner for MyExtension {
    fn scanner_name(&self) -> &str {
        "my-scanner"
    }

    async fn init(&mut self, config: &DriverConfig) -> Result<(), String> {
        Ok(())
    }

    async fn scan_image(
        &mut self,
        image_id: &str,
        opts: &ScanOpts,
    ) -> Result<ScanResult, String> {
        Ok(ScanResult {
            vulnerabilities: vec![],
            summary: ScanSummary {
                total: 0,
                critical: 0,
                high: 0,
                medium: 0,
                low: 0,
            },
        })
    }

    async fn scan_container(
        &mut self,
        container_id: &str,
        opts: &ScanOpts,
    ) -> Result<ScanResult, String> {
        Ok(ScanResult {
            vulnerabilities: vec![],
            summary: ScanSummary {
                total: 0,
                critical: 0,
                high: 0,
                medium: 0,
                low: 0,
            },
        })
    }
}
```

### Log Driver

```rust
#[async_trait]
impl LogDriver for MyExtension {
    fn driver_name(&self) -> &str {
        "my-logger"
    }

    async fn init(&mut self, config: &DriverConfig) -> Result<(), String> {
        Ok(())
    }

    async fn start_capture(
        &mut self,
        container_id: &str,
        opts: &LogOpts,
    ) -> Result<LogHandle, String> {
        Ok(LogHandle {
            id: "handle-123".to_string(),
            container_id: container_id.to_string(),
        })
    }

    async fn stop_capture(&mut self, handle: &LogHandle) -> Result<(), String> {
        Ok(())
    }

    async fn read_logs(
        &self,
        container_id: &str,
        opts: &LogReadOpts,
    ) -> Result<Vec<LogEntry>, String> {
        Ok(vec![])
    }
}
```

### Hook Extension

```rust
#[async_trait]
impl HookExtension for MyExtension {
    fn hook_name(&self) -> &str {
        "my-hooks"
    }

    fn priority(&self) -> u32 {
        100
    }

    async fn pre_create(&mut self, spec: &ContainerSpec) -> Result<HookDecision, String> {
        // Allow all creates
        Ok(HookDecision::Allow)
    }

    async fn post_create(&mut self, info: &ContainerInfo) -> Result<(), String> {
        tracing::info!("Container created: {}", info.id);
        Ok(())
    }

    async fn pre_start(&mut self, info: &ContainerInfo) -> Result<HookDecision, String> {
        Ok(HookDecision::Allow)
    }

    async fn post_start(&mut self, info: &ContainerInfo) -> Result<(), String> {
        Ok(())
    }

    async fn pre_stop(&mut self, info: &ContainerInfo) -> Result<HookDecision, String> {
        Ok(HookDecision::Allow)
    }

    async fn post_stop(&mut self, info: &ContainerInfo) -> Result<(), String> {
        Ok(())
    }
}
```

### Command Extension

```rust
#[async_trait]
impl CommandExtension for MyExtension {
    fn command_name(&self) -> &str {
        "my-cmd"
    }

    fn help(&self) -> &str {
        "My custom command"
    }

    async fn execute(&mut self, args: &[String]) -> Result<(), String> {
        println!("Executing with args: {:?}", args);
        Ok(())
    }

    fn completions(&self, current: &str) -> Vec<String> {
        vec!["sub1".to_string(), "sub2".to_string()]
    }
}
```

## Extension Capabilities

| Capability | Description |
|------------|-------------|
| `NetworkAccess` | Can make network connections |
| `FileSystemAccess` | Can read/write filesystem |
| `ContainerLifecycle` | Can manage containers |
| `ImageAccess` | Can read/write images |
| `Privileged` | Needs root access |
| `SystemInfo` | Can read system information |
| `RegistryAccess` | Can access registries |
| `ProcessSpawn` | Can spawn subprocesses |

## Hook Decisions

| Decision | Description |
|----------|-------------|
| `HookDecision::Allow` | Allow the operation |
| `HookDecision::Deny(reason)` | Deny with reason |
| `HookDecision::Modify(mods)` | Allow with modifications |

## Extension Context

The `ExtensionContext` provides access to host APIs:

```rust
#[async_trait]
pub trait ExtensionContext: Send + Sync {
    async fn get_config(&self, key: &str) -> Result<Option<String>, String>;
    async fn set_config(&self, key: &str, value: &str) -> Result<(), String>;
    async fn list_containers(&self) -> Result<Vec<ContainerInfo>, String>;
    async fn inspect_container(&self, id: &str) -> Result<ContainerDetail, String>;
    async fn list_images(&self) -> Result<Vec<ImageInfo>, String>;
    async fn inspect_image(&self, id: &str) -> Result<ImageDetail, String>;
    async fn subscribe(&self, event_types: &[EventType]) -> Result<EventStream, String>;
    async fn log(&self, level: LogLevel, message: &str) -> Result<(), String>;
}
```

## Events

| Event | Description |
|-------|-------------|
| `ContainerCreated` | Container created |
| `ContainerStarted` | Container started |
| `ContainerStopped` | Container stopped |
| `ContainerExited` | Container exited |
| `ContainerDeleted` | Container deleted |
| `ImagePulled` | Image pulled from registry |
| `ImageBuilt` | Image built from Dockerfile |
| `ImageDeleted` | Image deleted |
| `NetworkCreated` | Network created |
| `NetworkDeleted` | Network deleted |
| `ExtensionLoaded` | Extension loaded |
| `ExtensionUnloaded` | Extension unloaded |

## IPC Protocol

Extensions communicate via JSON-RPC 2.0 over Unix sockets.

### Request Format

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "network.create",
  "params": {
    "name": "my-network",
    "driver": "bridge",
    "subnet": "172.20.0.0/16"
  }
}
```

### Response Format

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "network_id": "abc123",
    "subnet": "172.20.0.0/16",
    "gateway": "172.20.0.1"
  }
}
```

### Event Format

```json
{
  "jsonrpc": "2.0",
  "method": "container.started",
  "params": {
    "container_id": "def456",
    "image": "alpine:latest",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

## Built-in IPC Methods

Host provides these methods to extensions:

| Method | Description |
|--------|-------------|
| `container.list` | List containers |
| `container.inspect` | Inspect container |
| `container.logs` | Get container logs |
| `image.list` | List images |
| `image.inspect` | Inspect image |
| `network.list` | List networks |
| `volume.list` | List volumes |
| `system.info` | Get system info |
| `config.get` | Get config value |
| `config.set` | Set config value |
| `event.subscribe` | Subscribe to events |
| `event.unsubscribe` | Unsubscribe |

## Extension Manifest

Every extension needs a `manifest.json`:

```json
{
  "id": "com.example.my-ext",
  "name": "my-ext",
  "display_name": "My Extension",
  "version": "1.0.0",
  "api_version": "1.0.0",
  "author": "Your Name",
  "description": "What it does",
  "category": "network",
  "capabilities": ["NetworkAccess"],
  "repository": "https://github.com/user/repo",
  "license": "Apache-2.0"
}
```

## CLI Commands

```bash
# List extensions
qcker extension ls

# Install
qcker extension install /path/to/extension.so

# Uninstall
qcker extension uninstall <id>

# Enable
qcker extension enable <id>

# Disable
qcker extension disable <id>

# Info
qcker extension info <id>
```

## Complete Example

```rust
use qcker_ext_api::prelude::*;
use std::collections::HashMap;

#[derive(Default)]
struct FirewallExtension {
    rules: Vec<FirewallRule>,
}

struct FirewallRule {
    container_id: String,
    allowed_ports: Vec<u16>,
}

#[async_trait]
impl Extension for FirewallExtension {
    fn id(&self) -> &str { "com.example.firewall" }
    fn version(&self) -> &str { "1.0.0" }
    fn name(&self) -> &str { "Container Firewall" }
    fn description(&self) -> &str { "Network firewall for containers" }
    fn author(&self) -> &str { "Security Team" }
    fn api_version(&self) -> &str { "1.0.0" }
    fn capabilities(&self) -> Vec<ExtensionCapability> {
        vec![ExtensionCapability::NetworkAccess, ExtensionCapability::ContainerLifecycle]
    }

    async fn on_load(&mut self, ctx: &ExtensionContext) -> Result<(), String> {
        // Load rules from config
        if let Some(rules_json) = ctx.get_config("rules").await? {
            self.rules = serde_json::from_str(&rules_json).unwrap_or_default();
        }
        Ok(())
    }

    async fn on_unload(&mut self) -> Result<(), String> {
        self.rules.clear();
        Ok(())
    }
}

#[async_trait]
impl HookExtension for FirewallExtension {
    fn hook_name(&self) -> &str { "firewall" }

    async fn pre_start(&mut self, info: &ContainerInfo) -> Result<HookDecision, String> {
        // Check if container has allowed ports
        tracing::info!("Checking firewall rules for container {}", info.id);
        Ok(HookDecision::Allow)
    }

    async fn post_start(&mut self, info: &ContainerInfo) -> Result<(), String> {
        // Apply iptables rules
        tracing::info!("Applying firewall rules for container {}", info.id);
        Ok(())
    }

    async fn pre_stop(&mut self, info: &ContainerInfo) -> Result<HookDecision, String> {
        Ok(HookDecision::Allow)
    }

    async fn post_stop(&mut self, info: &ContainerInfo) -> Result<(), String> {
        // Remove iptables rules
        tracing::info!("Removing firewall rules for container {}", info.id);
        Ok(())
    }
}

qcker_extension!(FirewallExtension);
```

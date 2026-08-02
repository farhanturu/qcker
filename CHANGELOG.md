# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2025-08-02

### Added
- **Real MicroVM Backend Implementation**: Full container lifecycle management via vsock
  - Container create/start/kill/delete via HostToVm protocol commands
  - Exec in container via vsock (ContainerExec command)
  - Container stats collection from guest agent (CPU, memory, network, PIDs)
  - File operations: list_files, read_file, write_file, delete_file, create_dir, upload_file, download_file
  - Lazy VM startup: VM starts automatically on first container creation
  - Port forwarding: host port → guest port mapping
  - Container state persistence to disk (JSON)
  - Graceful shutdown with VmShutdown notification to guest agent
- **vsock Communication Layer** (`vsock.rs`):
  - `SyncVsockChannel::connect()` for establishing vsock connections
  - Length-prefixed JSON protocol (4-byte BE header + JSON payload)
  - `recv_timeout()` with poll-based timeout handling
  - EINTR retry handling for partial reads/writes
  - Buffer size configuration via `set_buffer_sizes()`
- **Enhanced VmmManager** (`vmm.rs`):
  - `start_with_log()` for serial console logging to file
  - `is_running()` health check for QEMU process
  - `pid()` and `vsock_cid()` accessors
  - `read_log()` for reading VM serial console output
  - `check_kvm_available()` for KVM acceleration detection
  - `get_qemu_version()` for version query
  - `pci=off` kernel param for faster boot
  - `-display none` for headless operation
- **Dracula Theme System** (`theme.rs`): Full Dracula color palette with semantic naming
  - Status colors: `STATUS_RUNNING`, `STATUS_STOPPED`, `STATUS_CREATED`, `STATUS_PAUSED`, `STATUS_DEAD`
  - Gauge colors: `GAUGE_LOW`, `GAUGE_MED`, `GAUGE_HIGH` for CPU/memory bars
  - UI chrome: `HEADER_BG`, `FOOTER_BG`, `BORDER`, `SELECTED_BG`, `SURFACE_BRIGHT`
  - Accent colors: `CYAN`, `ACCENT` (alias), `PURPLE`, `GREEN`, `YELLOW`, `RED`, `ORANGE`, `PINK`
  - Text colors: `TEXT`, `TEXT_DIM`, `TEXT_SUBTLE`
  - 20+ pre-built style helpers: `text_style()`, `accent_style()`, `selected_style()`, `status_style()`, `gauge_color()`, etc.
  - Tab metadata: `TAB_TITLES`, `TAB_WIDTH` constants
- **TUI visual refinements**:
  - Status bar and footer use darker background (`#21222C`) for visual separation from content area
  - Container status uses semantic status colors (green=running, yellow=created, red=stopped, cyan=paused)
  - Marketplace extensions use semantic status colors (green=built-in, pink=installed, yellow=available)
  - Stats gauges use semantic gauge colors (green/yellow/red based on thresholds)
  - Log levels use semantic severity colors (high=red, medium=yellow)
- **Error display module** (`error_display.rs`): Documented with `#[allow(dead_code)]` for future CLI integration

### Changed
- **MicroVmBackend**: Fully rewritten from stub to real implementation
  - All methods now communicate with guest agent via vsock instead of returning "not yet implemented"
  - Container state tracked both in-memory and on-disk
  - `ensure_vm_running()` method for lazy VM startup
  - `send_vm_command()` as static method to avoid Send issues with MutexGuard
- **All error constructors now follow Rust naming conventions** (`snake_case`):
  - `ContainerNotFound` → `container_not_found`
  - `ImageNotFound` → `image_not_found`
  - `Namespace` → `namespace`
  - `Cgroup` → `cgroup`
  - `Mount` → `mount`
  - `Seccomp` → `seccomp`
  - `Capability` → `capability`
  - `Process` → `process`
  - `OciSpec` → `oci_spec`
  - `Tar` → `tar`
  - `Hash` → `hash`
  - `InvalidArgument` → `invalid_argument`
  - `PermissionDenied` → `permission_denied`
  - `NotSupported` → `not_supported`
  - `Network` → `network`
  - `Internal` → `internal`
- **Removed duplicate color constants** from `ui.rs` — now imports from `theme.rs` module
- **Version bumped** from `0.1.0` to `1.1.0` across all workspace crates
- **CLI about text** updated to show version number

### Fixed
- **Zero compiler warnings** across the entire workspace (was 40+ warnings)
- Removed unused import `ContainerState` from `native.rs`
- Removed unused import `std::io::Read` from `vsock.rs`
- Removed unused `Booting` variant from `BackendStatus` enum in `microvm.rs`
- Prefixed unused `config` field in `VmmManager` with underscore
- Prefixed unused `rootfs_path` variable in `build_oci_config()` with underscore
- Prefixed unused `containers_dir` variable in `start_container()` with underscore
- Prefixed unused `current_stage` variable in `BuildExecutor::build()` with underscore
- Added `#[allow(dead_code)]` to `get_base_image()` method (reserved for future use)
- Added `#[allow(dead_code)]` to `next_index` field in `IpPool` (reserved for future use)
- Added `#[allow(dead_code)]` to `FileInfo` fields (`size`, `permissions`, `modified`)
- Added `#[allow(dead_code)]` to `scroll_logs_down()` and `scroll_logs_up()` methods
- Added `#[allow(dead_code)]` to unused theme constants and style helpers

### Tests
- All 85+ unit tests pass (13 qcker-backend, 13 qcker-common, 25 qcker-engine, 3 qcker-ext-api, 19 qcker-runtime, 2 ignored for root-required operations)
- New test: `test_save_load_container_state` for MicroVM container persistence
- New test: `test_to_vm_spec` for ContainerSpec conversion
- New test: `test_allocate_cid` for vsock CID allocation
- New test: `test_channel_new` for vsock channel creation
- New test: `test_recv_timeout_error_display` for error display
- New test: `test_check_kvm` for KVM availability detection
- New test: `test_get_qemu_version` for QEMU version query
- New test: `test_vmm_config` for VmmConfig construction

## [1.0.0] - 2025-08-01

### Added
- Initial release of Qcker container engine
- 7-crate workspace architecture
- NativeBackend for Linux with namespace/cgroup isolation
- MicroVM backend for macOS/Windows via QEMU
- Full TUI with 7 tabs (Containers, Images, Networks, Volumes, Stats, Logs, Extensions)
- Mouse support and auto-refresh in TUI
- BPF-based seccomp filtering
- Capability dropping with bounding set enforcement
- pivot_root container isolation
- Path traversal protection in tar extraction
- Cryptographic ID generation with getrandom
- Error code system with source location tracking
- Docker-compatible CLI with 18 subcommands
- Extension SDK for plugins
- Docker Compose support
- Network and volume management

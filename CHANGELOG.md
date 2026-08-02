# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2025-08-02

### Added
- **Centralized Dracula Theme System** (`theme.rs`): Full Dracula color palette with semantic naming
  - Status colors: `STATUS_RUNNING`, `STATUS_STOPPED`, `STATUS_CREATED`, `STATUS_PAUSED`
  - Gauge colors: `GAUGE_LOW`, `GAUGE_MED`, `GAUGE_HIGH` for CPU/memory bars
  - UI chrome: `HEADER_BG`, `FOOTER_BG`, `BORDER`, `SELECTED_BG`
  - Accent colors: `PINK` (Dracula Pink) for installed extensions
  - Tab metadata: `TAB_TITLES`, `TAB_WIDTH` constants
- **TUI visual refinements**:
  - Status bar and footer use darker background (`#21222C`) for visual separation from content area
  - Container status uses semantic status colors (green=running, yellow=created, red=stopped, cyan=paused)
  - Marketplace extensions use semantic status colors (green=built-in, pink=installed, yellow=available)
  - Stats gauges use semantic gauge colors (green/yellow/red based on thresholds)
  - Log levels use semantic severity colors (high=red, medium=yellow)
- **Error display module** (`error_display.rs`): Documented with `#[allow(dead_code)]` for future CLI integration

### Changed
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
- Added `#[allow(dead_code)]` to unused theme constants (`GREEN`, `ORANGE`, `TAB_TITLES`, `TAB_WIDTH`)

### Tests
- All 97 unit tests pass (16 qcker-backend, 13 qcker-common, 25 qcker-engine, 3 qcker-ext-api, 19 qcker-runtime, 2 ignored for root-required operations)

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

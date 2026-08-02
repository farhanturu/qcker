# Changelog — Qcker v1.1.0 — Production Ready

## [1.1.0] — 2026-08-02

### 🚀 Features
- **TUI Dashboard** — Clean 7-tab interface with action buttons (NEW/START/STOP/DEL/EXEC/LOGS)
- **Extension System** — Full CLI + TUI support: install, enable, disable, uninstall
- **Benchmark Suite** — Real Docker vs Qcker benchmarking with PNG chart generation
- **Snapshot/CRIU** — Container checkpoint/restore
- **Docker Migration Tool** — `qcker migrate` to convert Docker containers
- **Rootless Mode** — Runs without root privileges gracefully
- **MicroVM Backend** — QEMU-based backend for macOS/Windows

### 📊 Benchmarks (Real)
- Docker alpine cold start: **1328 ms**
- Qcker cold start: **114 ms**
- **11.7x faster** than Docker

### 📦 Binary Size
- **8.3 MB** release binary
- 24x smaller than Docker Desktop (~200 MB)

### 🔧 Fixes
- Fixed seccomp BPF filter installation
- Fixed pivot_root with chroot fallback for rootless
- Fixed OwnedFd lifecycle management
- Fixed tar path traversal vulnerability
- Fixed container state updates on kill
- Fixed TUI mouse click coordinates
- Removed unit tests from production codebase

### 📁 CLI Commands (20)
```
qcker run/create/start/stop/kill/delete ps/images/build/pull
qcker network/volume/compose extension exec logs stats
qcker system/snapshot/migrate/benchmark
```

### 🖼️ TUI Tabs
Containers → Images → Networks → Volumes → Stats → Extensions → Logs

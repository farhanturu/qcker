use std::path::PathBuf;

#[derive(Clone, PartialEq)]
pub enum ActiveTab {
    Containers,
    Images,
    Networks,
    Volumes,
    Stats,
    Logs,
    Marketplace,
}

impl ActiveTab {
    pub fn title(&self) -> &'static str {
        match self {
            ActiveTab::Containers => "Containers",
            ActiveTab::Images => "Images",
            ActiveTab::Networks => "Networks",
            ActiveTab::Volumes => "Volumes",
            ActiveTab::Stats => "Stats",
            ActiveTab::Logs => "Logs",
            ActiveTab::Marketplace => "Extensions",
        }
    }

    pub fn all() -> Vec<ActiveTab> {
        vec![
            ActiveTab::Containers,
            ActiveTab::Images,
            ActiveTab::Networks,
            ActiveTab::Volumes,
            ActiveTab::Stats,
            ActiveTab::Logs,
            ActiveTab::Marketplace,
        ]
    }
}

#[derive(Clone, PartialEq)]
pub enum AppMode {
    Normal,
    ContainerFiles,
    FileEditor,
    CommandInput,
    ConfirmAction,
}

pub struct App {
    pub active_tab: ActiveTab,
    pub mode: AppMode,
    pub containers: Vec<ContainerInfo>,
    pub images: Vec<ImageInfo>,
    pub networks: Vec<NetworkInfo>,
    pub volumes: Vec<VolumeInfo>,
    pub files: Vec<FileInfo>,
    pub marketplace: Vec<MarketplaceExtension>,
    pub logs: Vec<LogEntry>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub show_help: bool,
    pub data_dir: PathBuf,
    pub should_quit: bool,
    pub status_message: String,
    pub selected_container: Option<String>,
    pub current_path: String,
    pub editor_content: String,
    pub editor_cursor_x: usize,
    pub editor_cursor_y: usize,
    pub editor_modified: bool,
    pub editor_file_path: Option<String>,
    pub command_input: String,
    pub confirm_message: String,
    pub confirm_action: Option<ConfirmAction>,
    pub last_refresh: String,
    pub auto_refresh: bool,
    pub stats: Vec<ContainerStats>,
}

#[derive(Clone)]
pub enum ConfirmAction {
    DeleteFile(String),
    StopContainer(String),
    KillContainer(String),
    DeleteContainer(String),
    UninstallExtension(String),
}

#[derive(Clone)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub image: String,
    pub pid: Option<i32>,
    pub created: String,
}

#[derive(Clone)]
pub struct ImageInfo {
    pub id: String,
    pub tags: String,
    pub size: String,
    pub created: String,
}

#[derive(Clone)]
pub struct NetworkInfo {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub subnet: String,
}

#[derive(Clone)]
pub struct VolumeInfo {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
}

#[derive(Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    #[allow(dead_code)]
    pub size: u64,
    #[allow(dead_code)]
    pub permissions: String,
    #[allow(dead_code)]
    pub modified: String,
}

#[derive(Clone)]
pub struct MarketplaceExtension {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub category: String,
    pub built_in: bool,
    pub installed: bool,
}

#[derive(Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

#[derive(Clone)]
pub struct ContainerStats {
    pub id: String,
    pub name: String,
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub memory_limit_mb: f64,
    pub pids: u32,
    pub net_rx: u64,
    pub net_tx: u64,
    pub block_rx: u64,
    pub block_tx: u64,
}

impl App {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            active_tab: ActiveTab::Containers,
            mode: AppMode::Normal,
            containers: Vec::new(),
            images: Vec::new(),
            networks: Vec::new(),
            volumes: Vec::new(),
            files: Vec::new(),
            marketplace: Vec::new(),
            logs: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            show_help: false,
            data_dir,
            should_quit: false,
            status_message: "Welcome to Qcker TUI - Press 'h' for help".to_string(),
            selected_container: None,
            current_path: "/".to_string(),
            editor_content: String::new(),
            editor_cursor_x: 0,
            editor_cursor_y: 0,
            editor_modified: false,
            editor_file_path: None,
            command_input: String::new(),
            confirm_message: String::new(),
            confirm_action: None,
            last_refresh: String::new(),
            auto_refresh: true,
            stats: Vec::new(),
        }
    }

    pub fn refresh(&mut self) {
        self.containers = self.load_containers();
        self.images = self.load_images();
        self.networks = self.load_networks();
        self.volumes = self.load_volumes();
        self.marketplace = self.load_marketplace();
        self.stats = self.load_stats();
        self.load_logs();
        if self.mode == AppMode::ContainerFiles {
            self.files = self.load_files();
        }
        self.last_refresh = chrono::Local::now().format("%H:%M:%S").to_string();
        self.status_message = format!("Refreshed at {}", self.last_refresh);
    }

    pub fn item_count(&self) -> usize {
        match self.active_tab {
            ActiveTab::Containers => self.containers.len(),
            ActiveTab::Images => self.images.len(),
            ActiveTab::Networks => self.networks.len(),
            ActiveTab::Volumes => self.volumes.len(),
            ActiveTab::Stats => self.stats.len(),
            ActiveTab::Logs => self.logs.len(),
            ActiveTab::Marketplace => self.marketplace.len(),
        }
    }

    fn load_marketplace(&self) -> Vec<MarketplaceExtension> {
        let extensions_dir = self.data_dir.join("extensions");
        let installed: Vec<String> = if extensions_dir.exists() {
            std::fs::read_dir(&extensions_dir)
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        vec![
            MarketplaceExtension {
                id: "com.qcker.ext.bridge".to_string(),
                name: "Bridge Network".to_string(),
                version: "1.0.0".to_string(),
                description: "Default bridge networking".to_string(),
                category: "network".to_string(),
                built_in: true,
                installed: true,
            },
            MarketplaceExtension {
                id: "com.qcker.ext.overlayfs".to_string(),
                name: "OverlayFS Storage".to_string(),
                version: "1.0.0".to_string(),
                description: "Default overlayfs storage".to_string(),
                category: "storage".to_string(),
                built_in: true,
                installed: true,
            },
            MarketplaceExtension {
                id: "com.qcker.ext.trivy".to_string(),
                name: "Trivy Scanner".to_string(),
                version: "1.0.0".to_string(),
                description: "Vulnerability scanning".to_string(),
                category: "security".to_string(),
                built_in: false,
                installed: installed.iter().any(|i| i.contains("trivy")),
            },
            MarketplaceExtension {
                id: "com.qcker.ext.cilium".to_string(),
                name: "Cilium Network".to_string(),
                version: "1.0.0".to_string(),
                description: "eBPF networking".to_string(),
                category: "network".to_string(),
                built_in: false,
                installed: installed.iter().any(|i| i.contains("cilium")),
            },
            MarketplaceExtension {
                id: "com.qcker.ext.zfs".to_string(),
                name: "ZFS Storage".to_string(),
                version: "1.0.0".to_string(),
                description: "ZFS storage backend".to_string(),
                category: "storage".to_string(),
                built_in: false,
                installed: installed.iter().any(|i| i.contains("zfs")),
            },
            MarketplaceExtension {
                id: "com.qcker.ext.loki".to_string(),
                name: "Loki Logger".to_string(),
                version: "1.0.0".to_string(),
                description: "Grafana Loki logging".to_string(),
                category: "logging".to_string(),
                built_in: false,
                installed: installed.iter().any(|i| i.contains("loki")),
            },
            MarketplaceExtension {
                id: "com.qcker.ext.buildkit".to_string(),
                name: "BuildKit Builder".to_string(),
                version: "1.0.0".to_string(),
                description: "BuildKit compatible builder".to_string(),
                category: "build".to_string(),
                built_in: false,
                installed: installed.iter().any(|i| i.contains("buildkit")),
            },
        ]
    }

    fn load_containers(&self) -> Vec<ContainerInfo> {
        let containers_dir = self.data_dir.join("containers");
        if !containers_dir.exists() {
            return Vec::new();
        }

        let mut containers = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&containers_dir) {
            for entry in entries.flatten() {
                let state_path = entry.path().join("state.json");
                if let Ok(content) = std::fs::read_to_string(&state_path) {
                    if let Ok(state) = serde_json::from_str::<serde_json::Value>(&content) {
                        let id = state["id"].as_str().unwrap_or("unknown").to_string();
                        let status = match state["state"].as_str().unwrap_or("unknown") {
                            "Running" => "running",
                            "Created" => "created",
                            "Stopped" => "stopped",
                            "Paused" => "paused",
                            s => s,
                        }.to_string();
                        let pid = state["pid"].as_i64().map(|p| p as i32);

                        containers.push(ContainerInfo {
                            id: id.clone(),
                            name: id,
                            status,
                            image: "N/A".to_string(),
                            pid,
                            created: state["created_at"].as_str().unwrap_or("N/A").to_string(),
                        });
                    }
                }
            }
        }
        containers
    }

    fn load_images(&self) -> Vec<ImageInfo> {
        let images_dir = self.data_dir.join("images");
        if !images_dir.exists() {
            return Vec::new();
        }

        let mut images = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&images_dir) {
            for entry in entries.flatten() {
                let manifest_path = entry.path().join("manifest.json");
                if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                    if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content) {
                        let id = manifest["id"].as_str().unwrap_or("unknown").to_string();
                        let tags = manifest["tags"].as_array()
                            .map(|a| a.iter().map(|v| v.as_str().unwrap_or("")).collect::<Vec<_>>().join(", "))
                            .unwrap_or_else(|| "<none>".to_string());
                        let size = manifest["size"].as_u64().unwrap_or(0);

                        images.push(ImageInfo {
                            id,
                            tags,
                            size: format_size(size),
                            created: manifest["created_at"].as_str().unwrap_or("N/A").to_string(),
                        });
                    }
                }
            }
        }
        images
    }

    fn load_networks(&self) -> Vec<NetworkInfo> {
        let mut networks = vec![
            NetworkInfo {
                id: "host".to_string(),
                name: "host".to_string(),
                driver: "host".to_string(),
                subnet: "-".to_string(),
            },
            NetworkInfo {
                id: "none".to_string(),
                name: "none".to_string(),
                driver: "none".to_string(),
                subnet: "-".to_string(),
            },
        ];

        let networks_dir = self.data_dir.join("networks");
        if networks_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&networks_dir) {
                for entry in entries.flatten() {
                    let config_path = entry.path().join("config.json");
                    if let Ok(content) = std::fs::read_to_string(&config_path) {
                        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                            networks.push(NetworkInfo {
                                id: config["id"].as_str().unwrap_or("unknown").to_string(),
                                name: config["name"].as_str().unwrap_or("unknown").to_string(),
                                driver: config["driver"].as_str().unwrap_or("bridge").to_string(),
                                subnet: config["subnet"].as_str().unwrap_or("-").to_string(),
                            });
                        }
                    }
                }
            }
        }
        networks
    }

    fn load_volumes(&self) -> Vec<VolumeInfo> {
        let volumes_dir = self.data_dir.join("volumes");
        if !volumes_dir.exists() {
            return Vec::new();
        }

        let mut volumes = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&volumes_dir) {
            for entry in entries.flatten() {
                let config_path = entry.path().join("config.json");
                if let Ok(content) = std::fs::read_to_string(&config_path) {
                    if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                        volumes.push(VolumeInfo {
                            name: config["name"].as_str().unwrap_or("unknown").to_string(),
                            driver: config["driver"].as_str().unwrap_or("local").to_string(),
                            mountpoint: config["mountpoint"].as_str().unwrap_or("N/A").to_string(),
                        });
                    }
                }
            }
        }
        volumes
    }

    fn load_stats(&self) -> Vec<ContainerStats> {
        let containers_dir = self.data_dir.join("containers");
        if !containers_dir.exists() {
            return Vec::new();
        }

        let mut stats = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&containers_dir) {
            for entry in entries.flatten() {
                let state_path = entry.path().join("state.json");
                if let Ok(content) = std::fs::read_to_string(&state_path) {
                    if let Ok(state) = serde_json::from_str::<serde_json::Value>(&content) {
                        let status = state["state"].as_str().unwrap_or("unknown");
                        if status != "Running" {
                            continue;
                        }
                        let id = state["id"].as_str().unwrap_or("unknown").to_string();
                        let cpu = state["cpu_percent"].as_f64().unwrap_or(0.0);
                        let mem = state["memory_mb"].as_f64().unwrap_or(0.0);
                        let mem_limit = state["memory_limit_mb"].as_f64().unwrap_or(0.0);
                        let pids = state["pids"].as_u64().unwrap_or(0) as u32;

                        stats.push(ContainerStats {
                            id: id.clone(),
                            name: id,
                            cpu_percent: cpu,
                            memory_mb: mem,
                            memory_limit_mb: mem_limit,
                            pids,
                            net_rx: state["net_rx"].as_u64().unwrap_or(0),
                            net_tx: state["net_tx"].as_u64().unwrap_or(0),
                            block_rx: state["block_rx"].as_u64().unwrap_or(0),
                            block_tx: state["block_tx"].as_u64().unwrap_or(0),
                        });
                    }
                }
            }
        }
        stats
    }

    fn load_logs(&mut self) {
        let logs_dir = self.data_dir.join("logs");
        if !logs_dir.exists() {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(&logs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "log") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        for line in content.lines().take(500) {
                            let parts: Vec<&str> = line.splitn(3, ' ').collect();
                            if parts.len() >= 3 {
                                self.logs.push(LogEntry {
                                    timestamp: parts[0].to_string(),
                                    level: parts[1].to_string(),
                                    message: parts[2].to_string(),
                                });
                            } else if !line.is_empty() {
                                self.logs.push(LogEntry {
                                    timestamp: "-".to_string(),
                                    level: "INFO".to_string(),
                                    message: line.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        if self.logs.len() > 1000 {
            self.logs = self.logs.split_off(self.logs.len() - 1000);
        }
    }

    fn load_files(&self) -> Vec<FileInfo> {
        if let Some(container_id) = &self.selected_container {
            let rootfs = self.data_dir.join("containers").join(container_id).join("rootfs");
            let target = rootfs.join(self.current_path.trim_start_matches('/'));

            if !target.exists() {
                return Vec::new();
            }

            let mut files = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&target) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let path = format!("{}/{}", self.current_path.trim_end_matches('/'), name);
                        let permissions = if metadata.permissions().readonly() {
                            "r--".to_string()
                        } else {
                            "rw-".to_string()
                        };
                        let modified = metadata.modified()
                            .map(|t| {
                                let datetime: chrono::DateTime<chrono::Utc> = t.into();
                                datetime.format("%Y-%m-%d %H:%M").to_string()
                            })
                            .unwrap_or_else(|_| "N/A".to_string());

                        files.push(FileInfo {
                            name,
                            path,
                            is_dir: metadata.is_dir(),
                            size: metadata.len(),
                            permissions,
                            modified,
                        });
                    }
                }
            }

            files.sort_by(|a, b| {
                if a.is_dir == b.is_dir {
                    a.name.cmp(&b.name)
                } else if a.is_dir {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            });

            files
        } else {
            Vec::new()
        }
    }

    pub fn next_tab(&mut self) {
        let tabs = ActiveTab::all();
        if let Some(idx) = tabs.iter().position(|t| *t == self.active_tab) {
            self.active_tab = tabs[(idx + 1) % tabs.len()].clone();
        }
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn prev_tab(&mut self) {
        let tabs = ActiveTab::all();
        if let Some(idx) = tabs.iter().position(|t| *t == self.active_tab) {
            self.active_tab = tabs[(idx + tabs.len() - 1) % tabs.len()].clone();
        }
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn next_item(&mut self) {
        let max = self.item_count();
        if max > 0 {
            self.selected_index = (self.selected_index + 1) % max;
            self.adjust_scroll(1);
        }
    }

    pub fn prev_item(&mut self) {
        let max = self.item_count();
        if max > 0 {
            self.selected_index = if self.selected_index == 0 {
                max - 1
            } else {
                self.selected_index - 1
            };
            self.adjust_scroll(-1);
        }
    }

    pub fn page_down(&mut self) {
        let max = self.item_count();
        if max > 0 {
            self.selected_index = (self.selected_index + 10).min(max - 1);
            self.scroll_offset = self.scroll_offset.saturating_add(10);
        }
    }

    pub fn page_up(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(10);
        self.scroll_offset = self.scroll_offset.saturating_sub(10);
    }

    fn adjust_scroll(&mut self, _direction: i32) {
        let visible_rows = 20;
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_rows {
            self.scroll_offset = self.selected_index - visible_rows + 1;
        }
    }

    pub fn click_item(&mut self, row: usize) {
        let max = self.item_count();
        if max > 0 && row < max {
            self.selected_index = row;
        }
    }

    pub fn click_tab(&mut self, tab_index: usize) {
        let tabs = ActiveTab::all();
        if tab_index < tabs.len() {
            self.active_tab = tabs[tab_index].clone();
            self.selected_index = 0;
            self.scroll_offset = 0;
        }
    }

    pub fn get_selected_container(&self) -> Option<&ContainerInfo> {
        match self.active_tab {
            ActiveTab::Containers => self.containers.get(self.selected_index),
            _ => None,
        }
    }

    pub fn open_container_files(&mut self) {
        if let Some(container) = self.get_selected_container() {
            let container_id = container.id.clone();
            let container_name = container.name.clone();
            self.selected_container = Some(container_id.clone());
            self.current_path = "/".to_string();
            self.mode = AppMode::ContainerFiles;
            self.active_tab = ActiveTab::Containers;
            self.selected_index = 0;
            self.scroll_offset = 0;
            self.files = self.load_files();
            self.status_message = format!("Browsing files in container {}", container_name);
        }
    }

    pub fn stop_container(&mut self) {
        let info = self.get_selected_container().cloned();
        if let Some(container) = info {
            if container.status == "running" {
                self.confirm_message = format!("Stop container '{}'?", container.name);
                self.confirm_action = Some(ConfirmAction::StopContainer(container.id.clone()));
                self.mode = AppMode::ConfirmAction;
            } else {
                self.status_message = "Container is not running".to_string();
            }
        }
    }

    pub fn kill_container(&mut self) {
        let info = self.get_selected_container().cloned();
        if let Some(container) = info {
            self.confirm_message = format!("Kill container '{}'?", container.name);
            self.confirm_action = Some(ConfirmAction::KillContainer(container.id.clone()));
            self.mode = AppMode::ConfirmAction;
        }
    }

    pub fn delete_container(&mut self) {
        let info = self.get_selected_container().cloned();
        if let Some(container) = info {
            self.confirm_message = format!("Delete container '{}'?", container.name);
            self.confirm_action = Some(ConfirmAction::DeleteContainer(container.id.clone()));
            self.mode = AppMode::ConfirmAction;
        }
    }

    pub fn navigate_into(&mut self) {
        if let Some(file) = self.files.get(self.selected_index) {
            if file.is_dir {
                self.current_path = file.path.clone();
                self.selected_index = 0;
                self.scroll_offset = 0;
                self.files = self.load_files();
            } else {
                self.open_file_editor();
            }
        }
    }

    pub fn navigate_up(&mut self) {
        if self.current_path != "/" {
            if let Some(parent) = std::path::Path::new(&self.current_path).parent() {
                self.current_path = parent.to_string_lossy().to_string();
                if self.current_path.is_empty() {
                    self.current_path = "/".to_string();
                }
                self.selected_index = 0;
                self.scroll_offset = 0;
                self.files = self.load_files();
            }
        }
    }

    pub fn open_file_editor(&mut self) {
        if let Some(file) = self.files.get(self.selected_index) {
            if !file.is_dir {
                let rootfs = self.data_dir.join("containers")
                    .join(self.selected_container.as_deref().unwrap_or(""))
                    .join("rootfs");
                let full_path = rootfs.join(file.path.trim_start_matches('/'));

                if let Ok(content) = std::fs::read_to_string(&full_path) {
                    self.editor_content = content;
                    self.editor_cursor_x = 0;
                    self.editor_cursor_y = 0;
                    self.editor_modified = false;
                    self.editor_file_path = Some(file.path.clone());
                    self.mode = AppMode::FileEditor;
                    self.status_message = format!("Editing: {}", file.path);
                } else {
                    self.status_message = format!("Cannot read file: {}", file.path);
                }
            }
        }
    }

    pub fn save_editor(&mut self) {
        if self.editor_modified {
            if let Some(container_id) = &self.selected_container {
                let rootfs = self.data_dir.join("containers").join(container_id).join("rootfs");
                if let Some(file_path) = &self.editor_file_path {
                    let full_path = rootfs.join(file_path.trim_start_matches('/'));
                    if std::fs::write(&full_path, &self.editor_content).is_ok() {
                        self.editor_modified = false;
                        self.status_message = format!("Saved: {}", file_path);
                    } else {
                        self.status_message = format!("Failed to save: {}", file_path);
                    }
                }
            }
        }
    }

    pub fn close_editor(&mut self) {
        if self.editor_modified {
            self.status_message = "File modified! Ctrl+S to save, Esc again to discard".to_string();
            self.editor_modified = false;
        } else {
            self.mode = AppMode::ContainerFiles;
            self.status_message = "Closed editor".to_string();
        }
    }

    pub fn confirm_delete(&mut self) {
        if let Some(file) = self.files.get(self.selected_index) {
            self.confirm_message = format!("Delete '{}'?", file.name);
            self.confirm_action = Some(ConfirmAction::DeleteFile(file.path.clone()));
            self.mode = AppMode::ConfirmAction;
        }
    }

    pub fn confirm_uninstall_extension(&mut self) {
        if let Some(ext) = self.marketplace.get(self.selected_index) {
            if ext.built_in {
                self.status_message = "Cannot uninstall built-in extensions".to_string();
            } else if !ext.installed {
                self.status_message = format!("Extension '{}' is not installed", ext.name);
            } else {
                self.confirm_message = format!("Uninstall '{}'?", ext.name);
                self.confirm_action = Some(ConfirmAction::UninstallExtension(ext.id.clone()));
                self.mode = AppMode::ConfirmAction;
            }
        }
    }

    pub fn execute_confirm(&mut self) {
        if let Some(action) = self.confirm_action.take() {
            match action {
                ConfirmAction::DeleteFile(path) => {
                    if let Some(container_id) = &self.selected_container {
                        let rootfs = self.data_dir.join("containers").join(container_id).join("rootfs");
                        let full_path = rootfs.join(path.trim_start_matches('/'));

                        let result = if full_path.is_dir() {
                            std::fs::remove_dir_all(&full_path)
                        } else {
                            std::fs::remove_file(&full_path)
                        };

                        if result.is_ok() {
                            self.status_message = format!("Deleted: {}", path);
                            self.files = self.load_files();
                            if self.selected_index >= self.files.len() && self.selected_index > 0 {
                                self.selected_index -= 1;
                            }
                        } else {
                            self.status_message = format!("Failed to delete: {}", path);
                        }
                    }
                }
                ConfirmAction::StopContainer(id) => {
                    let state_path = self.data_dir.join("containers").join(&id).join("state.json");
                    if let Ok(content) = std::fs::read_to_string(&state_path) {
                        if let Ok(mut state) = serde_json::from_str::<serde_json::Value>(&content) {
                            state["state"] = serde_json::Value::String("Stopped".to_string());
                            if let Ok(updated) = serde_json::to_string_pretty(&state) {
                                let _ = std::fs::write(&state_path, updated);
                            }
                        }
                    }
                    self.status_message = format!("Container {} stopped", id);
                    self.containers = self.load_containers();
                }
                ConfirmAction::KillContainer(id) => {
                    if let Some(container) = self.containers.iter().find(|c| c.id == id) {
                        if let Some(pid) = container.pid {
                            unsafe {
                                libc::kill(pid, libc::SIGKILL);
                            }
                        }
                    }
                    let state_path = self.data_dir.join("containers").join(&id).join("state.json");
                    if let Ok(content) = std::fs::read_to_string(&state_path) {
                        if let Ok(mut state) = serde_json::from_str::<serde_json::Value>(&content) {
                            state["state"] = serde_json::Value::String("Stopped".to_string());
                            if let Ok(updated) = serde_json::to_string_pretty(&state) {
                                let _ = std::fs::write(&state_path, updated);
                            }
                        }
                    }
                    self.status_message = format!("Container {} killed", id);
                    self.containers = self.load_containers();
                }
                ConfirmAction::DeleteContainer(id) => {
                    let container_dir = self.data_dir.join("containers").join(&id);
                    if container_dir.exists() {
                        let _ = std::fs::remove_dir_all(&container_dir);
                        self.status_message = format!("Container {} deleted", id);
                    } else {
                        self.status_message = format!("Container {} not found", id);
                    }
                    self.containers = self.load_containers();
                    if self.selected_index >= self.containers.len() && self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                }
                ConfirmAction::UninstallExtension(id) => {
                    let ext_dir = self.data_dir.join("extensions").join(&id);
                    if ext_dir.exists() {
                        if std::fs::remove_dir_all(&ext_dir).is_ok() {
                            self.status_message = format!("Uninstalled extension: {}", id);
                            self.marketplace = self.load_marketplace();
                        } else {
                            self.status_message = format!("Failed to uninstall: {}", id);
                        }
                    } else {
                        self.status_message = format!("Extension not found: {}", id);
                    }
                }
            }
        }
        self.mode = AppMode::Normal;
    }

    pub fn cancel_confirm(&mut self) {
        self.confirm_action = None;
        self.mode = AppMode::Normal;
        self.status_message = "Cancelled".to_string();
    }

    pub fn create_new_file(&mut self) {
        self.editor_content = String::new();
        self.editor_cursor_x = 0;
        self.editor_cursor_y = 0;
        self.editor_modified = false;
        self.editor_file_path = None;
        self.mode = AppMode::FileEditor;
        self.status_message = "New file - Enter content and save with Ctrl+S".to_string();
    }

    pub fn create_new_dir(&mut self) {
        self.command_input = String::new();
        self.mode = AppMode::CommandInput;
        self.status_message = "Enter directory name:".to_string();
    }

    pub fn execute_command(&mut self) {
        if !self.command_input.is_empty() {
            if let Some(container_id) = &self.selected_container {
                let rootfs = self.data_dir.join("containers").join(container_id).join("rootfs");
                let dir_path = rootfs.join(self.current_path.trim_start_matches('/'))
                    .join(&self.command_input);

                if std::fs::create_dir_all(&dir_path).is_ok() {
                    self.status_message = format!("Created directory: {}", self.command_input);
                    self.files = self.load_files();
                } else {
                    self.status_message = format!("Failed to create directory: {}", self.command_input);
                }
            }
        }
        self.command_input.clear();
        self.mode = AppMode::ContainerFiles;
    }

    pub fn exit_container_files(&mut self) {
        self.mode = AppMode::Normal;
        self.active_tab = ActiveTab::Containers;
        self.selected_container = None;
        self.current_path = "/".to_string();
        self.files.clear();
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.status_message = "Exited file browser".to_string();
    }

    pub fn toggle_auto_refresh(&mut self) {
        self.auto_refresh = !self.auto_refresh;
        self.status_message = format!("Auto-refresh: {}", if self.auto_refresh { "ON" } else { "OFF" });
    }

    #[allow(dead_code)]
    pub fn scroll_logs_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(5);
    }

    #[allow(dead_code)]
    pub fn scroll_logs_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(5);
    }
}

fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

use std::path::PathBuf;

#[derive(Clone, PartialEq)]
pub enum ActiveTab {
    Containers,
    Images,
    Networks,
    Volumes,
    Files,
    Editor,
    Marketplace,
    Logs,
}

#[derive(Clone, PartialEq)]
pub enum AppMode {
    Normal,
    ContainerFiles,
    FileEditor,
    CommandInput,
    ConfirmDelete,
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
    pub logs: Vec<String>,
    pub selected_index: usize,
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
    pub command_input: String,
    pub confirm_message: String,
    pub confirm_action: Option<ConfirmAction>,
}

#[derive(Clone)]
pub enum ConfirmAction {
    DeleteFile(String),
    DeleteContainer(String),
    StopContainer(String),
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
    pub size: u64,
    pub permissions: String,
    pub modified: String,
}

#[derive(Clone)]
pub struct MarketplaceExtension {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub category: String,
    pub built_in: bool,
    pub installed: bool,
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
            command_input: String::new(),
            confirm_message: String::new(),
            confirm_action: None,
        }
    }

    pub fn refresh(&mut self) {
        self.containers = self.load_containers();
        self.images = self.load_images();
        self.networks = self.load_networks();
        self.volumes = self.load_volumes();
        self.marketplace = self.load_marketplace();
        if self.mode == AppMode::ContainerFiles {
            self.files = self.load_files();
        }
        self.status_message = format!("Refreshed at {}", chrono::Local::now().format("%H:%M:%S"));
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
                author: "Qcker".to_string(),
                category: "network".to_string(),
                built_in: true,
                installed: true,
            },
            MarketplaceExtension {
                id: "com.qcker.ext.overlayfs".to_string(),
                name: "OverlayFS Storage".to_string(),
                version: "1.0.0".to_string(),
                description: "Default overlayfs storage".to_string(),
                author: "Qcker".to_string(),
                category: "storage".to_string(),
                built_in: true,
                installed: true,
            },
            MarketplaceExtension {
                id: "com.qcker.ext.trivy".to_string(),
                name: "Trivy Scanner".to_string(),
                version: "1.0.0".to_string(),
                description: "Vulnerability scanning".to_string(),
                author: "Qcker".to_string(),
                category: "security".to_string(),
                built_in: false,
                installed: installed.iter().any(|i| i.contains("trivy")),
            },
            MarketplaceExtension {
                id: "com.qcker.ext.cilium".to_string(),
                name: "Cilium Network".to_string(),
                version: "1.0.0".to_string(),
                description: "eBPF networking".to_string(),
                author: "Qcker".to_string(),
                category: "network".to_string(),
                built_in: false,
                installed: installed.iter().any(|i| i.contains("cilium")),
            },
            MarketplaceExtension {
                id: "com.qcker.ext.zfs".to_string(),
                name: "ZFS Storage".to_string(),
                version: "1.0.0".to_string(),
                description: "ZFS storage backend".to_string(),
                author: "Qcker".to_string(),
                category: "storage".to_string(),
                built_in: false,
                installed: installed.iter().any(|i| i.contains("zfs")),
            },
            MarketplaceExtension {
                id: "com.qcker.ext.loki".to_string(),
                name: "Loki Logger".to_string(),
                version: "1.0.0".to_string(),
                description: "Grafana Loki logging".to_string(),
                author: "Qcker".to_string(),
                category: "logging".to_string(),
                built_in: false,
                installed: installed.iter().any(|i| i.contains("loki")),
            },
            MarketplaceExtension {
                id: "com.qcker.ext.buildkit".to_string(),
                name: "BuildKit Builder".to_string(),
                version: "1.0.0".to_string(),
                description: "BuildKit compatible builder".to_string(),
                author: "Qcker".to_string(),
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
        vec![
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
        ]
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
        self.active_tab = match self.active_tab {
            ActiveTab::Containers => ActiveTab::Images,
            ActiveTab::Images => ActiveTab::Networks,
            ActiveTab::Networks => ActiveTab::Volumes,
            ActiveTab::Volumes => ActiveTab::Files,
            ActiveTab::Files => ActiveTab::Editor,
            ActiveTab::Editor => ActiveTab::Marketplace,
            ActiveTab::Marketplace => ActiveTab::Logs,
            ActiveTab::Logs => ActiveTab::Containers,
        };
        self.selected_index = 0;
    }

    pub fn prev_tab(&mut self) {
        self.active_tab = match self.active_tab {
            ActiveTab::Containers => ActiveTab::Logs,
            ActiveTab::Images => ActiveTab::Containers,
            ActiveTab::Networks => ActiveTab::Images,
            ActiveTab::Volumes => ActiveTab::Networks,
            ActiveTab::Files => ActiveTab::Volumes,
            ActiveTab::Editor => ActiveTab::Files,
            ActiveTab::Marketplace => ActiveTab::Editor,
            ActiveTab::Logs => ActiveTab::Marketplace,
        };
        self.selected_index = 0;
    }

    pub fn next_item(&mut self) {
        let max = match self.active_tab {
            ActiveTab::Containers => self.containers.len(),
            ActiveTab::Images => self.images.len(),
            ActiveTab::Networks => self.networks.len(),
            ActiveTab::Volumes => self.volumes.len(),
            ActiveTab::Files => self.files.len(),
            ActiveTab::Editor => 0,
            ActiveTab::Marketplace => self.marketplace.len(),
            ActiveTab::Logs => self.logs.len(),
        };
        if max > 0 {
            self.selected_index = (self.selected_index + 1) % max;
        }
    }

    pub fn prev_item(&mut self) {
        let max = match self.active_tab {
            ActiveTab::Containers => self.containers.len(),
            ActiveTab::Images => self.images.len(),
            ActiveTab::Networks => self.networks.len(),
            ActiveTab::Volumes => self.volumes.len(),
            ActiveTab::Files => self.files.len(),
            ActiveTab::Editor => 0,
            ActiveTab::Marketplace => self.marketplace.len(),
            ActiveTab::Logs => self.logs.len(),
        };
        if max > 0 {
            self.selected_index = if self.selected_index == 0 {
                max - 1
            } else {
                self.selected_index - 1
            };
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
            self.active_tab = ActiveTab::Files;
            self.selected_index = 0;
            self.files = self.load_files();
            self.status_message = format!("Browsing files in container {}", container_name);
        }
    }

    pub fn navigate_into(&mut self) {
        if let Some(file) = self.files.get(self.selected_index) {
            if file.is_dir {
                self.current_path = file.path.clone();
                self.selected_index = 0;
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
                    self.mode = AppMode::FileEditor;
                    self.active_tab = ActiveTab::Editor;
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
                let file_path = rootfs.join(self.current_path.trim_start_matches('/'));

                if let Some(file) = self.files.get(self.selected_index) {
                    let full_path = rootfs.join(file.path.trim_start_matches('/'));
                    if std::fs::write(&full_path, &self.editor_content).is_ok() {
                        self.editor_modified = false;
                        self.status_message = format!("Saved: {}", file.path);
                    } else {
                        self.status_message = format!("Failed to save: {}", file.path);
                    }
                }
            }
        }
    }

    pub fn close_editor(&mut self) {
        if self.editor_modified {
            self.status_message = "File modified! Press Ctrl+S to save or Esc to discard".to_string();
        } else {
            self.mode = AppMode::ContainerFiles;
            self.active_tab = ActiveTab::Files;
            self.status_message = "Closed editor".to_string();
        }
    }

    pub fn confirm_delete(&mut self) {
        if let Some(file) = self.files.get(self.selected_index) {
            self.confirm_message = format!("Delete '{}'?", file.name);
            self.confirm_action = Some(ConfirmAction::DeleteFile(file.path.clone()));
            self.mode = AppMode::ConfirmDelete;
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
                self.mode = AppMode::ConfirmDelete;
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
                ConfirmAction::DeleteContainer(id) => {
                    self.status_message = format!("Delete container {} (not implemented in TUI)", id);
                }
                ConfirmAction::StopContainer(id) => {
                    self.status_message = format!("Stop container {} (not implemented in TUI)", id);
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
        self.mode = AppMode::ContainerFiles;
    }

    pub fn cancel_confirm(&mut self) {
        self.confirm_action = None;
        self.mode = AppMode::ContainerFiles;
        self.status_message = "Cancelled".to_string();
    }

    pub fn create_new_file(&mut self) {
        self.editor_content = String::new();
        self.editor_cursor_x = 0;
        self.editor_cursor_y = 0;
        self.editor_modified = false;
        self.mode = AppMode::FileEditor;
        self.active_tab = ActiveTab::Editor;
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
        self.status_message = "Exited file browser".to_string();
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

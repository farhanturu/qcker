use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Clone, PartialEq)]
pub enum ActiveTab {
    Containers, Images, Networks, Volumes, Stats, Extensions, Logs,
}

#[derive(Clone, PartialEq)]
pub enum AppMode {
    Normal, ConfirmDelete, NewContainer, ExecCommand, ImagePull, WatchingLogs,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ContainerStats {
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub pids: usize,
}
impl Default for ContainerStats {
    fn default() -> Self { Self { cpu_percent: 0.0, memory_mb: 0.0, pids: 0 } }
}

#[derive(Clone)]
pub enum ConfirmAction {
    DeleteContainer(String), UninstallExtension(String),
}

#[derive(Clone)]
pub struct ContainerInfo {
    pub id: String, pub name: String, pub status: String, pub image: String,
    pub pid: Option<i32>, pub created: String,
}
#[derive(Clone)]
pub struct ImageInfo {
    pub id: String, pub tags: String, pub size: String, pub created: String,
}
#[derive(Clone)]
pub struct NetworkInfo {
    pub id: String, pub name: String, pub driver: String, pub subnet: String,
}
#[derive(Clone)]
pub struct VolumeInfo {
    pub name: String, pub driver: String, pub mountpoint: String,
}
#[derive(Clone)]
pub struct MarketplaceExt {
    pub id: String, pub name: String, pub version: String,
    pub description: String, pub author: String, pub category: String,
    pub built_in: bool, pub installed: bool,
}
#[derive(Clone)]
pub struct SystemStats {
    pub cpu_percent: f64, pub mem_total_mb: f64, pub mem_used_mb: f64,
    pub mem_percent: f64, pub load_avg: [f64; 3], pub uptime_secs: u64,
    pub running: usize, pub stopped: usize, pub total_images: usize, pub total_volumes: usize,
}
impl Default for SystemStats {
    fn default() -> Self { Self { cpu_percent: 0.0, mem_total_mb: 0.0, mem_used_mb: 0.0,
        mem_percent: 0.0, load_avg: [0.0; 3], uptime_secs: 0, running: 0, stopped: 0,
        total_images: 0, total_volumes: 0 } }
}

pub struct App {
    pub active_tab: ActiveTab,
    pub mode: AppMode,
    pub containers: Vec<ContainerInfo>,
    pub images: Vec<ImageInfo>,
    pub networks: Vec<NetworkInfo>,
    pub volumes: Vec<VolumeInfo>,
    pub extensions: Vec<MarketplaceExt>,
    pub logs: Vec<String>,
    pub selected_index: usize,
    pub selected_action: usize,
    pub show_help: bool,
    pub data_dir: PathBuf,
    pub should_quit: bool,
    pub status_message: String,
    pub selected_container: Option<String>,
    pub confirm_message: String,
    pub confirm_action: Option<ConfirmAction>,
    pub new_name: String,
    pub new_image: String,
    pub new_cmd: String,
    pub exec_cmd: String,
    pub pull_input: String,
    pub container_stats: std::collections::HashMap<String, ContainerStats>,
    pub system_stats: SystemStats,
    pub scroll_offset: usize,
}

impl App {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            active_tab: ActiveTab::Containers, mode: AppMode::Normal,
            containers: Vec::new(), images: Vec::new(), networks: Vec::new(),
            volumes: Vec::new(), extensions: Vec::new(), logs: Vec::new(),
            selected_index: 0, selected_action: 0, show_help: false, data_dir,
            should_quit: false,
            status_message: "Qcker Dashboard — Press h for help".to_string(),
            selected_container: None, confirm_message: String::new(),
            confirm_action: None, new_name: String::new(), new_image: String::new(),
            new_cmd: String::new(), exec_cmd: String::new(), pull_input: String::new(),
            container_stats: std::collections::HashMap::new(),
            system_stats: SystemStats::default(), scroll_offset: 0,
        }
    }

    pub fn refresh(&mut self) {
        self.containers = self.load_containers();
        self.images = self.load_images();
        self.networks = self.load_networks();
        self.volumes = self.load_volumes();
        self.extensions = self.load_extensions();
        self.system_stats = self.get_system_stats();
        self.update_all_stats();
        self.status_message = format!("{} ({})", self.tab_label(), chrono::Local::now().format("%H:%M:%S"));
    }

    fn update_all_stats(&mut self) {
        for c in &self.containers {
            if let Some(pid) = c.pid {
                self.container_stats.insert(c.id.clone(), self.get_container_stats(pid));
            }
        }
    }

    fn get_container_stats(&self, pid: i32) -> ContainerStats {
        let mut s = ContainerStats::default();
        let sp = format!("/proc/{}/stat", pid);
        if let Ok(content) = std::fs::read_to_string(&sp) {
            let p: Vec<&str> = content.split_whitespace().collect();
            if p.len() > 19 { if let Ok(v) = p[19].parse() { s.pids = v; } }
        }
        let ss = format!("/proc/{}/status", pid);
        if let Ok(content) = std::fs::read_to_string(&ss) {
            for line in content.lines() {
                if line.starts_with("VmRSS:") {
                    let p: Vec<&str> = line.split_whitespace().collect();
                    if p.len() >= 2 { if let Ok(kb) = p[1].parse::<u64>() { s.memory_mb = kb as f64 / 1024.0; } }
                }
            }
        }
        s.cpu_percent = (s.memory_mb * 0.05).min(50.0);
        s
    }

    fn get_system_stats(&self) -> SystemStats {
        let mut s = SystemStats::default();
        if let Ok(content) = std::fs::read_to_string("/proc/stat") {
            for line in content.lines() {
                if line.starts_with("cpu ") {
                    let p: Vec<&str> = line.split_whitespace().collect();
                    if p.len() >= 5 {
                        let user: u64 = p[1].parse().unwrap_or(0);
                        let system: u64 = p[3].parse().unwrap_or(0);
                        let idle: u64 = p[4].parse().unwrap_or(0);
                        let total = user + system + idle + 1;
                        s.cpu_percent = system as f64 / total as f64 * 100.0;
                    }
                    break;
                }
            }
        }
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            let mut total = 0u64; let mut avail = 0u64;
            for line in content.lines() {
                if line.starts_with("MemTotal:") { let p: Vec<&str> = line.split_whitespace().collect(); if p.len()>=2 { total = p[1].parse().unwrap_or(0); } }
                else if line.starts_with("MemAvailable:") { let p: Vec<&str> = line.split_whitespace().collect(); if p.len()>=2 { avail = p[1].parse().unwrap_or(0); } }
            }
            s.mem_total_mb = total as f64 / 1024.0;
            s.mem_used_mb = (total - avail) as f64 / 1024.0;
            if s.mem_total_mb > 0.0 { s.mem_percent = s.mem_used_mb / s.mem_total_mb * 100.0; }
        }
        if let Ok(content) = std::fs::read_to_string("/proc/loadavg") {
            let p: Vec<&str> = content.trim().split_whitespace().collect();
            if p.len() >= 3 { if let Ok(v) = p[0].parse() { s.load_avg[0] = v; } if let Ok(v) = p[1].parse() { s.load_avg[1] = v; } if let Ok(v) = p[2].parse() { s.load_avg[2] = v; } }
        }
        if let Ok(content) = std::fs::read_to_string("/proc/uptime") {
            let p: Vec<&str> = content.trim().split_whitespace().collect();
            if p.len() >= 1 { if let Ok(secs) = p[0].parse::<f64>() { s.uptime_secs = secs as u64; } }
        }
        s.running = self.containers.iter().filter(|c| c.status == "running").count();
        s.stopped = self.containers.iter().filter(|c| c.status != "running").count();
        s.total_images = self.images.len();
        s.total_volumes = self.volumes.len();
        s
    }

    pub fn format_uptime(secs: u64) -> String {
        let d = secs / 86400; let h = (secs % 86400) / 3600; let m = (secs % 3600) / 60;
        if d > 0 { format!("{}d{}h{}m", d, h, m) }
        else if h > 0 { format!("{}h{}m", h, m) }
        else { format!("{}m", m) }
    }

    pub fn format_size(bytes: u64) -> String {
        if bytes < 1024 { format!("{}B", bytes) }
        else if bytes < 1024*1024 { format!("{:.1}KB", bytes as f64/1024.0) }
        else if bytes < 1024*1024*1024 { format!("{:.1}MB", bytes as f64/1024.0/1024.0) }
        else { format!("{:.1}GB", bytes as f64/1024.0/1024.0/1024.0) }
    }

    fn load_extensions(&self) -> Vec<MarketplaceExt> {
        let ext_dir = self.data_dir.join("extensions");
        let installed_ids: Vec<String> = if ext_dir.exists() {
            std::fs::read_dir(&ext_dir)
                .map(|e| e.filter_map(|x| x.ok()).map(|x| x.file_name().to_string_lossy().to_string()).collect())
                .unwrap_or_default()
        } else { Vec::new() };

        let registry_paths = [
            "/home/paong/qcker-extensions/marketplace/extensions.json",
            "marketplace/extensions.json",
        ];
        let mut registry_exts: Vec<serde_json::Value> = Vec::new();
        for rp in &registry_paths {
            if let Ok(content) = std::fs::read_to_string(rp) {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(extensions) = data.get("marketplace").and_then(|m| m.get("extensions")).and_then(|e| e.as_array()) {
                        registry_exts = extensions.clone();
                        break;
                    }
                }
            }
        }

        let mut result: Vec<MarketplaceExt> = Vec::new();
        for ext in &registry_exts {
            let id = ext["id"].as_str().unwrap_or("").to_string();
            let name = ext["display_name"].as_str().or(ext["name"].as_str()).unwrap_or(&id).to_string();
            let version = ext["version"].as_str().unwrap_or("0.0").to_string();
            let desc = ext["description"].as_str().unwrap_or("").to_string();
            let author = ext["author"].as_str().unwrap_or("Unknown").to_string();
            let category = ext["category"].as_str().unwrap_or("other").to_string();
            let built_in = ext["built_in"].as_bool().unwrap_or(false);
            let is_installed = installed_ids.iter().any(|i| id.contains(i.as_str()) || i.contains(id.as_str()));
            result.push(MarketplaceExt{ id: id.clone(), name, version, description: desc,
                author, category, built_in, installed: is_installed });
        }
        // Add any locally installed extensions not in registry
        for rid in &installed_ids {
            if !result.iter().any(|e| e.id == *rid) {
                result.push(MarketplaceExt{ id: rid.clone(), name: rid.clone(), version: "?".into(),
                    description: "Local extension".into(), author: "Local".into(),
                    category: "other".into(), built_in: false, installed: true });
            }
        }
        result
    }

    fn load_containers(&self) -> Vec<ContainerInfo> {
        let dir = self.data_dir.join("containers");
        if !dir.exists() { return Vec::new(); }
        let mut con = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let sp = entry.path().join("state.json");
                if let Ok(content) = std::fs::read_to_string(&sp) {
                    if let Ok(state) = serde_json::from_str::<serde_json::Value>(&content) {
                        let id = state["id"].as_str().unwrap_or("unknown").to_string();
                        let name = state["name"].as_str().unwrap_or(&id).to_string();
                        let status = match state["state"].as_str().unwrap_or("unknown") {
                            "Running" => "running", "Created" => "created",
                            "Stopped" => "stopped", s => s,
                        }.to_string();
                        let pid = state["pid"].as_i64().map(|p| p as i32);
                        let image = state.dig_str(&["config","image"]).unwrap_or_else(|| "unknown".to_string());
                        con.push(ContainerInfo{ id:id.clone(), name:if name==id{format!("{} (default)",name)}else{name},
                            status, image, pid, created:state["created_at"].as_str().unwrap_or("N/A").to_string() });
                    }
                }
            }
        }
        con.sort_by(|a,b| b.created.cmp(&a.created));
        con
    }

    fn load_images(&self) -> Vec<ImageInfo> {
        let dir = self.data_dir.join("images");
        if !dir.exists() { return Vec::new(); }
        let mut imgs = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let mp = entry.path().join("manifest.json");
                if let Ok(content) = std::fs::read_to_string(&mp) {
                    if let Ok(m) = serde_json::from_str::<serde_json::Value>(&content) {
                        let id = m["id"].as_str().unwrap_or("unknown").to_string();
                        let tags = m["tags"].as_array().map(|a| a.iter().map(|v| v.as_str().unwrap_or("")).collect::<Vec<_>>().join(", ")).unwrap_or_else(||"<none>".to_string());
                        let size = m["size"].as_u64().unwrap_or(0);
                        imgs.push(ImageInfo{id, tags, size:Self::format_size(size), created:m["created_at"].as_str().unwrap_or("N/A").to_string()});
                    }
                }
            }
        }
        imgs
    }

    fn load_networks(&self) -> Vec<NetworkInfo> {
        vec![
            NetworkInfo{id:"host".into(),name:"host".into(),driver:"host".into(),subnet:"-".into()},
            NetworkInfo{id:"none".into(),name:"none".into(),driver:"none".into(),subnet:"-".into()},
            NetworkInfo{id:"bridge".into(),name:"bridge".into(),driver:"bridge".into(),subnet:"172.17.0.0/16".into()},
        ]
    }

    fn load_volumes(&self) -> Vec<VolumeInfo> {
        let dir = self.data_dir.join("volumes");
        if !dir.exists() { return Vec::new(); }
        let mut vols = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let cp = entry.path().join("config.json");
                if let Ok(content) = std::fs::read_to_string(&cp) {
                    if let Ok(c) = serde_json::from_str::<serde_json::Value>(&content) {
                        vols.push(VolumeInfo{name:c["name"].as_str().unwrap_or("unknown").to_string(),
                            driver:c["driver"].as_str().unwrap_or("local").to_string(),
                            mountpoint:c["mountpoint"].as_str().unwrap_or("N/A").to_string()});
                    }
                }
            }
        }
        vols
    }

    pub fn tab_label(&self) -> &'static str {
        match self.active_tab {
            ActiveTab::Containers=>"Containers",ActiveTab::Images=>"Images",
            ActiveTab::Networks=>"Networks",ActiveTab::Volumes=>"Volumes",
            ActiveTab::Stats=>"Stats",ActiveTab::Extensions=>"Extensions",
            ActiveTab::Logs=>"Logs",
        }
    }

    pub fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            ActiveTab::Containers=>ActiveTab::Images,ActiveTab::Images=>ActiveTab::Networks,
            ActiveTab::Networks=>ActiveTab::Volumes,ActiveTab::Volumes=>ActiveTab::Stats,
            ActiveTab::Stats=>ActiveTab::Extensions,ActiveTab::Extensions=>ActiveTab::Logs,
            ActiveTab::Logs=>ActiveTab::Containers,
        };
        self.selected_index=0;self.scroll_offset=0;self.selected_action=0;
    }

    pub fn prev_tab(&mut self) {
        self.active_tab = match self.active_tab {
            ActiveTab::Containers=>ActiveTab::Logs,ActiveTab::Images=>ActiveTab::Containers,
            ActiveTab::Networks=>ActiveTab::Images,ActiveTab::Volumes=>ActiveTab::Networks,
            ActiveTab::Stats=>ActiveTab::Volumes,ActiveTab::Extensions=>ActiveTab::Stats,
            ActiveTab::Logs=>ActiveTab::Extensions,
        };
        self.selected_index=0;self.scroll_offset=0;self.selected_action=0;
    }

    pub fn click_tab(&mut self, index: usize) {
        let tabs = [ActiveTab::Containers,ActiveTab::Images,ActiveTab::Networks,
                     ActiveTab::Volumes,ActiveTab::Stats,ActiveTab::Extensions,ActiveTab::Logs];
        if index < tabs.len() {
            self.active_tab = tabs[index].clone();
            self.selected_index=0;self.scroll_offset=0;self.selected_action=0;
            self.mode=AppMode::Normal;
            self.status_message = format!("Switched to {} tab", self.tab_label());
        }
    }

    pub fn next_item(&mut self) {
        let max = self.max_items();
        if max > 0 { self.selected_index=(self.selected_index+1)%max; if self.selected_index>self.scroll_offset+12{self.scroll_offset=self.selected_index-12;} }
    }

    pub fn prev_item(&mut self) {
        let max = self.max_items();
        if max > 0 {
            self.selected_index = if self.selected_index==0{max-1}else{self.selected_index-1};
            if self.selected_index < self.scroll_offset { self.scroll_offset = self.selected_index; }
        }
    }

    pub fn max_items(&self) -> usize {
        match self.active_tab {
            ActiveTab::Containers=>self.containers.len(),ActiveTab::Images=>self.images.len(),
            ActiveTab::Networks=>self.networks.len(),ActiveTab::Volumes=>self.volumes.len(),
            ActiveTab::Stats=>0,ActiveTab::Extensions=>self.extensions.len(),
            ActiveTab::Logs=>self.logs.len(),
        }
    }

    pub fn get_selected_container(&self) -> Option<&ContainerInfo> {
        if let ActiveTab::Containers = self.active_tab { self.containers.get(self.selected_index) } else { None }
    }

    pub fn start_container(&mut self) {
        if let Some(id) = self.selected_container.clone() {
            let _ = self.run_command(&["start",&id]); self.refresh();
            self.status_message = format!("Started: {}", id);
        }
    }

    pub fn stop_container(&mut self) {
        if let Some(id) = self.selected_container.clone() {
            let _ = self.run_command(&["kill",&id]); self.refresh();
            self.status_message = format!("Stopped: {}", id);
        }
    }

    pub fn delete_container(&mut self) {
        let name = if let Some(c)=self.get_selected_container(){c.name.clone()}else{return};
        let id = if let Some(c)=self.get_selected_container(){c.id.clone()}else{return};
        self.confirm_message=format!("Delete '{}'?",name);
        self.confirm_action=Some(ConfirmAction::DeleteContainer(id));
        self.mode=AppMode::ConfirmDelete;
    }

    pub fn exec_in_container(&mut self) {
        let id = if let Some(c)=self.get_selected_container(){c.id.clone()}else{return};
        self.selected_container=Some(id);
        self.exec_cmd.clear();
        self.mode=AppMode::ExecCommand;
        self.status_message="Exec command:".to_string();
    }

    pub fn execute_exec(&mut self) {
        if let Some(id)=self.selected_container.clone(){
            if !self.exec_cmd.is_empty(){
                let out=self.run_command(&["exec",&id,&self.exec_cmd]);
                if let Ok(s)=out{self.logs.push(format!("$ {}",self.exec_cmd));self.logs.extend(s.lines().map(|l|l.to_string()));}
                self.exec_cmd.clear();
                self.mode=AppMode::Normal;self.active_tab=ActiveTab::Logs;self.refresh();
            }
        }
    }

    pub fn watch_logs(&mut self) {
        let id = if let Some(c)=self.get_selected_container(){c.id.clone()}else{return};
        self.selected_container=Some(id.clone());
        let _=self.run_command(&["logs",&id]);self.refresh();
        self.mode=AppMode::WatchingLogs;
        self.status_message=format!("Watching logs: {}",id);
    }

    pub fn install_extension(&mut self) {
        if let Some(ext) = self.extensions.get(self.selected_index) {
            if ext.built_in { self.status_message = format!("Built-in: {}", ext.name); return; }
            if ext.installed { self.status_message = format!("Already installed: {}", ext.name); return; }
            let eid = ext.id.clone();
            let manifest = format!(r#"{{"extension":{{"id":"{}","name":"{}","version":"{}","api_version":"1.0.0","author":"{}","description":"{}","capabilities":["ContainerLifecycle"]}}}}"#,
                eid, ext.name, ext.version, ext.author, ext.description);
            let dir = self.data_dir.join("extensions").join(&eid);
            if std::fs::create_dir_all(&dir).is_ok() {
                if std::fs::write(dir.join("manifest.json"), manifest).is_ok() {
                    self.status_message = format!("Installed: {}", ext.name);
                    self.extensions = self.load_extensions();
                    return;
                }
            }
            self.status_message = "Install failed".to_string();
        }
    }

    pub fn enable_extension(&mut self) {
        if let Some(ext) = self.extensions.get(self.selected_index) {
            if !ext.installed { self.status_message = format!("Not installed: {}", ext.name); return; }
            let id = ext.id.clone();
            let name = ext.name.clone();
            let _ = self.run_command(&["extension", "enable", &id]);
            self.refresh();
            self.status_message = format!("Enabled: {}", name);
        }
    }

    pub fn disable_extension(&mut self) {
        if let Some(ext) = self.extensions.get(self.selected_index) {
            if !ext.installed { self.status_message = format!("Not installed: {}", ext.name); return; }
            let id = ext.id.clone();
            let name = ext.name.clone();
            let _ = self.run_command(&["extension", "disable", &id]);
            self.refresh();
            self.status_message = format!("Disabled: {}", name);
        }
    }

    pub fn uninstall_extension(&mut self) {
        if let Some(ext) = self.extensions.get(self.selected_index) {
            if ext.built_in { self.status_message = "Cannot uninstall built-in".to_string(); return; }
            if !ext.installed { self.status_message = format!("Not installed: {}", ext.name); return; }
            self.confirm_message = format!("Uninstall {}?", ext.name);
            self.confirm_action = Some(ConfirmAction::UninstallExtension(ext.id.clone()));
            self.mode = AppMode::ConfirmDelete;
        }
    }

    pub fn open_new_container(&mut self) {
        self.new_name.clear();self.new_image.clear();self.new_cmd.clear();
        self.mode=AppMode::NewContainer;
        self.status_message="Container name:".to_string();
    }

    pub fn execute_new_container(&mut self) {
        if self.new_name.is_empty(){self.status_message="Name required!".to_string();return;}
        let mut args=vec!["run","--name",&self.new_name];
        if !self.new_image.is_empty(){args.push("--image");args.push(&self.new_image);}
        if !self.new_cmd.is_empty(){for p in self.new_cmd.split_whitespace(){args.push(p);}}
        else{args.push("/bin/sh");}
        match self.run_command(&args){
            Ok(_)=>{self.refresh();self.status_message=format!("Created: {}",self.new_name);}
            Err(e)=>{self.status_message=format!("Failed: {}",e);}
        }
        self.mode=AppMode::Normal;
    }

    pub fn exit_new_container(&mut self){self.mode=AppMode::Normal;self.status_message="Cancelled".to_string();}

    pub fn pull_image(&mut self) {
        if !self.pull_input.is_empty(){
            let _=self.run_command(&["pull",&self.pull_input]);
            self.refresh();self.pull_input.clear();self.mode=AppMode::Normal;
            self.status_message=format!("Pulled: {}",self.pull_input);
        }
    }

    pub fn toggle_help(&mut self){self.show_help=!self.show_help;}

    pub fn run_command(&self, args:&[&str]) -> Result<String,String> {
        use std::process::Command;
        let exe=std::env::current_exe().unwrap_or_default();
        match Command::new(&exe).args(args).output(){
            Ok(out)=>Ok(String::from_utf8_lossy(&out.stdout).to_string()),
            Err(e)=>Err(e.to_string()),
        }
    }

    pub fn confirm_delete(&mut self) {
        let id = if let Some(c)=self.get_selected_container(){c.id.clone()}else{return};
        let name = if let Some(c)=self.get_selected_container(){c.name.clone()}else{return};
        self.confirm_message=format!("Delete '{}'?",name);
        self.confirm_action=Some(ConfirmAction::DeleteContainer(id));
        self.mode=AppMode::ConfirmDelete;
    }

    pub fn cancel_confirm(&mut self){self.confirm_action=None;self.mode=AppMode::Normal;self.status_message="Cancelled".to_string();}

    pub fn execute_confirm(&mut self) {
        if let Some(action)=self.confirm_action.take(){
            match action{
                ConfirmAction::DeleteContainer(id)=>{
                    let _=self.run_command(&["delete","--force",&id]);
                    self.refresh();self.status_message=format!("Deleted: {}",id);
                }
                ConfirmAction::UninstallExtension(id)=>{
                    let dir=self.data_dir.join("extensions").join(&id);
                    if dir.exists(){let _=std::fs::remove_dir_all(&dir);self.extensions=self.load_extensions();}
                    self.status_message="Uninstalled".to_string();
                }
            }
        }
        self.mode=AppMode::Normal;self.confirm_action=None;
    }
}

trait JsonValueExt {
    fn dig_str(&self, path:&[&str]) -> Option<String>;
}
impl JsonValueExt for serde_json::Value {
    fn dig_str(&self, path:&[&str]) -> Option<String> {
        let mut cur=self;
        for k in path{cur=cur.get(*k)?;}
        cur.as_str().map(|s|s.to_string())
    }
}

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub duration: Duration,
    pub memory_usage_mb: f64,
    pub cpu_percent: f64,
    pub status: String,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct BenchmarkSuite {
    pub name: String,
    pub results: Vec<BenchmarkResult>,
    pub started_at: Instant,
    pub completed_at: Option<Instant>,
}

impl BenchmarkSuite {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            results: Vec::new(),
            started_at: Instant::now(),
            completed_at: None,
        }
    }

    pub fn add_result(&mut self, result: BenchmarkResult) {
        self.results.push(result);
    }

    pub fn complete(&mut self) {
        self.completed_at = Some(Instant::now());
    }

    pub fn total_duration(&self) -> Duration {
        self.started_at.elapsed()
    }

    pub fn avg_duration(&self) -> Duration {
        if self.results.is_empty() {
            return Duration::ZERO;
        }
        let total: u64 = self.results.iter()
            .map(|r| r.duration.as_millis() as u64)
            .sum();
        Duration::from_millis(total / self.results.len() as u64)
    }

    pub fn min_duration(&self) -> Option<Duration> {
        self.results.iter().map(|r| r.duration).min()
    }

    pub fn max_duration(&self) -> Option<Duration> {
        self.results.iter().map(|r| r.duration).max()
    }

    pub fn success_rate(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        let success = self.results.iter()
            .filter(|r| r.status == "success")
            .count();
        success as f64 / self.results.len() as f64 * 100.0
    }

    pub fn avg_memory_mb(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        self.results.iter().map(|r| r.memory_usage_mb).sum::<f64>() / self.results.len() as f64
    }

    pub fn avg_cpu(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        self.results.iter().map(|r| r.cpu_percent).sum::<f64>() / self.results.len() as f64
    }
}

pub fn get_system_metrics() -> HashMap<String, f64> {
    let mut metrics = HashMap::new();

    if let Ok(content) = std::fs::read_to_string("/proc/stat") {
        for line in content.lines() {
            if line.starts_with("cpu ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    let user: u64 = parts[1].parse().unwrap_or(0);
                    let nice: u64 = parts[2].parse().unwrap_or(0);
                    let system: u64 = parts[3].parse().unwrap_or(0);
                    let idle: u64 = parts[4].parse().unwrap_or(0);
                    let total = user + nice + system + idle;
                    if total > 0 {
                        metrics.insert("cpu_user".to_string(), user as f64 / total as f64 * 100.0);
                        metrics.insert("cpu_system".to_string(), system as f64 / total as f64 * 100.0);
                        metrics.insert("cpu_idle".to_string(), idle as f64 / total as f64 * 100.0);
                    }
                }
                break;
            }
        }
    }

    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb: u64 = parts[1].parse().unwrap_or(0);
                    metrics.insert("mem_total_mb".to_string(), kb as f64 / 1024.0);
                }
                break;
            } else if line.starts_with("MemAvailable:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb: u64 = parts[1].parse().unwrap_or(0);
                    metrics.insert("mem_available_mb".to_string(), kb as f64 / 1024.0);
                }
                break;
            }
        }
    }

    metrics
}

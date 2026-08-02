use clap::Args;
use std::path::Path;


#[derive(Args)]
pub struct LogsArgs {
    pub container_id: String,

    #[arg(short, long)]
    pub follow: bool,

    #[arg(short, long)]
    pub tail: Option<usize>,

    #[arg(long)]
    pub since: Option<String>,

    #[arg(short = 'T', long)]
    pub timestamps: bool,
}

use tokio::time::{sleep, Duration};

pub async fn execute(args: LogsArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let log_path = data_dir
        .join("containers")
        .join(&args.container_id)
        .join("container.log");

    if !log_path.exists() {
        if format == "json" {
            let output = serde_json::json!({
                "container": args.container_id,
                "logs": [],
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("No logs available for container {}", args.container_id);
        }
        return Ok(());
    }

    if args.follow {
        let content = std::fs::read_to_string(&log_path)?;
        let lines: Vec<&str> = content.lines().collect();
        let start = if let Some(tail) = args.tail {
            if lines.len() > tail { lines.len() - tail } else { 0 }
        } else {
            0
        };
        for line in &lines[start..] {
            if format == "json" {
                let entry = serde_json::json!({ "log": line });
                println!("{}", serde_json::to_string(&entry)?);
            } else if args.timestamps {
                println!("[{}] {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), line);
            } else {
                println!("{}", line);
            }
        }
        loop {
            sleep(Duration::from_millis(100)).await;
            let new_content = std::fs::read_to_string(&log_path)?;
            let new_lines: Vec<&str> = new_content.lines().collect();
            if new_lines.len() > lines.len() {
                for line in &new_lines[lines.len()..new_lines.len()] {
                    if format == "json" {
                        let entry = serde_json::json!({ "log": line });
                        println!("{}", serde_json::to_string(&entry)?);
                    } else if args.timestamps {
                        println!("[{}] {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), line);
                    } else {
                        println!("{}", line);
                    }
                }
            }
        }
    } else {
        let content = std::fs::read_to_string(&log_path)?;
        let lines: Vec<&str> = content.lines().collect();
        let start = if let Some(tail) = args.tail {
            if lines.len() > tail { lines.len() - tail } else { 0 }
        } else {
            0
        };
        let display_lines = &lines[start..];

        if format == "json" {
            let log_entries: Vec<serde_json::Value> = display_lines
                .iter()
                .map(|line| {
                    serde_json::json!({
                        "log": line,
                    })
                })
                .collect();
            let output = serde_json::json!({
                "container": args.container_id,
                "logs": log_entries,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            for line in display_lines {
                if args.timestamps {
                    println!("[{}] {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), line);
                } else {
                    println!("{}", line);
                }
            }
        }
    }

    Ok(())
}

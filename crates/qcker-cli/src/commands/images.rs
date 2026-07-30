use clap::Args;
use std::path::Path;

use qcker_engine::image::store::ImageStore;

#[derive(Args)]
pub struct ImagesArgs {
    #[arg(short, long)]
    _all: bool,
}

pub fn execute(_args: ImagesArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let store = ImageStore::new(data_dir.to_path_buf());
    store.init()?;

    let images = store.list_images()?;

    if format == "json" {
        let output: Vec<serde_json::Value> = images
            .iter()
            .map(|img| {
                serde_json::json!({
                    "id": img.id,
                    "tags": img.tags,
                    "size": img.size,
                    "created_at": img.created_at,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        if images.is_empty() {
            println!("No images found.");
            return Ok(());
        }

        println!("{:<15} {:<40} {:<15} {:<20}", "IMAGE ID", "TAGS", "SIZE", "CREATED");
        for img in &images {
            let tags = if img.tags.is_empty() {
                "<none>".to_string()
            } else {
                img.tags.join(", ")
            };
            println!(
                "{:<15} {:<40} {:<15} {:<20}",
                img.id,
                tags,
                format_size(img.size),
                img.created_at.split('T').next().unwrap_or("")
            );
        }
    }

    Ok(())
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

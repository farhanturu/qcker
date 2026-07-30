use clap::Args;
use std::path::Path;

use crate::output;
use qcker_engine::registry::client::RegistryClient;

#[derive(Args)]
pub struct PullArgs {
    image: String,

    #[arg(long, default_value = "registry-1.docker.io")]
    registry: String,
}

pub async fn execute(args: PullArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let client = RegistryClient::new(&args.registry);

    println!("Pulling {} from {}...", args.image, args.registry);

    let image = client.pull_image(&args.image, data_dir.to_path_buf()).await?;

    output::print_success(&format!("Pulled image {}:{}", args.image, image.id));

    if format == "json" {
        let output = serde_json::json!({
            "id": image.id,
            "tags": image.tags,
            "size": image.size,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Image ID: {}", image.id);
        println!("Tags:     {}", image.tags.join(", "));
        println!("Size:     {} bytes", image.size);
    }

    Ok(())
}

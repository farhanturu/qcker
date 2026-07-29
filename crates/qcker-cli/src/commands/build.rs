use clap::Args;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::output;
use qcker_engine::build::dockerfile;
use qcker_engine::build::executor::{BuildContext as ExecutorContext, BuildExecutor};

/// Build an image from a Dockerfile
#[derive(Args)]
pub struct BuildArgs {
    /// Build context directory
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Image tag(s)
    #[arg(short, long)]
    tag: Vec<String>,

    /// Dockerfile path
    #[arg(short, long, default_value = "Dockerfile")]
    file: PathBuf,

    /// Build arguments
    #[arg(long)]
    build_arg: Vec<String>,

    /// Do not use cache
    #[arg(long)]
    no_cache: bool,
}

pub fn execute(args: BuildArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    // Resolve paths
    let context_dir = args.path.canonicalize()?;
    let dockerfile_path = if args.file.is_absolute() {
        args.file.clone()
    } else {
        context_dir.join(&args.file)
    };

    // Parse Dockerfile
    println!("Building image from {}...", dockerfile_path.display());
    let dockerfile = dockerfile::parse_file(&dockerfile_path)?;

    // Parse build args
    let mut build_args = HashMap::new();
    for arg in &args.build_arg {
        let parts: Vec<&str> = arg.splitn(2, '=').collect();
        if parts.len() == 2 {
            build_args.insert(parts[0].to_string(), parts[1].to_string());
        }
    }

    // Create build context
    let build_context = ExecutorContext {
        context_dir: context_dir.clone(),
        dockerfile,
        tags: args.tag.clone(),
        build_args,
        no_cache: args.no_cache,
    };

    // Execute build
    let executor = BuildExecutor::new(data_dir.to_path_buf());
    let result = executor.build(build_context)?;

    output::print_success(&format!("Built image {}", result.image_id));

    if format == "json" {
        let output = serde_json::json!({
            "image_id": result.image_id,
            "tags": result.tags,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Image ID: {}", result.image_id);
        if !result.tags.is_empty() {
            println!("Tags:     {}", result.tags.join(", "));
        }
    }

    Ok(())
}

use clap::Args;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::output;
use qcker_engine::build::dockerfile;
use qcker_engine::build::executor::{BuildContext as ExecutorContext, BuildExecutor};

#[derive(Args)]
pub struct BuildArgs {
    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short, long)]
    tag: Vec<String>,

    #[arg(short, long, default_value = "Dockerfile")]
    file: PathBuf,

    #[arg(long)]
    build_arg: Vec<String>,

    #[arg(long)]
    no_cache: bool,
}

pub fn execute(args: BuildArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let context_dir = args.path.canonicalize()?;
    let dockerfile_path = if args.file.is_absolute() {
        args.file.clone()
    } else {
        context_dir.join(&args.file)
    };

    println!("Building image from {}...", dockerfile_path.display());
    let dockerfile = dockerfile::parse_file(&dockerfile_path)?;

    let mut build_args = HashMap::new();
    for arg in &args.build_arg {
        let parts: Vec<&str> = arg.splitn(2, '=').collect();
        if parts.len() == 2 {
            build_args.insert(parts[0].to_string(), parts[1].to_string());
        }
    }

    let build_context = ExecutorContext {
        context_dir: context_dir.clone(),
        dockerfile,
        tags: args.tag.clone(),
        build_args,
        no_cache: args.no_cache,
    };

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

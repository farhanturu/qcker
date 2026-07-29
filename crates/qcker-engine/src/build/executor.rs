use std::collections::HashMap;
use std::path::PathBuf;

use qcker_common::error::{QckerError, Result};

use super::dockerfile::{Dockerfile, Instruction};
use crate::image::store::{ContainerConfig, Image, ImageConfig, ImageStore, RootFs};

/// Build context
pub struct BuildContext {
    pub context_dir: PathBuf,
    pub dockerfile: Dockerfile,
    pub tags: Vec<String>,
    pub build_args: HashMap<String, String>,
    pub no_cache: bool,
}

/// Build result
pub struct BuildResult {
    pub image_id: String,
    pub tags: Vec<String>,
}

/// Build executor
pub struct BuildExecutor {
    pub data_dir: PathBuf,
}

impl BuildExecutor {
    /// Create a new build executor
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    /// Execute a build
    pub fn build(&self, context: BuildContext) -> Result<BuildResult> {
        let mut current_image: Option<Image> = None;
        let mut build_args = context.build_args.clone();

        for (i, stage) in context.dockerfile.stages.iter().enumerate() {
            tracing::info!("Building stage {}: FROM {}", i, stage.base_image);

            // Get base image
            let _base_image = self.get_base_image(&stage.base_image)?;

            // Execute instructions
            let mut env_vars: HashMap<String, String> = HashMap::new();
            let mut working_dir = String::from("/");
            let mut user = String::from("root");
            let mut cmd: Option<Vec<String>> = None;
            let mut entrypoint: Option<Vec<String>> = None;
            let mut exposed_ports: Vec<String> = Vec::new();
            let mut volumes: Vec<String> = Vec::new();
            let mut labels: HashMap<String, String> = HashMap::new();

            for instruction in &stage.instructions {
                match instruction {
                    Instruction::Run(command) => {
                        tracing::info!("RUN {}", command);
                        // In a real implementation, this would execute the command
                        // in a container and snapshot the filesystem diff
                    }
                    Instruction::Cmd(args) => {
                        cmd = Some(args.clone());
                    }
                    Instruction::Entrypoint(args) => {
                        entrypoint = Some(args.clone());
                    }
                    Instruction::Env(envs) => {
                        for (key, value) in envs {
                            let expanded = expand_build_args(value, &build_args);
                            env_vars.insert(key.clone(), expanded);
                        }
                    }
                    Instruction::Workdir(dir) => {
                        let expanded = expand_build_args(dir, &build_args);
                        working_dir = expanded;
                    }
                    Instruction::User(u) => {
                        user = u.clone();
                    }
                    Instruction::Expose(ports) => {
                        exposed_ports.extend(ports.clone());
                    }
                    Instruction::Volume(vols) => {
                        volumes.extend(vols.clone());
                    }
                    Instruction::Label(labels_vec) => {
                        for (key, value) in labels_vec {
                            labels.insert(key.clone(), value.clone());
                        }
                    }
                    Instruction::Arg(name, default) => {
                        // Use build arg if provided, otherwise use default
                        if !build_args.contains_key(name) {
                            if let Some(default_val) = default {
                                build_args.insert(name.clone(), default_val.clone());
                            }
                        }
                    }
                    Instruction::Copy {
                        sources,
                        destination,
                        from: _,
                        ..
                    } => {
                        tracing::info!("COPY {:?} -> {}", sources, destination);
                        // In a real implementation, this would copy files
                        // from the build context into the container
                    }
                    Instruction::Add {
                        sources,
                        destination,
                        ..
                    } => {
                        tracing::info!("ADD {:?} -> {}", sources, destination);
                        // Similar to COPY but can handle URLs and tar archives
                    }
                    _ => {
                        // Handle other instructions
                    }
                }
            }

            // Create image from this stage
            let env_list: Vec<String> = env_vars
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();

            let image_config = ImageConfig {
                architecture: "amd64".to_string(),
                os: "linux".to_string(),
                config: Some(ContainerConfig {
                    cmd,
                    entrypoint,
                    env: Some(env_list),
                    working_dir: Some(working_dir),
                    user: Some(user),
                }),
                rootfs: RootFs {
                    r#type: "layers".to_string(),
                    diff_ids: vec![],
                },
            };

            let image_id = format!("{:x}", md5::compute(format!("{}-{}", i, stage.base_image)));
            let image = Image {
                id: image_id[..12].to_string(),
                tags: if i == context.dockerfile.stages.len() - 1 {
                    context.tags.clone()
                } else {
                    vec![]
                },
                created_at: chrono::Utc::now().to_rfc3339(),
                size: 0,
                layers: vec![],
                config: image_config,
            };

            current_image = Some(image);
        }

        let final_image = current_image
            .ok_or_else(|| QckerError::Internal("No image built".to_string()))?;

        // Store the image
        let store = ImageStore::new(self.data_dir.clone());
        store.init()?;
        store.store_image(&final_image)?;

        Ok(BuildResult {
            image_id: final_image.id,
            tags: context.tags,
        })
    }

    /// Get base image (from local store or pull)
    fn get_base_image(&self, reference: &str) -> Result<Image> {
        let store = ImageStore::new(self.data_dir.clone());
        store.init()?;

        // Try to get from local store
        match store.get_image(reference) {
            Ok(image) => Ok(image),
            Err(_) => {
                // For now, create a dummy image
                // In a real implementation, this would pull from registry
                tracing::warn!("Base image not found locally: {}", reference);
                Ok(Image {
                    id: "scratch".to_string(),
                    tags: vec![reference.to_string()],
                    created_at: chrono::Utc::now().to_rfc3339(),
                    size: 0,
                    layers: vec![],
                    config: ImageConfig {
                        architecture: "amd64".to_string(),
                        os: "linux".to_string(),
                        config: None,
                        rootfs: RootFs {
                            r#type: "layers".to_string(),
                            diff_ids: vec![],
                        },
                    },
                })
            }
        }
    }
}

/// Expand build args in a string
fn expand_build_args(value: &str, build_args: &HashMap<String, String>) -> String {
    let mut result = value.to_string();
    for (key, val) in build_args {
        result = result.replace(&format!("${{{}}}", key), val);
        result = result.replace(&format!("${}", key), val);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_build_args() {
        let mut args = HashMap::new();
        args.insert("VERSION".to_string(), "1.0".to_string());

        let result = expand_build_args("app:${VERSION}", &args);
        assert_eq!(result, "app:1.0");

        let result = expand_build_args("app:$VERSION", &args);
        assert_eq!(result, "app:1.0");
    }
}

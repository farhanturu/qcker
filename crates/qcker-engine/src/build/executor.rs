use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use qcker_common::error::{QckerError, Result};

use super::dockerfile::{Dockerfile, Instruction};
use crate::image::store::{ImageStore, ImageConfig, ContainerConfig, RootFs, Image};

pub struct BuildContext {
    pub context_dir: PathBuf,
    pub dockerfile: Dockerfile,
    pub tags: Vec<String>,
    pub build_args: HashMap<String, String>,
    pub no_cache: bool,
}

pub struct BuildResult {
    pub image_id: String,
    pub tags: Vec<String>,
}

pub struct BuildExecutor {
    pub data_dir: PathBuf,
}

impl BuildExecutor {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    pub fn build(&self, context: BuildContext) -> Result<BuildResult> {
        let mut env_vars: HashMap<String, String> = HashMap::new();
        let mut working_dir = String::from("/");
        let mut user = String::from("root");
        let mut cmd: Option<Vec<String>> = None;
        let mut entrypoint: Option<Vec<String>> = None;

        let build_dir = self.data_dir.join("build").join(qcker_common::id::generate_container_id());
        fs::create_dir_all(&build_dir)?;

        for stage in &context.dockerfile.stages {
            for instruction in &stage.instructions {
                match instruction {
                    Instruction::Run(command) => {
                        self.execute_run(&build_dir, command, &env_vars, &working_dir)?;
                    }
                    Instruction::Env(envs) => {
                        for (key, value) in envs {
                            let expanded = self.expand_args(value, &context.build_args);
                            env_vars.insert(key.clone(), expanded);
                        }
                    }
                    Instruction::Workdir(dir) => {
                        working_dir = self.expand_args(dir, &context.build_args);
                    }
                    Instruction::Copy { sources, destination, .. } => {
                        self.execute_copy(&context.context_dir, &build_dir, sources, destination)?;
                    }
                    Instruction::Add { sources, destination, .. } => {
                        self.execute_add(&context.context_dir, &build_dir, sources, destination)?;
                    }
                    Instruction::Cmd(args) => {
                        cmd = Some(args.clone());
                    }
                    Instruction::Entrypoint(args) => {
                        entrypoint = Some(args.clone());
                    }
                    Instruction::User(u) => {
                        user = u.clone();
                    }
                    _ => {}
                }
            }
        }

        let image_id = qcker_common::id::generate_image_id(
            serde_json::to_string(&context.tags).unwrap_or_default().as_bytes()
        );

        let image = Image {
            id: image_id[..12].to_string(),
            tags: context.tags.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            size: 0,
            layers: vec![],
            config: ImageConfig {
                architecture: "amd64".to_string(),
                os: "linux".to_string(),
                config: Some(ContainerConfig {
                    cmd,
                    entrypoint,
                    env: Some(env_vars.iter().map(|(k, v)| format!("{}={}", k, v)).collect()),
                    working_dir: Some(working_dir),
                    user: Some(user),
                }),
                rootfs: RootFs {
                    r#type: "layers".to_string(),
                    diff_ids: vec![],
                },
            },
        };

        let store = ImageStore::new(self.data_dir.clone());
        store.init()?;
        store.store_image(&image)?;

        let _ = fs::remove_dir_all(&build_dir);

        Ok(BuildResult {
            image_id: image.id,
            tags: context.tags,
        })
    }

    fn execute_run(&self, build_dir: &Path, command: &str, env: &HashMap<String, String>, workdir: &str) -> Result<()> {
        let output = Command::new("/bin/sh")
            .arg("-c")
            .arg(command)
            .current_dir(build_dir.join(workdir.trim_start_matches('/')))
            .envs(env)
            .output()
            .map_err(|e| QckerError::internal(format!("Failed to execute RUN: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(QckerError::internal(format!("RUN command failed: {}", stderr)));
        }

        Ok(())
    }

    fn execute_copy(&self, context_dir: &Path, build_dir: &Path, sources: &[String], destination: &str) -> Result<()> {
        let dest_path = build_dir.join(destination.trim_start_matches('/'));
        fs::create_dir_all(&dest_path)?;

        for source in sources {
            let src_path = context_dir.join(source);
            if src_path.exists() {
                if src_path.is_dir() {
                    self.copy_dir_all(&src_path, &dest_path)?;
                } else {
                    let dest_file = dest_path.join(src_path.file_name().unwrap());
                    fs::copy(&src_path, &dest_file)?;
                }
            }
        }

        Ok(())
    }

    fn execute_add(&self, context_dir: &Path, build_dir: &Path, sources: &[String], destination: &str) -> Result<()> {
        self.execute_copy(context_dir, build_dir, sources, destination)
    }

    fn copy_dir_all(&self, src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let dest_path = dst.join(entry.file_name());
            if ty.is_dir() {
                self.copy_dir_all(&entry.path(), &dest_path)?;
            } else {
                fs::copy(entry.path(), &dest_path)?;
            }
        }
        Ok(())
    }

    fn expand_args(&self, value: &str, build_args: &HashMap<String, String>) -> String {
        let mut result = value.to_string();
        for (key, val) in build_args {
            result = result.replace(&format!("${{{}}}", key), val);
            result = result.replace(&format!("${}", key), val);
        }
        result
    }

    fn get_base_image(&self, reference: &str) -> Result<Image> {
        let store = ImageStore::new(self.data_dir.clone());
        store.init()?;

        match store.get_image(reference) {
            Ok(image) => Ok(image),
            Err(_) => {
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


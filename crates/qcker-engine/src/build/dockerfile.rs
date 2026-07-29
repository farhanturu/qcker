use serde::{Deserialize, Serialize};
use std::path::Path;

use qcker_common::error::{QckerError, Result};

/// Dockerfile instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    /// FROM <image> [AS <name>]
    From {
        image: String,
        alias: Option<String>,
    },
    /// RUN <command>
    Run(String),
    /// CMD ["executable","param1","param2"]
    Cmd(Vec<String>),
    /// LABEL <key>=<value> ...
    Label(Vec<(String, String)>),
    /// EXPOSE <port> [<port>/<protocol>...]
    Expose(Vec<String>),
    /// ENV <key>=<value> ...
    Env(Vec<(String, String)>),
    /// ADD [--chown=<user>:<group>] <src>... <dest>
    Add {
        sources: Vec<String>,
        destination: String,
        chown: Option<String>,
    },
    /// COPY [--chown=<user>:<group>] <src>... <dest>
    Copy {
        sources: Vec<String>,
        destination: String,
        chown: Option<String>,
        from: Option<String>,
    },
    /// ENTRYPOINT ["executable", "param1"]
    Entrypoint(Vec<String>),
    /// VOLUME ["/data"]
    Volume(Vec<String>),
    /// USER <user>[:<group>]
    User(String),
    /// WORKDIR /path/to/workdir
    Workdir(String),
    /// ARG <name>[=<default>]
    Arg(String, Option<String>),
    /// ONBUILD <INSTRUCTION>
    Onbuild(Box<Instruction>),
    /// STOPSIGNAL <signal>
    Stopsignal(String),
    /// HEALTHCHECK [OPTIONS] CMD command
    Healthcheck {
        interval: Option<String>,
        timeout: Option<String>,
        retries: Option<u32>,
        command: String,
    },
    /// SHELL ["executable", "parameters"]
    Shell(Vec<String>),
}

/// Build stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage {
    pub name: Option<String>,
    pub base_image: String,
    pub instructions: Vec<Instruction>,
}

/// Parsed Dockerfile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dockerfile {
    pub stages: Vec<Stage>,
}

/// Parse a Dockerfile
pub fn parse(content: &str) -> Result<Dockerfile> {
    let mut stages = Vec::new();
    let mut current_stage: Option<Stage> = None;
    let mut current_instruction = String::new();
    let mut in_continuation = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Handle line continuation
        if in_continuation {
            current_instruction.push(' ');
            current_instruction.push_str(trimmed.trim_end_matches('\\').trim());
            if !trimmed.ends_with('\\') {
                in_continuation = false;
                process_instruction(&mut current_stage, &mut stages, &current_instruction)?;
                current_instruction.clear();
            }
            continue;
        }

        // Check for line continuation
        if trimmed.ends_with('\\') {
            in_continuation = true;
            current_instruction = trimmed.trim_end_matches('\\').trim().to_string();
            continue;
        }

        // Process single line instruction
        process_instruction(&mut current_stage, &mut stages, trimmed)?;
    }

    // Process any remaining instruction
    if !current_instruction.is_empty() {
        process_instruction(&mut current_stage, &mut stages, &current_instruction)?;
    }

    // Add the last stage
    if let Some(stage) = current_stage {
        stages.push(stage);
    }

    if stages.is_empty() {
        return Err(QckerError::InvalidArgument("Empty Dockerfile".to_string()));
    }

    Ok(Dockerfile { stages })
}

/// Process a single instruction
fn process_instruction(
    current_stage: &mut Option<Stage>,
    stages: &mut Vec<Stage>,
    line: &str,
) -> Result<()> {
    let (keyword, args) = parse_instruction(line)?;

    match keyword.as_str() {
        "FROM" => {
            // Save previous stage
            if let Some(stage) = current_stage.take() {
                stages.push(stage);
            }

            // Parse FROM instruction
            let parts: Vec<&str> = args.split_whitespace().collect();
            let image = parts[0].to_string();
            let alias = if parts.len() > 2 && parts[1].eq_ignore_ascii_case("AS") {
                Some(parts[2].to_string())
            } else {
                None
            };

            *current_stage = Some(Stage {
                name: alias,
                base_image: image,
                instructions: Vec::new(),
            });
        }
        "RUN" => {
            if let Some(ref mut stage) = current_stage {
                stage.instructions.push(Instruction::Run(args));
            }
        }
        "CMD" => {
            if let Some(ref mut stage) = current_stage {
                let cmd = parse_json_or_shell(&args)?;
                stage.instructions.push(Instruction::Cmd(cmd));
            }
        }
        "LABEL" => {
            if let Some(ref mut stage) = current_stage {
                let labels = parse_key_value_pairs(&args);
                stage.instructions.push(Instruction::Label(labels));
            }
        }
        "EXPOSE" => {
            if let Some(ref mut stage) = current_stage {
                let ports: Vec<String> = args.split_whitespace().map(String::from).collect();
                stage.instructions.push(Instruction::Expose(ports));
            }
        }
        "ENV" => {
            if let Some(ref mut stage) = current_stage {
                let envs = parse_key_value_pairs(&args);
                stage.instructions.push(Instruction::Env(envs));
            }
        }
        "ADD" => {
            if let Some(ref mut stage) = current_stage {
                let (sources, dest, chown) = parse_copy_add_args(&args)?;
                stage.instructions.push(Instruction::Add {
                    sources,
                    destination: dest,
                    chown,
                });
            }
        }
        "COPY" => {
            if let Some(ref mut stage) = current_stage {
                let (sources, dest, chown) = parse_copy_add_args(&args)?;
                stage.instructions.push(Instruction::Copy {
                    sources,
                    destination: dest,
                    chown,
                    from: None,
                });
            }
        }
        "ENTRYPOINT" => {
            if let Some(ref mut stage) = current_stage {
                let entrypoint = parse_json_or_shell(&args)?;
                stage.instructions.push(Instruction::Entrypoint(entrypoint));
            }
        }
        "VOLUME" => {
            if let Some(ref mut stage) = current_stage {
                let volumes = parse_json_array(&args);
                stage.instructions.push(Instruction::Volume(volumes));
            }
        }
        "USER" => {
            if let Some(ref mut stage) = current_stage {
                stage.instructions.push(Instruction::User(args));
            }
        }
        "WORKDIR" => {
            if let Some(ref mut stage) = current_stage {
                stage.instructions.push(Instruction::Workdir(args));
            }
        }
        "ARG" => {
            if let Some(ref mut stage) = current_stage {
                let (name, default) = parse_arg(&args);
                stage.instructions.push(Instruction::Arg(name, default));
            }
        }
        "STOPSIGNAL" => {
            if let Some(ref mut stage) = current_stage {
                stage.instructions.push(Instruction::Stopsignal(args));
            }
        }
        "SHELL" => {
            if let Some(ref mut stage) = current_stage {
                let shell = parse_json_array(&args);
                stage.instructions.push(Instruction::Shell(shell));
            }
        }
        _ => {
            tracing::warn!("Unknown instruction: {}", keyword);
        }
    }

    Ok(())
}

/// Parse instruction keyword and args
fn parse_instruction(line: &str) -> Result<(String, String)> {
    let line = line.trim();
    let space_pos = line.find(char::is_whitespace).unwrap_or(line.len());
    let keyword = line[..space_pos].to_uppercase();
    let args = line[space_pos..].trim().to_string();
    Ok((keyword, args))
}

/// Parse JSON array or shell command
fn parse_json_or_shell(args: &str) -> Result<Vec<String>> {
    let trimmed = args.trim();
    if trimmed.starts_with('[') {
        parse_json_array_inner(trimmed)
    } else {
        // Shell form - use /bin/sh -c
        Ok(vec![
            "/bin/sh".to_string(),
            "-c".to_string(),
            trimmed.to_string(),
        ])
    }
}

/// Parse JSON array
fn parse_json_array(args: &str) -> Vec<String> {
    parse_json_array_inner(args).unwrap_or_else(|_| {
        args.split_whitespace().map(String::from).collect()
    })
}

/// Parse JSON array inner
fn parse_json_array_inner(args: &str) -> Result<Vec<String>> {
    let trimmed = args.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Err(QckerError::InvalidArgument(format!(
            "Invalid JSON array: {}",
            trimmed
        )));
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escape_next = false;

    for ch in inner.chars() {
        if escape_next {
            current.push(ch);
            escape_next = false;
            continue;
        }

        if ch == '\\' {
            escape_next = true;
            continue;
        }

        if ch == '"' {
            in_string = !in_string;
            continue;
        }

        if ch == ',' && !in_string {
            result.push(current.trim().to_string());
            current.clear();
            continue;
        }

        current.push(ch);
    }

    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }

    Ok(result)
}

/// Parse key=value pairs
fn parse_key_value_pairs(args: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let mut parts = args.split_whitespace();

    while let Some(part) = parts.next() {
        if let Some(eq_pos) = part.find('=') {
            let key = part[..eq_pos].to_string();
            let value = part[eq_pos + 1..].to_string();
            pairs.push((key, value));
        } else if let Some(next) = parts.next() {
            pairs.push((part.to_string(), next.to_string()));
        }
    }

    pairs
}

/// Parse COPY/ADD arguments
fn parse_copy_add_args(args: &str) -> Result<(Vec<String>, String, Option<String>)> {
    let parts: Vec<&str> = args.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(QckerError::InvalidArgument(
            "COPY/ADD requires at least source and destination".to_string(),
        ));
    }

    let destination = parts.last().unwrap().to_string();
    let sources = parts[..parts.len() - 1]
        .iter()
        .filter(|s| !s.starts_with("--"))
        .map(|s| s.to_string())
        .collect();

    // Parse --chown flag
    let chown = parts
        .iter()
        .find(|s| s.starts_with("--chown="))
        .map(|s| s.strip_prefix("--chown=").unwrap().to_string());

    Ok((sources, destination, chown))
}

/// Parse ARG instruction
fn parse_arg(args: &str) -> (String, Option<String>) {
    if let Some(eq_pos) = args.find('=') {
        let name = args[..eq_pos].trim().to_string();
        let default = args[eq_pos + 1..].trim().to_string();
        (name, Some(default))
    } else {
        (args.trim().to_string(), None)
    }
}

/// Parse Dockerfile from file
pub fn parse_file(path: &Path) -> Result<Dockerfile> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| QckerError::Internal(format!("Failed to read Dockerfile: {}", e)))?;
    parse(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_dockerfile() {
        let content = r#"
FROM alpine:latest
RUN echo "hello"
CMD ["echo", "world"]
"#;
        let dockerfile = parse(content).unwrap();
        assert_eq!(dockerfile.stages.len(), 1);
        assert_eq!(dockerfile.stages[0].base_image, "alpine:latest");
        assert_eq!(dockerfile.stages[0].instructions.len(), 2);
    }

    #[test]
    fn test_parse_multi_stage() {
        let content = r#"
FROM golang:1.21 AS builder
RUN go build -o app .

FROM alpine:latest
COPY --from=builder /app /app
CMD ["/app"]
"#;
        let dockerfile = parse(content).unwrap();
        assert_eq!(dockerfile.stages.len(), 2);
        assert_eq!(dockerfile.stages[0].name, Some("builder".to_string()));
    }

    #[test]
    fn test_parse_json_array() {
        let result = parse_json_array_inner(r#"["echo", "hello", "world"]"#).unwrap();
        assert_eq!(result, vec!["echo", "hello", "world"]);
    }

    #[test]
    fn test_parse_key_value_pairs() {
        let pairs = parse_key_value_pairs("KEY1=value1 KEY2=value2");
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].0, "KEY1");
        assert_eq!(pairs[0].1, "value1");
    }
}

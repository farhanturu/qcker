#[derive(Debug, thiserror::Error)]
pub enum QckerError {
    #[error("Container not found: {0}")]
    ContainerNotFound(String),

    #[error("Image not found: {0}. Try 'qcker pull {0}' first")]
    ImageNotFound(String),

    #[error("Namespace error: {0}")]
    Namespace(String),

    #[error("Cgroup error: {0}")]
    Cgroup(String),

    #[error("Mount error: {0}")]
    Mount(String),

    #[error("Seccomp error: {0}")]
    Seccomp(String),

    #[error("Capability error: {0}")]
    Capability(String),

    #[error("Process error: {0}")]
    Process(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("OCI spec error: {0}")]
    OciSpec(String),

    #[error("Tar error: {0}")]
    Tar(String),

    #[error("Hash error: {0}")]
    Hash(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Not supported: {0}")]
    NotSupported(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, QckerError>;

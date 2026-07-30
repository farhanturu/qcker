use serde::{Deserialize, Serialize};
use std::fmt;

use crate::codes::{ErrorCode, ErrorSeverity};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug)]
pub struct QckerError {
    pub code: ErrorCode,
    pub message: String,
    pub location: ErrorLocation,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
    pub suggestion: Option<String>,
    pub timestamp: String,
}

impl fmt::Display for QckerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code.code, self.message)
    }
}

impl std::error::Error for QckerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as &dyn std::error::Error)
    }
}

impl QckerError {
    #[track_caller]
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        let loc = std::panic::Location::caller();
        Self {
            code,
            message: message.into(),
            location: ErrorLocation {
                file: loc.file().to_string(),
                line: loc.line(),
                column: loc.column(),
            },
            source: None,
            suggestion: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn with_source(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn exit_code(&self) -> i32 {
        self.code.exit_code
    }

    pub fn error_code(&self) -> &'static str {
        self.code.code
    }

    pub fn retryable(&self) -> bool {
        self.code.retryable
    }

    pub fn severity(&self) -> ErrorSeverity {
        self.code.severity
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "error": {
                "code": self.code.code,
                "message": self.message,
                "severity": format!("{:?}", self.code.severity),
                "retryable": self.code.retryable,
                "suggestion": self.suggestion,
                "location": {
                    "file": self.location.file,
                    "line": self.location.line,
                    "column": self.location.column,
                },
                "timestamp": self.timestamp,
            }
        })
    }
}

pub type Result<T> = std::result::Result<T, QckerError>;

impl From<std::io::Error> for QckerError {
    fn from(err: std::io::Error) -> Self {
        QckerError::new(ErrorCode::IO_ERROR, err.to_string()).with_source(err)
    }
}

impl From<serde_json::Error> for QckerError {
    fn from(err: serde_json::Error) -> Self {
        QckerError::new(ErrorCode::JSON_ERROR, err.to_string()).with_source(err)
    }
}

impl From<String> for QckerError {
    fn from(msg: String) -> Self {
        QckerError::new(ErrorCode::UNKNOWN, msg)
    }
}

impl From<&str> for QckerError {
    fn from(msg: &str) -> Self {
        QckerError::new(ErrorCode::UNKNOWN, msg)
    }
}

impl QckerError {
    pub fn ContainerNotFound(id: impl Into<String>) -> Self {
        QckerError::new(ErrorCode::CONTAINER_NOT_FOUND, format!("Container not found: {}", id.into()))
    }
    pub fn ImageNotFound(id: impl Into<String>) -> Self {
        QckerError::new(ErrorCode::IMAGE_NOT_FOUND, format!("Image not found: {}", id.into()))
    }
    pub fn Namespace(msg: impl Into<String>) -> Self {
        QckerError::new(ErrorCode::RUNTIME_NAMESPACE_CREATE_FAILED, msg)
    }
    pub fn Cgroup(msg: impl Into<String>) -> Self {
        QckerError::new(ErrorCode::RUNTIME_CGROUP_CREATE_FAILED, msg)
    }
    pub fn Mount(msg: impl Into<String>) -> Self {
        QckerError::new(ErrorCode::RUNTIME_MOUNT_FAILED, msg)
    }
    pub fn Seccomp(msg: impl Into<String>) -> Self {
        QckerError::new(ErrorCode::RUNTIME_SECCOMP_APPLY_FAILED, msg)
    }
    pub fn Capability(msg: impl Into<String>) -> Self {
        QckerError::new(ErrorCode::RUNTIME_CAPABILITY_DROP_FAILED, msg)
    }
    pub fn Process(msg: impl Into<String>) -> Self {
        QckerError::new(ErrorCode::CONTAINER_START_FAILED, msg)
    }
    pub fn OciSpec(msg: impl Into<String>) -> Self {
        QckerError::new(ErrorCode::BUILD_DOCKERFILE_PARSE_FAILED, msg)
    }
    pub fn Tar(msg: impl Into<String>) -> Self {
        QckerError::new(ErrorCode::IO_ERROR, msg)
    }
    pub fn Hash(msg: impl Into<String>) -> Self {
        QckerError::new(ErrorCode::IO_ERROR, msg)
    }
    pub fn InvalidArgument(msg: impl Into<String>) -> Self {
        QckerError::new(ErrorCode::INVALID_ARGUMENT, msg)
    }
    pub fn PermissionDenied(msg: impl Into<String>) -> Self {
        QckerError::new(ErrorCode::PERMISSION_DENIED, msg)
    }
    pub fn NotSupported(msg: impl Into<String>) -> Self {
        QckerError::new(ErrorCode::RUNTIME_BACKEND_NOT_AVAILABLE, msg)
    }
    pub fn Network(msg: impl Into<String>) -> Self {
        QckerError::new(ErrorCode::NETWORK_CREATE_FAILED, msg)
    }
    pub fn Internal(msg: impl Into<String>) -> Self {
        QckerError::new(ErrorCode::UNKNOWN, msg)
    }
}

#[macro_export]
macro_rules! qcker_err {
    ($code:expr, $msg:expr) => {
        $crate::QckerError::new($code, $msg)
    };
    ($code:expr, $msg:expr, $($arg:tt)*) => {
        $crate::QckerError::new($code, format!($msg, $($arg)*))
    };
}

#[macro_export]
macro_rules! qcker_err_with_source {
    ($code:expr, $msg:expr, $source:expr) => {
        $crate::QckerError::new($code, $msg).with_source($source)
    };
    ($code:expr, $msg:expr, $($arg:tt)*, $source:expr) => {
        $crate::QckerError::new($code, format!($msg, $($arg)*)).with_source($source)
    };
}

#[macro_export]
macro_rules! qcker_err_with_hint {
    ($code:expr, $msg:expr, $hint:expr) => {
        $crate::QckerError::new($code, $msg).with_suggestion($hint)
    };
    ($code:expr, $msg:expr, $($arg:tt)*, $hint:expr) => {
        $crate::QckerError::new($code, format!($msg, $($arg)*)).with_suggestion($hint)
    };
}

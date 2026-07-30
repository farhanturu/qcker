use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ErrorCode {
    pub code: &'static str,
    pub description: &'static str,
    pub severity: ErrorSeverity,
    pub retryable: bool,
    pub exit_code: i32,
}

impl ErrorCode {
    pub const CONTAINER_NOT_FOUND: ErrorCode = ErrorCode {
        code: "Q-C001",
        description: "Container not found",
        severity: ErrorSeverity::Error,
        retryable: false,
        exit_code: 1,
    };
    pub const CONTAINER_ALREADY_EXISTS: ErrorCode = ErrorCode {
        code: "Q-C002",
        description: "Container already exists",
        severity: ErrorSeverity::Error,
        retryable: false,
        exit_code: 1,
    };
    pub const CONTAINER_NOT_RUNNING: ErrorCode = ErrorCode {
        code: "Q-C003",
        description: "Container not running",
        severity: ErrorSeverity::Error,
        retryable: false,
        exit_code: 1,
    };
    pub const CONTAINER_ALREADY_RUNNING: ErrorCode = ErrorCode {
        code: "Q-C004",
        description: "Container already running",
        severity: ErrorSeverity::Error,
        retryable: false,
        exit_code: 1,
    };
    pub const CONTAINER_CREATE_FAILED: ErrorCode = ErrorCode {
        code: "Q-C005",
        description: "Container creation failed",
        severity: ErrorSeverity::Error,
        retryable: true,
        exit_code: 1,
    };
    pub const CONTAINER_START_FAILED: ErrorCode = ErrorCode {
        code: "Q-C006",
        description: "Container start failed",
        severity: ErrorSeverity::Error,
        retryable: true,
        exit_code: 1,
    };
    pub const CONTAINER_STOP_FAILED: ErrorCode = ErrorCode {
        code: "Q-C007",
        description: "Container stop failed",
        severity: ErrorSeverity::Error,
        retryable: true,
        exit_code: 1,
    };
    pub const CONTAINER_DELETE_FAILED: ErrorCode = ErrorCode {
        code: "Q-C008",
        description: "Container delete failed",
        severity: ErrorSeverity::Error,
        retryable: true,
        exit_code: 1,
    };
    pub const CONTAINER_EXEC_FAILED: ErrorCode = ErrorCode {
        code: "Q-C009",
        description: "Container exec failed",
        severity: ErrorSeverity::Error,
        retryable: true,
        exit_code: 1,
    };
    pub const CONTAINER_ROOTFS_NOT_FOUND: ErrorCode = ErrorCode {
        code: "Q-C012",
        description: "Container rootfs not found",
        severity: ErrorSeverity::Error,
        retryable: false,
        exit_code: 1,
    };
    pub const CONTAINER_FORK_FAILED: ErrorCode = ErrorCode {
        code: "Q-C015",
        description: "Container process fork failed",
        severity: ErrorSeverity::Critical,
        retryable: true,
        exit_code: 1,
    };
    pub const CONTAINER_CHROOT_FAILED: ErrorCode = ErrorCode {
        code: "Q-C020",
        description: "Container chroot failed",
        severity: ErrorSeverity::Error,
        retryable: false,
        exit_code: 1,
    };
    pub const IMAGE_NOT_FOUND: ErrorCode = ErrorCode {
        code: "Q-I001",
        description: "Image not found",
        severity: ErrorSeverity::Error,
        retryable: false,
        exit_code: 1,
    };
    pub const IMAGE_PULL_FAILED: ErrorCode = ErrorCode {
        code: "Q-I003",
        description: "Image pull failed",
        severity: ErrorSeverity::Error,
        retryable: true,
        exit_code: 1,
    };
    pub const NETWORK_NOT_FOUND: ErrorCode = ErrorCode {
        code: "Q-N001",
        description: "Network not found",
        severity: ErrorSeverity::Error,
        retryable: false,
        exit_code: 1,
    };
    pub const NETWORK_CREATE_FAILED: ErrorCode = ErrorCode {
        code: "Q-N003",
        description: "Network create failed",
        severity: ErrorSeverity::Error,
        retryable: true,
        exit_code: 1,
    };
    pub const NETWORK_IP_EXHAUSTED: ErrorCode = ErrorCode {
        code: "Q-N006",
        description: "Network IP pool exhausted",
        severity: ErrorSeverity::Error,
        retryable: false,
        exit_code: 1,
    };
    pub const RUNTIME_NAMESPACE_CREATE_FAILED: ErrorCode = ErrorCode {
        code: "Q-R001",
        description: "Runtime namespace create failed",
        severity: ErrorSeverity::Critical,
        retryable: true,
        exit_code: 1,
    };
    pub const RUNTIME_CGROUP_CREATE_FAILED: ErrorCode = ErrorCode {
        code: "Q-R003",
        description: "Runtime cgroup create failed",
        severity: ErrorSeverity::Error,
        retryable: true,
        exit_code: 1,
    };
    pub const RUNTIME_SECCOMP_APPLY_FAILED: ErrorCode = ErrorCode {
        code: "Q-R005",
        description: "Runtime seccomp apply failed",
        severity: ErrorSeverity::Warning,
        retryable: false,
        exit_code: 0,
    };
    pub const RUNTIME_CAPABILITY_DROP_FAILED: ErrorCode = ErrorCode {
        code: "Q-R006",
        description: "Runtime capability drop failed",
        severity: ErrorSeverity::Warning,
        retryable: false,
        exit_code: 0,
    };
    pub const RUNTIME_USER_NAMESPACE_FAILED: ErrorCode = ErrorCode {
        code: "Q-R008",
        description: "Runtime user namespace failed",
        severity: ErrorSeverity::Error,
        retryable: true,
        exit_code: 1,
    };
    pub const RUNTIME_MOUNT_FAILED: ErrorCode = ErrorCode {
        code: "Q-R012",
        description: "Runtime mount failed",
        severity: ErrorSeverity::Error,
        retryable: false,
        exit_code: 1,
    };
    pub const RUNTIME_HOSTNAME_SET_FAILED: ErrorCode = ErrorCode {
        code: "Q-R013",
        description: "Runtime hostname set failed",
        severity: ErrorSeverity::Warning,
        retryable: false,
        exit_code: 0,
    };
    pub const RUNTIME_BACKEND_NOT_AVAILABLE: ErrorCode = ErrorCode {
        code: "Q-R015",
        description: "Runtime backend not available",
        severity: ErrorSeverity::Critical,
        retryable: false,
        exit_code: 1,
    };
    pub const RUNTIME_VM_BOOT_FAILED: ErrorCode = ErrorCode {
        code: "Q-R017",
        description: "Runtime VM boot failed",
        severity: ErrorSeverity::Critical,
        retryable: true,
        exit_code: 1,
    };
    pub const RUNTIME_KERNEL_CHECKSUM_MISMATCH: ErrorCode = ErrorCode {
        code: "Q-R022",
        description: "Runtime kernel checksum mismatch",
        severity: ErrorSeverity::Critical,
        retryable: false,
        exit_code: 1,
    };
    pub const VOLUME_NOT_FOUND: ErrorCode = ErrorCode {
        code: "Q-S001",
        description: "Volume not found",
        severity: ErrorSeverity::Error,
        retryable: false,
        exit_code: 1,
    };
    pub const VOLUME_CREATE_FAILED: ErrorCode = ErrorCode {
        code: "Q-S003",
        description: "Volume create failed",
        severity: ErrorSeverity::Error,
        retryable: true,
        exit_code: 1,
    };
    pub const BUILD_DOCKERFILE_PARSE_FAILED: ErrorCode = ErrorCode {
        code: "Q-B001",
        description: "Build Dockerfile parse failed",
        severity: ErrorSeverity::Error,
        retryable: false,
        exit_code: 1,
    };
    pub const BUILD_INSTRUCTION_FAILED: ErrorCode = ErrorCode {
        code: "Q-B002",
        description: "Build instruction failed",
        severity: ErrorSeverity::Error,
        retryable: true,
        exit_code: 1,
    };
    pub const EXTENSION_NOT_FOUND: ErrorCode = ErrorCode {
        code: "Q-E001",
        description: "Extension not found",
        severity: ErrorSeverity::Error,
        retryable: false,
        exit_code: 1,
    };
    pub const EXTENSION_LOAD_FAILED: ErrorCode = ErrorCode {
        code: "Q-E002",
        description: "Extension load failed",
        severity: ErrorSeverity::Error,
        retryable: true,
        exit_code: 1,
    };
    pub const PERMISSION_DENIED: ErrorCode = ErrorCode {
        code: "Q-P001",
        description: "Permission denied",
        severity: ErrorSeverity::Error,
        retryable: false,
        exit_code: 1,
    };
    pub const ROOT_REQUIRED: ErrorCode = ErrorCode {
        code: "Q-P002",
        description: "Root required",
        severity: ErrorSeverity::Error,
        retryable: false,
        exit_code: 1,
    };
    pub const IO_ERROR: ErrorCode = ErrorCode {
        code: "Q-X001",
        description: "IO error",
        severity: ErrorSeverity::Error,
        retryable: true,
        exit_code: 1,
    };
    pub const JSON_ERROR: ErrorCode = ErrorCode {
        code: "Q-X002",
        description: "JSON error",
        severity: ErrorSeverity::Error,
        retryable: false,
        exit_code: 1,
    };
    pub const HTTP_ERROR: ErrorCode = ErrorCode {
        code: "Q-X005",
        description: "HTTP error",
        severity: ErrorSeverity::Error,
        retryable: true,
        exit_code: 1,
    };
    pub const INVALID_ARGUMENT: ErrorCode = ErrorCode {
        code: "Q-X008",
        description: "Invalid argument",
        severity: ErrorSeverity::Error,
        retryable: false,
        exit_code: 1,
    };
    pub const UNKNOWN: ErrorCode = ErrorCode {
        code: "Q-U001",
        description: "Unknown error",
        severity: ErrorSeverity::Error,
        retryable: false,
        exit_code: 1,
    };
}

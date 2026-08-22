use std::path::PathBuf;

/// Errors produced by the core engine. The TUI (or any other frontend) is
/// expected to turn these into user-facing messages rather than panicking.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("file not found: {0}")]
    NotFound(PathBuf),

    #[error("permission denied: {0}")]
    PermissionDenied(PathBuf),

    #[error("file is not valid UTF-8: {0}")]
    InvalidUtf8(PathBuf),

    #[error("path escapes the workspace root: {0}")]
    OutsideWorkspace(PathBuf),

    #[error("file too large to open ({size_mb} MB, limit is {limit_mb} MB): {path}")]
    FileTooLarge {
        path: PathBuf,
        size_mb: u64,
        limit_mb: u64,
    },

    #[error("a file already exists at: {0}")]
    AlreadyExists(PathBuf),

    #[error("invalid configuration: {0}")]
    Config(String),

    #[error("no document is currently open")]
    NoDocument,

    #[error("invalid task index: {0}")]
    InvalidTaskIndex(usize),
}

pub type Result<T> = std::result::Result<T, AppError>;

impl AppError {
    pub fn from_io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        let path = path.into();
        match source.kind() {
            std::io::ErrorKind::NotFound => AppError::NotFound(path),
            std::io::ErrorKind::PermissionDenied => AppError::PermissionDenied(path),
            _ => AppError::Io { path, source },
        }
    }
}

use thiserror::Error;

pub type Result<T> = std::result::Result<T, CodexError>;

#[derive(Error, Debug)]
pub enum CodexError {
    #[error("Codex library error: {message}")]
    LibraryError { message: String },

    #[error("Node operation failed: {operation} - {message}")]
    NodeError { operation: String, message: String },

    #[error("Upload failed: {message}")]
    UploadError { message: String },

    #[error("Download failed: {message}")]
    DownloadError { message: String },

    #[error("Storage operation failed: {operation} - {message}")]
    StorageError { operation: String, message: String },

    #[error("P2P operation failed: {message}")]
    P2PError { message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Invalid parameter: {parameter} - {message}")]
    InvalidParameter { parameter: String, message: String },

    #[error("Operation timed out: {operation}")]
    Timeout { operation: String },

    #[error("Operation cancelled: {operation}")]
    Cancelled { operation: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Null pointer encountered in {context}")]
    NullPointer { context: String },

    #[error("Task join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

impl CodexError {
    pub fn library_error(message: impl Into<String>) -> Self {
        CodexError::LibraryError {
            message: message.into(),
        }
    }

    pub fn node_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        CodexError::NodeError {
            operation: operation.into(),
            message: message.into(),
        }
    }

    pub fn upload_error(message: impl Into<String>) -> Self {
        CodexError::UploadError {
            message: message.into(),
        }
    }

    pub fn download_error(message: impl Into<String>) -> Self {
        CodexError::DownloadError {
            message: message.into(),
        }
    }

    pub fn storage_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        CodexError::StorageError {
            operation: operation.into(),
            message: message.into(),
        }
    }

    pub fn p2p_error(message: impl Into<String>) -> Self {
        CodexError::P2PError {
            message: message.into(),
        }
    }

    pub fn config_error(message: impl Into<String>) -> Self {
        CodexError::ConfigError {
            message: message.into(),
        }
    }

    pub fn invalid_parameter(parameter: impl Into<String>, message: impl Into<String>) -> Self {
        CodexError::InvalidParameter {
            parameter: parameter.into(),
            message: message.into(),
        }
    }

    pub fn timeout(operation: impl Into<String>) -> Self {
        CodexError::Timeout {
            operation: operation.into(),
        }
    }

    pub fn cancelled(operation: impl Into<String>) -> Self {
        CodexError::Cancelled {
            operation: operation.into(),
        }
    }

    pub fn null_pointer(context: impl Into<String>) -> Self {
        CodexError::NullPointer {
            context: context.into(),
        }
    }
}

pub fn from_c_error(code: i32, message: &str) -> CodexError {
    match code {
        0 => CodexError::library_error(format!("Unexpected success with message: {}", message)),
        1 => CodexError::library_error(message),
        _ => CodexError::library_error(format!("Unknown error code {}: {}", code, message)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = CodexError::library_error("Test error");
        assert!(matches!(err, CodexError::LibraryError { .. }));

        let err = CodexError::node_error("start", "Failed to start");
        assert!(matches!(err, CodexError::NodeError { .. }));

        let err = CodexError::upload_error("Upload failed");
        assert!(matches!(err, CodexError::UploadError { .. }));
    }

    #[test]
    fn test_error_display() {
        let err = CodexError::library_error("Test error");
        assert_eq!(err.to_string(), "Codex library error: Test error");

        let err = CodexError::node_error("start", "Failed to start");
        assert_eq!(
            err.to_string(),
            "Node operation failed: start - Failed to start"
        );
    }
}

impl Clone for CodexError {
    fn clone(&self) -> Self {
        match self {
            CodexError::LibraryError { message } => CodexError::LibraryError {
                message: message.clone(),
            },
            CodexError::NodeError { operation, message } => CodexError::NodeError {
                operation: operation.clone(),
                message: message.clone(),
            },
            CodexError::UploadError { message } => CodexError::UploadError {
                message: message.clone(),
            },
            CodexError::DownloadError { message } => CodexError::DownloadError {
                message: message.clone(),
            },
            CodexError::StorageError { operation, message } => CodexError::StorageError {
                operation: operation.clone(),
                message: message.clone(),
            },
            CodexError::P2PError { message } => CodexError::P2PError {
                message: message.clone(),
            },
            CodexError::ConfigError { message } => CodexError::ConfigError {
                message: message.clone(),
            },
            CodexError::InvalidParameter { parameter, message } => CodexError::InvalidParameter {
                parameter: parameter.clone(),
                message: message.clone(),
            },
            CodexError::Timeout { operation } => CodexError::Timeout {
                operation: operation.clone(),
            },
            CodexError::Cancelled { operation } => CodexError::Cancelled {
                operation: operation.clone(),
            },
            CodexError::Io(_) => CodexError::library_error("I/O error"),
            CodexError::Json(_) => CodexError::library_error("JSON error"),
            CodexError::Utf8(_) => CodexError::library_error("UTF-8 error"),
            CodexError::NullPointer { context } => CodexError::NullPointer {
                context: context.clone(),
            },
            CodexError::JoinError(_) => CodexError::library_error("Task join error"),
        }
    }
}

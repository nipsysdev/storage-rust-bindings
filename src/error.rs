use thiserror::Error;

pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Storage library error: {message}")]
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

    #[error("Missing callback: {message}")]
    MissingCallback { message: String },

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

impl StorageError {
    pub fn library_error(message: impl Into<String>) -> Self {
        StorageError::LibraryError {
            message: message.into(),
        }
    }

    pub fn node_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        StorageError::NodeError {
            operation: operation.into(),
            message: message.into(),
        }
    }

    pub fn upload_error(message: impl Into<String>) -> Self {
        StorageError::UploadError {
            message: message.into(),
        }
    }

    pub fn download_error(message: impl Into<String>) -> Self {
        StorageError::DownloadError {
            message: message.into(),
        }
    }

    pub fn storage_operation_error(
        operation: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        StorageError::StorageError {
            operation: operation.into(),
            message: message.into(),
        }
    }

    pub fn p2p_error(message: impl Into<String>) -> Self {
        StorageError::P2PError {
            message: message.into(),
        }
    }

    pub fn config_error(message: impl Into<String>) -> Self {
        StorageError::ConfigError {
            message: message.into(),
        }
    }

    pub fn invalid_parameter(parameter: impl Into<String>, message: impl Into<String>) -> Self {
        StorageError::InvalidParameter {
            parameter: parameter.into(),
            message: message.into(),
        }
    }

    pub fn timeout(operation: impl Into<String>) -> Self {
        StorageError::Timeout {
            operation: operation.into(),
        }
    }

    pub fn cancelled(operation: impl Into<String>) -> Self {
        StorageError::Cancelled {
            operation: operation.into(),
        }
    }

    pub fn missing_callback(message: impl Into<String>) -> Self {
        StorageError::MissingCallback {
            message: message.into(),
        }
    }

    pub fn null_pointer(context: impl Into<String>) -> Self {
        StorageError::NullPointer {
            context: context.into(),
        }
    }
}

pub fn from_c_error(code: i32, message: &str) -> StorageError {
    match code {
        0 => StorageError::library_error(format!("Unexpected success with message: {}", message)),
        1 => StorageError::library_error(message),
        2 => StorageError::missing_callback(message),
        _ => StorageError::library_error(format!("Unknown error code {}: {}", code, message)),
    }
}

impl Clone for StorageError {
    fn clone(&self) -> Self {
        match self {
            StorageError::LibraryError { message } => StorageError::LibraryError {
                message: message.clone(),
            },
            StorageError::NodeError { operation, message } => StorageError::NodeError {
                operation: operation.clone(),
                message: message.clone(),
            },
            StorageError::UploadError { message } => StorageError::UploadError {
                message: message.clone(),
            },
            StorageError::DownloadError { message } => StorageError::DownloadError {
                message: message.clone(),
            },
            StorageError::StorageError { operation, message } => StorageError::StorageError {
                operation: operation.clone(),
                message: message.clone(),
            },
            StorageError::P2PError { message } => StorageError::P2PError {
                message: message.clone(),
            },
            StorageError::ConfigError { message } => StorageError::ConfigError {
                message: message.clone(),
            },
            StorageError::InvalidParameter { parameter, message } => {
                StorageError::InvalidParameter {
                    parameter: parameter.clone(),
                    message: message.clone(),
                }
            }
            StorageError::Timeout { operation } => StorageError::Timeout {
                operation: operation.clone(),
            },
            StorageError::Cancelled { operation } => StorageError::Cancelled {
                operation: operation.clone(),
            },
            StorageError::MissingCallback { message } => StorageError::MissingCallback {
                message: message.clone(),
            },
            StorageError::Io(_) => StorageError::library_error("I/O error"),
            StorageError::Json(_) => StorageError::library_error("JSON error"),
            StorageError::Utf8(_) => StorageError::library_error("UTF-8 error"),
            StorageError::NullPointer { context } => StorageError::NullPointer {
                context: context.clone(),
            },
            StorageError::JoinError(_) => StorageError::library_error("Task join error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = StorageError::library_error("Test error");
        assert!(matches!(err, StorageError::LibraryError { .. }));

        let err = StorageError::node_error("start", "Failed to start");
        assert!(matches!(err, StorageError::NodeError { .. }));

        let err = StorageError::upload_error("Upload failed");
        assert!(matches!(err, StorageError::UploadError { .. }));
    }

    #[test]
    fn test_error_display() {
        let err = StorageError::library_error("Test error");
        assert_eq!(err.to_string(), "Storage library error: Test error");

        let err = StorageError::node_error("start", "Failed to start");
        assert_eq!(
            err.to_string(),
            "Node operation failed: start - Failed to start"
        );
    }
}

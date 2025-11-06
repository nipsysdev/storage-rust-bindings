//! Types for upload operations

use crate::error::{CodexError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Upload strategy determines how data is uploaded to the network
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UploadStrategy {
    /// Upload data in chunks
    Chunked,
    /// Upload data as a single stream
    Stream,
    /// Automatically choose the best strategy
    Auto,
}

impl Default for UploadStrategy {
    fn default() -> Self {
        UploadStrategy::Auto
    }
}

/// Progress information for upload operations
#[derive(Debug, Clone)]
pub struct UploadProgress {
    /// Number of bytes uploaded
    pub bytes_uploaded: usize,
    /// Total number of bytes to upload (if known)
    pub total_bytes: Option<usize>,
    /// Progress percentage (0.0 to 1.0)
    pub percentage: f64,
    /// Current chunk being uploaded (if applicable)
    pub current_chunk: Option<usize>,
    /// Total number of chunks (if applicable)
    pub total_chunks: Option<usize>,
}

impl UploadProgress {
    /// Create new upload progress
    pub fn new(bytes_uploaded: usize, total_bytes: Option<usize>) -> Self {
        let percentage = if let Some(total) = total_bytes {
            if total > 0 {
                bytes_uploaded as f64 / total as f64
            } else {
                0.0
            }
        } else {
            0.0
        };

        Self {
            bytes_uploaded,
            total_bytes,
            percentage: percentage.min(1.0),
            current_chunk: None,
            total_chunks: None,
        }
    }

    /// Create new chunked upload progress
    pub fn new_chunked(
        bytes_uploaded: usize,
        total_bytes: Option<usize>,
        current_chunk: usize,
        total_chunks: usize,
    ) -> Self {
        let mut progress = Self::new(bytes_uploaded, total_bytes);
        progress.current_chunk = Some(current_chunk);
        progress.total_chunks = Some(total_chunks);
        progress
    }
}

/// Options for upload operations
pub struct UploadOptions {
    /// Path to the file to upload (if uploading from file)
    pub filepath: Option<PathBuf>,
    /// Chunk size for chunked uploads (in bytes)
    pub chunk_size: Option<usize>,
    /// Upload strategy to use
    pub strategy: UploadStrategy,
    /// Progress callback function
    pub on_progress: Option<Box<dyn Fn(UploadProgress) + Send + Sync>>,
    /// Whether to verify the upload after completion
    pub verify: bool,
    /// Custom metadata to attach to the upload
    pub metadata: Option<serde_json::Value>,
    /// Timeout for the upload operation (in seconds)
    pub timeout: Option<u64>,
}

impl std::fmt::Debug for UploadOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UploadOptions")
            .field("filepath", &self.filepath)
            .field("chunk_size", &self.chunk_size)
            .field("strategy", &self.strategy)
            .field("on_progress", &self.on_progress.is_some())
            .field("verify", &self.verify)
            .field("metadata", &self.metadata)
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl Default for UploadOptions {
    fn default() -> Self {
        Self {
            filepath: None,
            chunk_size: Some(1024 * 1024), // 1 MB default
            strategy: UploadStrategy::Auto,
            on_progress: None,
            verify: true,
            metadata: None,
            timeout: Some(300), // 5 minutes default
        }
    }
}

impl UploadOptions {
    /// Create new upload options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the file path to upload
    pub fn filepath<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.filepath = Some(path.into());
        self
    }

    /// Set the chunk size for chunked uploads
    pub fn chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = Some(size);
        self
    }

    /// Set the upload strategy
    pub fn strategy(mut self, strategy: UploadStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set the progress callback
    pub fn on_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(UploadProgress) + Send + Sync + 'static,
    {
        self.on_progress = Some(Box::new(callback));
        self
    }

    /// Set whether to verify the upload
    pub fn verify(mut self, verify: bool) -> Self {
        self.verify = verify;
        self
    }

    /// Set custom metadata
    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set the timeout
    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Validate the upload options
    pub fn validate(&self) -> Result<()> {
        if let Some(chunk_size) = self.chunk_size {
            if chunk_size == 0 {
                return Err(CodexError::invalid_parameter(
                    "chunk_size",
                    "Chunk size must be greater than 0",
                ));
            }
        }

        if let Some(timeout) = self.timeout {
            if timeout == 0 {
                return Err(CodexError::invalid_parameter(
                    "timeout",
                    "Timeout must be greater than 0",
                ));
            }
        }

        Ok(())
    }
}

/// Result of an upload operation
#[derive(Debug, Clone)]
pub struct UploadResult {
    /// Content ID (CID) of the uploaded content
    pub cid: String,
    /// Size of the uploaded content in bytes
    pub size: usize,
    /// Number of chunks uploaded (if applicable)
    pub chunks: Option<usize>,
    /// Time taken for the upload (in milliseconds)
    pub duration_ms: u64,
    /// Whether the upload was verified (if verification was requested)
    pub verified: bool,
}

impl UploadResult {
    /// Create a new upload result
    pub fn new(cid: String, size: usize) -> Self {
        Self {
            cid,
            size,
            chunks: None,
            duration_ms: 0,
            verified: false,
        }
    }

    /// Set the number of chunks
    pub fn chunks(mut self, chunks: usize) -> Self {
        self.chunks = Some(chunks);
        self
    }

    /// Set the duration
    pub fn duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    /// Set whether the upload was verified
    pub fn verified(mut self, verified: bool) -> Self {
        self.verified = verified;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_progress() {
        let progress = UploadProgress::new(500, Some(1000));
        assert_eq!(progress.bytes_uploaded, 500);
        assert_eq!(progress.total_bytes, Some(1000));
        assert_eq!(progress.percentage, 0.5);

        let chunked = UploadProgress::new_chunked(500, Some(1000), 2, 4);
        assert_eq!(chunked.current_chunk, Some(2));
        assert_eq!(chunked.total_chunks, Some(4));
    }

    #[test]
    fn test_upload_options() {
        let options = UploadOptions::new()
            .filepath("/test/file.txt")
            .chunk_size(2048)
            .strategy(UploadStrategy::Chunked)
            .verify(false)
            .timeout(600);

        assert_eq!(options.filepath, Some(PathBuf::from("/test/file.txt")));
        assert_eq!(options.chunk_size, Some(2048));
        assert_eq!(options.strategy, UploadStrategy::Chunked);
        assert_eq!(options.verify, false);
        assert_eq!(options.timeout, Some(600));
    }

    #[test]
    fn test_upload_options_validation() {
        let mut options = UploadOptions::new();
        assert!(options.validate().is_ok());

        options.chunk_size = Some(0);
        assert!(options.validate().is_err());

        options.chunk_size = Some(1024);
        options.timeout = Some(0);
        assert!(options.validate().is_err());
    }

    #[test]
    fn test_upload_result() {
        let result = UploadResult::new("QmExample".to_string(), 1024)
            .chunks(4)
            .duration_ms(5000)
            .verified(true);

        assert_eq!(result.cid, "QmExample");
        assert_eq!(result.size, 1024);
        assert_eq!(result.chunks, Some(4));
        assert_eq!(result.duration_ms, 5000);
        assert!(result.verified);
    }
}

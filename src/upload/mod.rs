//! Upload operations for Codex
//!
//! This module provides comprehensive upload functionality for the Codex distributed storage network.
//! It supports both high-level and low-level upload operations, streaming uploads, and progress tracking.
//!
//! ## High-Level Operations
//!
//! - [`file::upload_file()`] - Upload a file from the filesystem
//! - [`file::upload_reader()`] - Upload data from any Read implementation
//!
//! ## Low-Level Operations
//!
//! - [`session::upload_init()`] - Initialize an upload session
//! - [`chunks::upload_chunk()`] - Upload a chunk of data
//! - [`session::upload_finalize()`] - Finalize an upload and get the CID
//! - [`session::upload_cancel()`] - Cancel an upload session
//!
//! ## Streaming Support
//!
//! - [`streaming::create_streaming_reader()`] - Create a streaming reader with progress tracking
//! - [`streaming::StreamingUploadReader`] - Sync streaming reader with progress callbacks
//! - [`streaming::AsyncStreamingUploadReader`] - Async streaming reader with progress callbacks
//!
//! ## Configuration
//!
//! - [`types::UploadOptions`] - Configure upload behavior including chunk size, verification, and progress callbacks
//! - [`types::UploadStrategy`] - Different strategies for upload optimization

pub mod chunks;
pub mod file;
pub mod session;
pub mod streaming;
pub mod types;

// Re-export types
pub use types::{UploadOptions, UploadProgress, UploadResult, UploadStrategy};

// Re-export streaming utilities
pub use streaming::{
    create_streaming_reader, AsyncStreamingUploadReader, StreamingUploadReader, UploadProgressExt,
};

// Re-export high-level file operations
pub use file::{upload_file, upload_reader};

// Re-export session management operations
pub use session::{upload_cancel, upload_finalize, upload_init};

// Re-export chunk operations
pub use chunks::{upload_chunk, upload_chunks};

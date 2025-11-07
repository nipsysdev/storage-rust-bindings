//! Download operations for Codex
//!
//! This module provides core download functionality that directly maps to the C API.
//! It includes operations for downloading content, managing download sessions,
//! and retrieving manifest information.
//!
//! ## Core Functions
//!
//! - [`download_init()`] - Initialize a download session
//! - [`download_chunk()`] - Download a chunk of content
//! - [`download_cancel()`] - Cancel an active download
//! - [`download_stream()`] - Stream content directly to a file or writer
//! - [`download_manifest()`] - Download manifest information for content

pub mod basic;
pub mod manifest;
pub mod streaming;
pub mod types;

// Re-export basic download operations
pub use basic::{download_cancel, download_chunk, download_init};

// Re-export manifest operations
pub use manifest::download_manifest;

// Re-export streaming operations
pub use streaming::{download_stream, DownloadProgressExt, StreamingDownloadWriter};

// Re-export types
pub use types::{
    DownloadOptions, DownloadProgress, DownloadResult, DownloadStreamOptions, Manifest,
};

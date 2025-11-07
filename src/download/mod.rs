//! Download operations for Codex
//!
//! This module provides functionality for downloading data from the Codex network
//! with support for streaming and chunk-based downloads with progress tracking.

pub mod basic;
pub mod manifest;
pub mod streaming;
pub mod types;

// Re-export basic download operations
pub use basic::{download_cancel, download_chunk, download_init};

// Re-export manifest operations
pub use manifest::{
    download_manifest, estimate_download_time, get_optimal_chunk_size, is_manifest_accessible,
    validate_manifest,
};

// Re-export streaming operations
pub use streaming::{download_stream, DownloadProgressExt, StreamingDownloadWriter};

// Re-export types
pub use types::{
    DownloadOptions, DownloadProgress, DownloadResult, DownloadStreamOptions, Manifest,
};

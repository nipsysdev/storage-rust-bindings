//! Download operations for Codex
//!
//! This module provides comprehensive download functionality for the Codex distributed storage network.
//! It supports both high-level and low-level download operations, streaming downloads, and manifest handling.
//!
//! ## High-Level Operations
//!
//! - [`stream::download_stream()`] - Download content to file or writer with progress tracking
//! - [`stream::download_to_file()`] - Download content directly to a file
//! - [`stream::download_to_writer()`] - Download content to any Write implementation
//!
//! ## Low-Level Operations
//!
//! - [`session::download_init()`] - Initialize a download session
//! - [`chunks::download_chunk()`] - Download a specific chunk of data
//! - [`session::download_cancel()`] - Cancel a download session
//!
//! ## Manifest Operations
//!
//! - [`manifest::download_manifest()`] - Download and parse manifest information
//!
//! ## Configuration
//!
//! - [`types::DownloadOptions`] - Configure download behavior including chunk size and timeout
//! - [`types::DownloadStreamOptions`] - Configure streaming downloads with output destinations
//! - [`types::Manifest`] - Manifest structure for metadata and content information

pub mod chunks;
pub mod manifest;
pub mod session;
pub mod stream;
pub mod types;

// Re-export types
pub use types::{
    DownloadOptions, DownloadProgress, DownloadResult, DownloadStreamOptions, Manifest,
};

// Re-export manifest operations
pub use manifest::download_manifest;

// Re-export session management operations
pub use session::{download_cancel, download_init};

// Re-export chunk operations
pub use chunks::{download_chunk, download_chunk_with_progress, download_chunks};

// Re-export stream operations
pub use stream::{download_stream, download_to_file, download_to_writer};

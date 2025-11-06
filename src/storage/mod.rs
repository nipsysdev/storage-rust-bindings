//! Storage management operations for Codex
//!
//! This module provides functionality for managing stored content,
//! including listing, fetching, deleting, and checking existence of content.

pub mod operations;

pub use operations::{Manifest, Space, delete, exists, fetch, manifests, space};

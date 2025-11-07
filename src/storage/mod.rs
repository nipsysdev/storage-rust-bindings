//! Storage management operations for Codex
//!
//! This module provides core storage functionality that directly maps to the C API.
//! It includes operations for listing manifests, managing storage space, and basic
//! CRUD operations for stored content.
//!
//! ## Core Functions
//!
//! - [`manifests()`] - List all manifests stored by the node
//! - [`space()`] - Get storage space information
//! - [`fetch()`] - Fetch manifest information for specific content
//! - [`delete()`] - Delete content from storage
//! - [`exists()`] - Check if content exists in storage

pub mod crud;
pub mod space;
pub mod types;

// Re-export CRUD operations
pub use crud::{delete, exists, fetch};

// Re-export space management operations
pub use space::{manifests, space, Manifest, ManifestWithCid, Space};

// Re-export types
pub use types::Manifest as StorageManifest;

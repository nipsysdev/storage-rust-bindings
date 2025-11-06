//! Node management for Codex
//!
//! This module provides functionality for creating, configuring, starting,
//! stopping, and destroying Codex nodes.

pub mod config;
pub mod lifecycle;

pub use config::{CodexConfig, LogFormat, LogLevel, RepoKind};
pub use lifecycle::CodexNode;

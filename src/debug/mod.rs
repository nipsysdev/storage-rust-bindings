//! Debug operations for Codex
//!
//! This module provides functionality for debugging and diagnostics,
//! including getting node info, updating log levels, and peer debugging.

pub mod node;
pub mod peer;

// Re-export node debugging operations
pub use node::{debug, update_log_level, DebugInfo, LogLevel};

// Re-export peer debugging operations
pub use peer::peer_debug;

// Re-export types from p2p module
pub use crate::p2p::{ConnectionQuality, PeerRecord};

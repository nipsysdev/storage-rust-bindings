//! Debug operations for Codex
//!
//! This module provides functionality for debugging and diagnostics,
//! including getting node info, updating log levels, and peer debugging.

pub mod operations;

pub use operations::{
    debug, network_stats, peer_debug, update_log_level, DebugInfo, LogLevel, PeerRecord,
};

//! Debug operations for Storage
//!
//! This module provides comprehensive debugging and diagnostics functionality for Storage nodes.
//! It includes operations for getting node information, updating log levels, and peer debugging.
//!
//! ## Node Debugging
//!
//! - [`debug()`] - Get comprehensive debug information about the node
//! - [`update_log_level()`] - Dynamically update the node's log level
//!
//! ## Peer Debugging
//!
//! - [`peer_debug()`] - Get detailed information about a specific peer
//!
//! ## Types
//!
//! - [`DebugInfo`] - Comprehensive node debug information including network status
//! - [`LogLevel`] - Enum for different log levels (Trace, Debug, Info, etc.)
//! - [`PeerRecord`] - Detailed peer information for debugging

pub mod node;
pub mod peer;

// Re-export node debugging operations
pub use node::{debug, update_log_level, DebugInfo, LogLevel};

// Re-export peer debugging operations
pub use peer::peer_debug;

// Re-export types from p2p module
pub use crate::p2p::{ConnectionQuality, PeerRecord};

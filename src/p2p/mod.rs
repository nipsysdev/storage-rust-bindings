//! P2P operations for Codex
//!
//! This module provides functionality for connecting to peers in the Codex network.

pub mod connection;
pub mod discovery;
pub mod types;

// Re-export connection operations
pub use connection::{connect, connect_to_multiple, validate_addresses, validate_peer_id};

// Re-export discovery operations
pub use discovery::{get_peer_id, get_peer_info};

// Re-export types
pub use types::{ConnectionQuality, PeerInfo, PeerRecord};

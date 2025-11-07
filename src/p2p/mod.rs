//! P2P operations for Codex
//!
//! This module provides functionality for connecting to peers in the Codex network.

pub mod connection;
pub mod discovery;
pub mod types;

// Re-export connection operations
pub use connection::{
    connect, connect_to_multiple, connection_stats, disconnect, validate_addresses,
    validate_peer_id, ConnectionStats,
};

// Re-export discovery operations
pub use discovery::{
    discover_peers_by_protocol, get_peer_id, get_peer_info, get_peer_reputation, list_peers,
    network_stats, search_peers, PeerReputation, ReputationLevel,
};

// Re-export types
pub use types::{PeerInfo, PeerRecord, PeerSearchCriteria};

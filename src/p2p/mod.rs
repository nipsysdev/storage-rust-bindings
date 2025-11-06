//! P2P operations for Codex
//!
//! This module provides functionality for connecting to peers in the Codex network.

pub mod operations;

pub use operations::{
    connect, disconnect, get_peer_id, get_peer_info, list_peers, PeerInfo, PeerRecord,
};

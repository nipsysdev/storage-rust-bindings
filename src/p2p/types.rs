//! Types for P2P operations

use serde::{Deserialize, Serialize};

/// Information about a peer in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer ID
    pub id: String,
    /// Multiaddresses of the peer
    pub addresses: Vec<String>,
    /// Whether the peer is connected
    pub connected: bool,
    /// Connection direction (inbound/outbound)
    pub direction: Option<String>,
    /// Latency to the peer (in milliseconds)
    pub latency_ms: Option<u64>,
}

impl PeerInfo {
    /// Create a new peer info
    pub fn new(id: String) -> Self {
        Self {
            id,
            addresses: Vec::new(),
            connected: false,
            direction: None,
            latency_ms: None,
        }
    }

    /// Set the addresses
    pub fn addresses(mut self, addresses: Vec<String>) -> Self {
        self.addresses = addresses;
        self
    }

    /// Set the connection status
    pub fn connected(mut self, connected: bool) -> Self {
        self.connected = connected;
        self
    }

    /// Set the connection direction
    pub fn direction(mut self, direction: String) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Set the latency
    pub fn latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }

    /// Check if the peer is reachable (has addresses and is connected)
    pub fn is_reachable(&self) -> bool {
        self.connected && !self.addresses.is_empty()
    }

    /// Get the primary address (first one in the list)
    pub fn primary_address(&self) -> Option<&String> {
        self.addresses.first()
    }

    /// Check if the connection is inbound
    pub fn is_inbound(&self) -> bool {
        self.direction.as_ref().map_or(false, |d| d == "inbound")
    }

    /// Check if the connection is outbound
    pub fn is_outbound(&self) -> bool {
        self.direction.as_ref().map_or(false, |d| d == "outbound")
    }

    /// Get a human-readable latency string
    pub fn latency_string(&self) -> String {
        match self.latency_ms {
            Some(latency) => format!("{}ms", latency),
            None => "Unknown".to_string(),
        }
    }
}

/// Detailed peer record for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerRecord {
    /// Peer ID
    pub id: String,
    /// Multiaddresses of the peer
    pub addresses: Vec<String>,
    /// Connection state
    pub connected: bool,
    /// Connection direction
    pub direction: Option<String>,
    /// Latency information
    pub latency_ms: Option<u64>,
    /// Protocols supported by the peer
    pub protocols: Vec<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Last seen timestamp
    pub last_seen: Option<String>,
    /// Connection duration in seconds
    pub connection_duration_seconds: Option<u64>,
    /// Bytes sent to this peer
    pub bytes_sent: Option<u64>,
    /// Bytes received from this peer
    pub bytes_received: Option<u64>,
    /// Additional peer metadata
    pub metadata: Option<serde_json::Value>,
}

impl PeerRecord {
    /// Create a new peer record
    pub fn new(id: String) -> Self {
        Self {
            id,
            addresses: Vec::new(),
            connected: false,
            direction: None,
            latency_ms: None,
            protocols: Vec::new(),
            user_agent: None,
            last_seen: None,
            connection_duration_seconds: None,
            bytes_sent: None,
            bytes_received: None,
            metadata: None,
        }
    }

    /// Set the addresses
    pub fn addresses(mut self, addresses: Vec<String>) -> Self {
        self.addresses = addresses;
        self
    }

    /// Set the connection status
    pub fn connected(mut self, connected: bool) -> Self {
        self.connected = connected;
        self
    }

    /// Set the connection direction
    pub fn direction(mut self, direction: String) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Set the latency
    pub fn latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }

    /// Set the protocols
    pub fn protocols(mut self, protocols: Vec<String>) -> Self {
        self.protocols = protocols;
        self
    }

    /// Set the user agent
    pub fn user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// Set the last seen time
    pub fn last_seen(mut self, last_seen: String) -> Self {
        self.last_seen = Some(last_seen);
        self
    }

    /// Set the connection duration
    pub fn connection_duration(mut self, duration_seconds: u64) -> Self {
        self.connection_duration_seconds = Some(duration_seconds);
        self
    }

    /// Set the bytes sent
    pub fn bytes_sent(mut self, bytes_sent: u64) -> Self {
        self.bytes_sent = Some(bytes_sent);
        self
    }

    /// Set the bytes received
    pub fn bytes_received(mut self, bytes_received: u64) -> Self {
        self.bytes_received = Some(bytes_received);
        self
    }

    /// Set the metadata
    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Get the total bytes transferred
    pub fn total_bytes(&self) -> u64 {
        self.bytes_sent.unwrap_or(0) + self.bytes_received.unwrap_or(0)
    }

    /// Check if the peer supports a specific protocol
    pub fn supports_protocol(&self, protocol: &str) -> bool {
        self.protocols.contains(&protocol.to_string())
    }

    /// Get the connection duration as a human-readable string
    pub fn duration_string(&self) -> String {
        match self.connection_duration_seconds {
            Some(seconds) => {
                if seconds < 60 {
                    format!("{}s", seconds)
                } else if seconds < 3600 {
                    format!("{}m {}s", seconds / 60, seconds % 60)
                } else {
                    format!(
                        "{}h {}m {}s",
                        seconds / 3600,
                        (seconds % 3600) / 60,
                        seconds % 60
                    )
                }
            }
            None => "Unknown".to_string(),
        }
    }

    /// Get a human-readable size string for bytes transferred
    pub fn bytes_string(&self) -> String {
        let total = self.total_bytes();
        if total < 1024 {
            format!("{}B", total)
        } else if total < 1024 * 1024 {
            format!("{:.1}KB", total as f64 / 1024.0)
        } else if total < 1024 * 1024 * 1024 {
            format!("{:.1}MB", total as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1}GB", total as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Convert to a PeerInfo (simplified version)
    pub fn to_peer_info(&self) -> PeerInfo {
        PeerInfo {
            id: self.id.clone(),
            addresses: self.addresses.clone(),
            connected: self.connected,
            direction: self.direction.clone(),
            latency_ms: self.latency_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_info_creation() {
        let peer_info = PeerInfo::new("12D3KooWExamplePeer".to_string())
            .addresses(vec!["/ip4/192.168.1.100/tcp/8080".to_string()])
            .connected(true)
            .direction("outbound".to_string())
            .latency(50);

        assert_eq!(peer_info.id, "12D3KooWExamplePeer");
        assert_eq!(peer_info.addresses.len(), 1);
        assert!(peer_info.connected);
        assert_eq!(peer_info.direction, Some("outbound".to_string()));
        assert_eq!(peer_info.latency_ms, Some(50));
    }

    #[test]
    fn test_peer_info_methods() {
        let peer_info = PeerInfo::new("12D3KooWExamplePeer".to_string())
            .addresses(vec!["/ip4/192.168.1.100/tcp/8080".to_string()])
            .connected(true)
            .direction("outbound".to_string())
            .latency(50);

        assert!(peer_info.is_reachable());
        assert_eq!(
            peer_info.primary_address(),
            Some(&"/ip4/192.168.1.100/tcp/8080".to_string())
        );
        assert!(peer_info.is_outbound());
        assert!(!peer_info.is_inbound());
        assert_eq!(peer_info.latency_string(), "50ms");
    }

    #[test]
    fn test_peer_record_creation() {
        let peer_record = PeerRecord::new("12D3KooWExamplePeer".to_string())
            .addresses(vec!["/ip4/192.168.1.100/tcp/8080".to_string()])
            .connected(true)
            .direction("outbound".to_string())
            .latency(50)
            .protocols(vec!["/codex/1.0.0".to_string()])
            .user_agent("codex-rust-bindings/0.1.0".to_string())
            .last_seen("2023-01-01T12:00:00Z".to_string())
            .connection_duration(1800)
            .bytes_sent(1024 * 1024)
            .bytes_received(2 * 1024 * 1024)
            .metadata(serde_json::json!({"version": "1.0.0"}));

        assert_eq!(peer_record.id, "12D3KooWExamplePeer");
        assert_eq!(peer_record.addresses.len(), 1);
        assert!(peer_record.connected);
        assert_eq!(peer_record.latency_ms, Some(50));
        assert_eq!(peer_record.protocols.len(), 1);
        assert!(peer_record.supports_protocol("/codex/1.0.0"));
        assert_eq!(peer_record.total_bytes(), 3 * 1024 * 1024);
        assert_eq!(peer_record.duration_string(), "30m 0s");
        assert_eq!(peer_record.bytes_string(), "3.0MB");
    }

    #[test]
    fn test_serialization() {
        let peer_info = PeerInfo::new("12D3KooWExamplePeer".to_string())
            .addresses(vec!["/ip4/192.168.1.100/tcp/8080".to_string()])
            .connected(true)
            .direction("outbound".to_string())
            .latency(50);

        let json = serde_json::to_string(&peer_info).unwrap();
        let deserialized: PeerInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(peer_info.id, deserialized.id);
        assert_eq!(peer_info.addresses, deserialized.addresses);
        assert_eq!(peer_info.connected, deserialized.connected);
        assert_eq!(peer_info.direction, deserialized.direction);
        assert_eq!(peer_info.latency_ms, deserialized.latency_ms);
    }
}

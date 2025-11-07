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

/// Connection statistics
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    /// Number of active connections
    pub active_connections: usize,
    /// Total number of connections attempted
    pub total_connections: usize,
    /// Number of failed connections
    pub failed_connections: usize,
    /// Average latency in milliseconds
    pub average_latency_ms: f64,
    /// Last successful connection time
    pub last_connection_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl ConnectionStats {
    /// Create new connection stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the active connections
    pub fn active_connections(mut self, count: usize) -> Self {
        self.active_connections = count;
        self
    }

    /// Set the total connections
    pub fn total_connections(mut self, count: usize) -> Self {
        self.total_connections = count;
        self
    }

    /// Set the failed connections
    pub fn failed_connections(mut self, count: usize) -> Self {
        self.failed_connections = count;
        self
    }

    /// Set the average latency
    pub fn average_latency(mut self, latency_ms: f64) -> Self {
        self.average_latency_ms = latency_ms;
        self
    }

    /// Set the last connection time
    pub fn last_connection_time(mut self, time: chrono::DateTime<chrono::Utc>) -> Self {
        self.last_connection_time = Some(time);
        self
    }

    /// Get the success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_connections == 0 {
            0.0
        } else {
            (self.total_connections - self.failed_connections) as f64
                / self.total_connections as f64
                * 100.0
        }
    }

    /// Get the failure rate as a percentage
    pub fn failure_rate(&self) -> f64 {
        if self.total_connections == 0 {
            0.0
        } else {
            self.failed_connections as f64 / self.total_connections as f64 * 100.0
        }
    }

    /// Get a human-readable latency string
    pub fn latency_string(&self) -> String {
        if self.average_latency_ms < 1.0 {
            format!("{:.2}ms", self.average_latency_ms)
        } else {
            format!("{:.1}ms", self.average_latency_ms)
        }
    }
}

impl Default for ConnectionStats {
    fn default() -> Self {
        Self {
            active_connections: 0,
            total_connections: 0,
            failed_connections: 0,
            average_latency_ms: 0.0,
            last_connection_time: None,
        }
    }
}

/// Peer reputation information
#[derive(Debug, Clone)]
pub struct PeerReputation {
    /// Peer ID
    pub peer_id: String,
    /// Reputation score (0.0 to 1.0)
    pub score: f64,
    /// Number of successful interactions
    pub successful_interactions: u64,
    /// Number of failed interactions
    pub failed_interactions: u64,
    /// Last interaction time
    pub last_interaction: Option<chrono::DateTime<chrono::Utc>>,
    /// Reputation level
    pub reputation_level: ReputationLevel,
}

/// Reputation level
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReputationLevel {
    /// Very poor reputation
    VeryPoor,
    /// Poor reputation
    Poor,
    /// Neutral reputation
    Neutral,
    /// Good reputation
    Good,
    /// Excellent reputation
    Excellent,
}

impl PeerReputation {
    /// Create a new peer reputation
    pub fn new(peer_id: String) -> Self {
        Self {
            peer_id,
            score: 0.5,
            successful_interactions: 0,
            failed_interactions: 0,
            last_interaction: None,
            reputation_level: ReputationLevel::Neutral,
        }
    }

    /// Set the score
    pub fn score(mut self, score: f64) -> Self {
        self.score = score.clamp(0.0, 1.0);
        self.reputation_level = self.calculate_reputation_level();
        self
    }

    /// Set the successful interactions
    pub fn successful_interactions(mut self, count: u64) -> Self {
        self.successful_interactions = count;
        self
    }

    /// Set the failed interactions
    pub fn failed_interactions(mut self, count: u64) -> Self {
        self.failed_interactions = count;
        self
    }

    /// Set the last interaction time
    pub fn last_interaction(mut self, time: chrono::DateTime<chrono::Utc>) -> Self {
        self.last_interaction = Some(time);
        self
    }

    /// Get the total number of interactions
    pub fn total_interactions(&self) -> u64 {
        self.successful_interactions + self.failed_interactions
    }

    /// Get the success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        let total = self.total_interactions();
        if total == 0 {
            0.0
        } else {
            self.successful_interactions as f64 / total as f64 * 100.0
        }
    }

    /// Update reputation based on a new interaction
    pub fn update_interaction(&mut self, success: bool) {
        if success {
            self.successful_interactions += 1;
        } else {
            self.failed_interactions += 1;
        }

        // Update score and level
        self.score = self.success_rate() / 100.0;
        self.reputation_level = self.calculate_reputation_level();
        self.last_interaction = Some(chrono::Utc::now());
    }

    fn calculate_reputation_level(&self) -> ReputationLevel {
        if self.score >= 0.9 {
            ReputationLevel::Excellent
        } else if self.score >= 0.7 {
            ReputationLevel::Good
        } else if self.score >= 0.3 {
            ReputationLevel::Neutral
        } else if self.score >= 0.1 {
            ReputationLevel::Poor
        } else {
            ReputationLevel::VeryPoor
        }
    }

    /// Get a human-readable reputation string
    pub fn reputation_string(&self) -> &'static str {
        match self.reputation_level {
            ReputationLevel::VeryPoor => "Very Poor",
            ReputationLevel::Poor => "Poor",
            ReputationLevel::Neutral => "Neutral",
            ReputationLevel::Good => "Good",
            ReputationLevel::Excellent => "Excellent",
        }
    }
}

/// Peer search criteria
#[derive(Debug, Clone, Default)]
pub struct PeerSearchCriteria {
    /// Minimum number of connections
    pub min_connections: usize,
    /// Maximum number of connections
    pub max_connections: usize,
    /// Required protocols
    pub required_protocols: Vec<String>,
    /// Exclude protocols
    pub exclude_protocols: Vec<String>,
    /// Maximum latency in milliseconds
    pub max_latency_ms: Option<u64>,
    /// Minimum reputation score
    pub min_reputation: Option<f64>,
    /// Limit the number of results
    pub limit: Option<usize>,
}

impl PeerSearchCriteria {
    /// Create new search criteria
    pub fn new() -> Self {
        Self::default()
    }

    /// Set minimum connections
    pub fn min_connections(mut self, min: usize) -> Self {
        self.min_connections = min;
        self
    }

    /// Set maximum connections
    pub fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = max;
        self
    }

    /// Add a required protocol
    pub fn require_protocol(mut self, protocol: String) -> Self {
        self.required_protocols.push(protocol);
        self
    }

    /// Add an excluded protocol
    pub fn exclude_protocol(mut self, protocol: String) -> Self {
        self.exclude_protocols.push(protocol);
        self
    }

    /// Set maximum latency
    pub fn max_latency(mut self, latency_ms: u64) -> Self {
        self.max_latency_ms = Some(latency_ms);
        self
    }

    /// Set minimum reputation
    pub fn min_reputation(mut self, score: f64) -> Self {
        self.min_reputation = Some(score);
        self
    }

    /// Set result limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Check if a peer matches this criteria
    pub fn matches(&self, peer: &PeerRecord) -> bool {
        // Check connection count (this would need to be tracked elsewhere)
        // For now, we'll skip this check

        // Check required protocols
        for protocol in &self.required_protocols {
            if !peer.supports_protocol(protocol) {
                return false;
            }
        }

        // Check excluded protocols
        for protocol in &self.exclude_protocols {
            if peer.supports_protocol(protocol) {
                return false;
            }
        }

        // Check latency
        if let Some(max_latency) = self.max_latency_ms {
            if let Some(latency) = peer.latency_ms {
                if latency > max_latency {
                    return false;
                }
            } else {
                return false; // Unknown latency might be too high
            }
        }

        // Check reputation (this would need to be tracked elsewhere)
        // For now, we'll skip this check

        true
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
    fn test_connection_stats() {
        let stats = ConnectionStats::new()
            .active_connections(5)
            .total_connections(10)
            .failed_connections(2)
            .average_latency(50.5)
            .last_connection_time(chrono::Utc::now());

        assert_eq!(stats.active_connections, 5);
        assert_eq!(stats.total_connections, 10);
        assert_eq!(stats.failed_connections, 2);
        assert_eq!(stats.success_rate(), 80.0);
        assert_eq!(stats.failure_rate(), 20.0);
        assert_eq!(stats.latency_string(), "50.5ms");
    }

    #[test]
    fn test_peer_reputation() {
        let mut reputation = PeerReputation::new("12D3KooWExamplePeer".to_string())
            .score(0.7)
            .successful_interactions(7)
            .failed_interactions(3);

        assert_eq!(reputation.peer_id, "12D3KooWExamplePeer");
        assert_eq!(reputation.score, 0.7);
        assert_eq!(reputation.successful_interactions, 7);
        assert_eq!(reputation.failed_interactions, 3);
        assert_eq!(reputation.total_interactions(), 10);
        assert_eq!(reputation.success_rate(), 70.0);
        assert_eq!(reputation.reputation_level, ReputationLevel::Good);
        assert_eq!(reputation.reputation_string(), "Good");

        // Update with successful interaction
        reputation.update_interaction(true);
        assert_eq!(reputation.successful_interactions, 8);
        assert_eq!(reputation.total_interactions(), 11);
        assert!((reputation.success_rate() - 72.727).abs() < 0.01);
    }

    #[test]
    fn test_peer_search_criteria() {
        let criteria = PeerSearchCriteria::new()
            .min_connections(1)
            .max_connections(10)
            .require_protocol("/codex/1.0.0".to_string())
            .exclude_protocol("/legacy/1.0.0".to_string())
            .max_latency(1000)
            .min_reputation(0.5)
            .limit(50);

        assert_eq!(criteria.min_connections, 1);
        assert_eq!(criteria.max_connections, 10);
        assert_eq!(criteria.required_protocols.len(), 1);
        assert_eq!(criteria.exclude_protocols.len(), 1);
        assert_eq!(criteria.max_latency_ms, Some(1000));
        assert_eq!(criteria.min_reputation, Some(0.5));
        assert_eq!(criteria.limit, Some(50));

        let peer = PeerRecord::new("12D3KooWExamplePeer".to_string())
            .protocols(vec!["/codex/1.0.0".to_string()])
            .latency(500);

        assert!(criteria.matches(&peer));

        let peer_without_protocol = PeerRecord::new("12D3KooWExamplePeer2".to_string())
            .protocols(vec!["/other/1.0.0".to_string()])
            .latency(500);

        assert!(!criteria.matches(&peer_without_protocol));
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

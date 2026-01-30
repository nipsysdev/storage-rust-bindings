//! Type-safe wrappers for common storage types

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

/// Content Identifier (CID) for Logos Storage
///
/// CIDs are used to uniquely identify content in the storage system.
/// They follow the CIDv1 specification with base32 encoding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cid(String);

impl Cid {
    /// Creates a new CID from a string without validation
    ///
    /// # Safety
    ///
    /// This method does not validate the CID format. Use `from_str()` for
    /// validated CID creation.
    pub fn new(cid: String) -> Self {
        Self(cid)
    }

    /// Returns the CID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the CID and returns the inner String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl FromStr for Cid {
    type Err = CidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Validate CID format (CIDv1 with base32 encoding)
        if !s.starts_with('z') {
            return Err(CidError::InvalidFormat("CID must start with 'z'".into()));
        }

        // Basic validation: CID should be at least 2 characters
        if s.len() < 2 {
            return Err(CidError::InvalidFormat("CID is too short".into()));
        }

        // Validate base32 characters (after the 'z' prefix)
        // CIDv1 base32 uses lowercase letters, but we'll be lenient and accept uppercase too
        let base32_chars = &s[1..];
        if !base32_chars
            .chars()
            .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '2'..='7' | '='))
        {
            return Err(CidError::InvalidEncoding("Invalid base32 encoding".into()));
        }

        Ok(Cid(s.to_string()))
    }
}

impl Display for Cid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Cid {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<Cid> for String {
    fn from(cid: Cid) -> Self {
        cid.0
    }
}

/// Peer ID for Logos Storage
///
/// Peer IDs are used to identify peers in the P2P network.
/// They use base58 encoding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(String);

impl PeerId {
    /// Creates a new Peer ID from a string without validation
    ///
    /// # Safety
    ///
    /// This method does not validate the Peer ID format. Use `from_str()` for
    /// validated Peer ID creation.
    pub fn new(peer_id: String) -> Self {
        Self(peer_id)
    }

    /// Returns the Peer ID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the Peer ID and returns the inner String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl FromStr for PeerId {
    type Err = PeerIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Validate base58 encoding
        if !is_valid_base58(s) {
            return Err(PeerIdError::InvalidEncoding(
                "Invalid base58 encoding".into(),
            ));
        }

        Ok(PeerId(s.to_string()))
    }
}

impl Display for PeerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for PeerId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<PeerId> for String {
    fn from(peer_id: PeerId) -> Self {
        peer_id.0
    }
}

/// MultiAddress for Logos Storage
///
/// MultiAddresses are used to represent network addresses in the P2P network.
/// They follow the multiaddr specification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MultiAddress(String);

impl MultiAddress {
    /// Creates a new MultiAddress from a string without validation
    ///
    /// # Safety
    ///
    /// This method does not validate the MultiAddress format. Use `from_str()` for
    /// validated MultiAddress creation.
    pub fn new(addr: String) -> Self {
        Self(addr)
    }

    /// Returns the MultiAddress as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the MultiAddress and returns the inner String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl FromStr for MultiAddress {
    type Err = MultiAddrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Validate multiaddr format (must start with '/')
        if !s.starts_with('/') {
            return Err(MultiAddrError::InvalidFormat(
                "MultiAddress must start with '/'".into(),
            ));
        }

        Ok(MultiAddress(s.to_string()))
    }
}

impl Display for MultiAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for MultiAddress {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<MultiAddress> for String {
    fn from(addr: MultiAddress) -> Self {
        addr.0
    }
}

// Helper function to validate base58 encoding
fn is_valid_base58(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    s.chars().all(|c| {
        matches!(
            c,
            '1' | '2'
                | '3'
                | '4'
                | '5'
                | '6'
                | '7'
                | '8'
                | '9'
                | 'A'
                | 'B'
                | 'C'
                | 'D'
                | 'E'
                | 'F'
                | 'G'
                | 'H'
                | 'J'
                | 'K'
                | 'L'
                | 'M'
                | 'N'
                | 'P'
                | 'Q'
                | 'R'
                | 'S'
                | 'T'
                | 'U'
                | 'V'
                | 'W'
                | 'X'
                | 'Y'
                | 'Z'
                | 'a'
                | 'b'
                | 'c'
                | 'd'
                | 'e'
                | 'f'
                | 'g'
                | 'h'
                | 'i'
                | 'j'
                | 'k'
                | 'm'
                | 'n'
                | 'o'
                | 'p'
                | 'q'
                | 'r'
                | 's'
                | 't'
                | 'u'
                | 'v'
                | 'w'
                | 'x'
                | 'y'
                | 'z'
        )
    })
}

// Error types

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CidError {
    InvalidFormat(String),
    InvalidEncoding(String),
}

impl fmt::Display for CidError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CidError::InvalidFormat(msg) => write!(f, "Invalid CID format: {}", msg),
            CidError::InvalidEncoding(msg) => write!(f, "Invalid CID encoding: {}", msg),
        }
    }
}

impl std::error::Error for CidError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerIdError {
    InvalidEncoding(String),
}

impl fmt::Display for PeerIdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            PeerIdError::InvalidEncoding(msg) => write!(f, "Invalid Peer ID encoding: {}", msg),
        }
    }
}

impl std::error::Error for PeerIdError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MultiAddrError {
    InvalidFormat(String),
}

impl fmt::Display for MultiAddrError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MultiAddrError::InvalidFormat(msg) => write!(f, "Invalid MultiAddress format: {}", msg),
        }
    }
}

impl std::error::Error for MultiAddrError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cid_from_str_valid() {
        let cid = Cid::from_str("zabc23def456").unwrap();
        assert_eq!(cid.as_str(), "zabc23def456");
    }

    #[test]
    fn test_cid_from_str_invalid_format() {
        let result = Cid::from_str("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_cid_display() {
        let cid = Cid::from_str("zabc23def456").unwrap();
        assert_eq!(cid.to_string(), "zabc23def456");
    }

    #[test]
    fn test_peer_id_from_str_valid() {
        let peer_id = PeerId::from_str("12D3KooW").unwrap();
        assert_eq!(peer_id.as_str(), "12D3KooW");
    }

    #[test]
    fn test_peer_id_from_str_invalid() {
        let result = PeerId::from_str("invalid@peer");
        assert!(result.is_err());
    }

    #[test]
    fn test_multiaddress_from_str_valid() {
        let addr = MultiAddress::from_str("/ip4/127.0.0.1/tcp/1234").unwrap();
        assert_eq!(addr.as_str(), "/ip4/127.0.0.1/tcp/1234");
    }

    #[test]
    fn test_multiaddress_from_str_invalid() {
        let result = MultiAddress::from_str("invalid");
        assert!(result.is_err());
    }
}

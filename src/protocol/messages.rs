use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkMessage {
    OnionPacket(Vec<u8>),
    CircuitBuild(String),
    FragmentStore(Vec<u8>),
    FragmentRequest(String),
    Heartbeat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub addresses: Vec<String>,
    pub last_seen: u64,
}

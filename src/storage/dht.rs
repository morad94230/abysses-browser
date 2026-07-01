use serde::{Deserialize, Serialize};

pub fn fragment_key(root_hash: &[u8; 32], index: u8) -> String {
    format!("abyss:fragment:{}:{:02x}", hex::encode(root_hash), index)
}

pub fn name_key(name: &str) -> String {
    format!("abyss:name:{}", name)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NameRecord {
    pub name: String,
    pub root_hash: [u8; 32],
    pub owner: String,
    pub owner_public_key: Vec<u8>,
    pub created_at: u64,
    pub ttl: u32,
    pub signature: Vec<u8>,
    pub pow_nonce: u64,
    pub pow_difficulty: u8,
}

pub struct DhtStorage {
    pub replication_factor: usize,
}

impl DhtStorage {
    pub fn new(replication_factor: usize) -> Self {
        Self { replication_factor }
    }
}

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainEntry {
    pub parent_hash: [u8; 32],
    pub entry_type: EntryType,
    pub data: Vec<u8>,
    pub timestamp: u64,
    pub nonce: u64,
    pub difficulty: u8,
    pub signature: [u8; 64],
    pub hash: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EntryType {
    NameRegistration { name: String, root_hash: [u8; 32], owner: String, ttl: u32 },
    NameUpdate { name: String, new_root_hash: [u8; 32] },
    NameRevocation { name: String },
}

pub struct SignedChain {
    pub entries: Vec<ChainEntry>,
    pub name_index: HashMap<String, usize>,
}

impl SignedChain {
    pub fn new() -> Self {
        Self { entries: vec![], name_index: HashMap::new() }
    }

    pub fn resolve(&self, name: &str) -> Option<[u8; 32]> {
        self.name_index.get(name).and_then(|&idx| match &self.entries[idx].entry_type {
            EntryType::NameRegistration { root_hash, .. } => Some(*root_hash),
            _ => None,
        })
    }
}
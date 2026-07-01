use reed_solomon_erasure::galois_8::ReedSolomon;
use serde::{Serialize, Deserialize};

pub const DATA_SHARDS: usize = 10;
pub const PARITY_SHARDS: usize = 5;
pub const TOTAL_SHARDS: usize = 15;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fragment {
    pub index: u8,
    pub data: Vec<u8>,
    pub hash: [u8; 32],
    pub root_hash: [u8; 32],
    pub ttl: u32,
    pub signature: [u8; 64],
    pub shard_index: usize,
}

#[derive(Clone, Debug)]
pub struct Content {
    pub data: Vec<u8>,
    pub metadata: ContentMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub name: String,
    pub description: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub owner: String,
    pub mime_type: String,
    pub tags: Vec<String>,
    pub original_size: usize,
}

pub struct Fragmenter;

impl Fragmenter {
    pub fn fragment(content: &Content, keypair: &ed25519_dalek::Keypair) -> Result<(Vec<Fragment>, [u8; 32]), String> {
        let data = &content.data;
        let chunk_size = (data.len() + DATA_SHARDS - 1) / DATA_SHARDS;
        let mut padded = data.clone();
        padded.resize(chunk_size * DATA_SHARDS, 0);
        let mut shards: Vec<Vec<u8>> = (0..DATA_SHARDS)
            .map(|i| padded[i*chunk_size..(i+1)*chunk_size].to_vec())
            .collect();
        for _ in 0..PARITY_SHARDS { shards.push(vec![0u8; chunk_size]); }
        let r = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS).map_err(|e| e.to_string())?;
        r.encode(&mut shards).map_err(|e| e.to_string())?;
        let mut hashes = Vec::new();
        for s in &shards { hashes.push(blake3::hash(s).into()); }
        let root = crate::storage::merkle::MerkleTree::new(&hashes).root();
        let fragments: Vec<Fragment> = shards.iter().enumerate().map(|(i, s)| {
            let sig = keypair.sign(&hashes[i]);
            Fragment {
                index: i as u8, data: s.clone(), hash: hashes[i], root_hash: root,
                ttl: 86400, signature: sig.to_bytes(), shard_index: i,
            }
        }).collect();
        Ok((fragments, root))
    }
}
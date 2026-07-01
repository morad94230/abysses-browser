use reed_solomon_erasure::galois_8::ReedSolomon;
use serde::{Serialize, Deserialize};
use ed25519_dalek::SigningKey;

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
    pub signature: Vec<u8>,
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
    pub fn fragment(content: &Content, keypair: &SigningKey) -> Result<(Vec<Fragment>, [u8; 32]), String> {
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
                ttl: 86400, signature: sig.to_vec(), shard_index: i,
            }
        }).collect();
        Ok((fragments, root))
    }

    pub fn reconstruct_fragments(fragments: &[Fragment]) -> Result<Content, String> {
        if fragments.len() < DATA_SHARDS {
            return Err("Not enough fragments".to_string());
        }
        let chunk_size = fragments[0].data.len();
        let mut shards: Vec<Option<Vec<u8>>> = vec![None; TOTAL_SHARDS];
        for f in fragments { shards[f.shard_index] = Some(f.data.clone()); }
        let r = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS).map_err(|e| e.to_string())?;
        r.reconstruct(&mut shards).map_err(|e| e.to_string())?;
        let mut data = Vec::new();
        for i in 0..DATA_SHARDS {
            if let Some(s) = &shards[i] { data.extend_from_slice(s); }
        }
        let mut end = data.len();
        while end > 0 && data[end-1] == 0 { end -= 1; }
        Ok(Content {
            data: data[..end].to_vec(),
            metadata: ContentMetadata {
                name: String::new(), description: String::new(),
                created_at: 0, expires_at: 0, owner: String::new(),
                mime_type: String::new(), tags: vec![], original_size: end,
            },
        })
    }
}
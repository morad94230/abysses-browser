use std::sync::atomic::{AtomicU64, Ordering};

use chacha20poly1305::aead::Aead;
use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce};
use serde::{Deserialize, Serialize};

use crate::error::AbyssError;

pub const ONION_LAYER_SIZE: usize = 1024;
pub const MAX_LAYERS: usize = 3;

static NONCE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn generate_secure_nonce() -> [u8; 12] {
    let counter = NONCE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let random: [u8; 12] = rand::random();
    let mut nonce = [0u8; 12];
    nonce[..8].copy_from_slice(&counter.to_le_bytes());
    nonce[8..].copy_from_slice(&random[8..]);
    for i in 0..12 {
        nonce[i] ^= random[i];
    }
    nonce
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnionLayerHeader {
    pub next_hop: String,
    pub nonce: [u8; 12],
    pub padding: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OnionLayer {
    pub header: OnionLayerHeader,
    pub encrypted_payload: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OnionPacket {
    pub layers: Vec<OnionLayer>,
    pub created_at: u64,
    pub ttl: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FinalPayload {
    HttpRequest {
        method: String,
        path: String,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
        target_hash: [u8; 32],
    },
    StoreRequest {
        fragment_hash: [u8; 32],
        fragment_data: Vec<u8>,
        ttl: u32,
    },
    RetrieveRequest {
        name: String,
        expected_hash: [u8; 32],
    },
    CoverTraffic {
        nonce: [u8; 32],
        padding: Vec<u8>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FinalResponse {
    HttpResponse {
        status: u16,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    },
    StoreAck {
        fragment_hash: [u8; 32],
        stored_by: Vec<String>,
    },
    CoverAck {
        nonce: [u8; 32],
    },
}

impl OnionPacket {
    pub fn build(
        hop1_id: &str,
        hop2_id: &str,
        hop3_id: &str,
        shared_keys: &[[u8; 32]; 3],
        payload: &FinalPayload,
    ) -> Result<Self, AbyssError> {
        let payload_bytes =
            bincode::serialize(payload).map_err(|e| AbyssError::Encryption(e.to_string()))?;
        let layer3 = Self::build_layer(hop3_id, "EXIT", &shared_keys[2], &payload_bytes)?;
        let layer2_bytes =
            bincode::serialize(&layer3).map_err(|e| AbyssError::Encryption(e.to_string()))?;
        let layer2 = Self::build_layer(hop2_id, hop3_id, &shared_keys[1], &layer2_bytes)?;
        let layer1_bytes =
            bincode::serialize(&layer2).map_err(|e| AbyssError::Encryption(e.to_string()))?;
        let layer1 = Self::build_layer(hop1_id, hop2_id, &shared_keys[0], &layer1_bytes)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Ok(Self {
            layers: vec![layer1, layer2, layer3],
            created_at: now,
            ttl: MAX_LAYERS as u8,
        })
    }

    fn build_layer(
        _current_hop: &str,
        next_hop: &str,
        shared_key: &[u8; 32],
        payload: &[u8],
    ) -> Result<OnionLayer, AbyssError> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(shared_key));
        let nonce = generate_secure_nonce();
        let encrypted = cipher
            .encrypt(Nonce::from_slice(&nonce), payload)
            .map_err(|e| AbyssError::Encryption(e.to_string()))?;
        Ok(OnionLayer {
            header: OnionLayerHeader {
                next_hop: next_hop.to_string(),
                nonce,
                padding: vec![0u8; ONION_LAYER_SIZE - 100 - encrypted.len()],
            },
            encrypted_payload: encrypted,
        })
    }
}

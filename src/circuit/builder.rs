use crate::protocol::onion::{OnionPacket, FinalPayload};
use crate::protocol::pheromone::PheromoneTable;
use crate::error::AbyssError;
use x25519_dalek::StaticSecret;
use rand::rngs::OsRng;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Default)]
pub struct CircuitKeys {
    pub e1: StaticSecret,
    pub shared1: [u8; 32],
    pub nonce2: [u8; 12],
    pub nonce3: [u8; 12],
}

#[derive(Clone)]
pub struct OnionCircuit {
    pub hop1: String,
    pub keys: CircuitKeys,
    pub created_at: u64,
    pub expires_at: u64,
    pub is_active: bool,
    pub message_count: u64,
    pub last_used: u64,
}

pub struct CircuitBuilder {
    pub pheromone_table: Arc<Mutex<PheromoneTable>>,
}

impl CircuitBuilder {
    pub fn new(pheromone_table: Arc<Mutex<PheromoneTable>>) -> Self {
        Self { pheromone_table }
    }

    pub async fn build_circuit(&self) -> Result<OnionCircuit, AbyssError> {
        let mut table = self.pheromone_table.lock().await;
        let hop1 = table.select_relay(&[]).ok_or(AbyssError::InsufficientNeighbors(0))?;
        drop(table);
        let e1 = StaticSecret::random_from_rng(OsRng);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        Ok(OnionCircuit {
            hop1,
            keys: CircuitKeys { e1, shared1: [0u8; 32], nonce2: [0u8; 12], nonce3: [0u8; 12] },
            created_at: now,
            expires_at: now + 600,
            is_active: true,
            message_count: 0,
            last_used: now,
        })
    }

    pub fn build_packet(&self, circuit: &OnionCircuit, payload: &FinalPayload) -> Result<OnionPacket, AbyssError> {
        let shared_keys = [circuit.keys.shared1, [0u8; 32], [0u8; 32]];
        OnionPacket::build(&circuit.hop1, "HOP2_UNKNOWN", "HOP3_UNKNOWN", &shared_keys, payload)
    }
}
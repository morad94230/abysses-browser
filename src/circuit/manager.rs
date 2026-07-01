use crate::circuit::builder::{OnionCircuit, CircuitBuilder};
use crate::circuit::health::{CircuitHealth, CircuitHealthChecker};
use crate::protocol::onion::FinalPayload;
use crate::AbyssError;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct CircuitManager {
    pub circuits: HashMap<String, OnionCircuit>,
    pub builder: CircuitBuilder,
    pub health_checker: CircuitHealthChecker,
    pub max_circuits: usize,
    pub active_circuit_id: Option<String>,
    pub max_messages_per_circuit: u64,
    pub max_circuit_lifetime_seconds: u64,
}

impl CircuitManager {
    pub fn new(builder: CircuitBuilder, health_checker: CircuitHealthChecker, max_circuits: usize) -> Self {
        Self {
            circuits: HashMap::new(),
            builder,
            health_checker,
            max_circuits,
            active_circuit_id: None,
            max_messages_per_circuit: 10,
            max_circuit_lifetime_seconds: 300,
        }
    }

    fn should_rotate(&self, circuit: &OnionCircuit) -> bool {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        circuit.message_count >= self.max_messages_per_circuit || now - circuit.created_at > self.max_circuit_lifetime_seconds
    }

    pub async fn get_or_create_circuit(&mut self) -> Result<String, AbyssError> {
        if let Some(ref id) = self.active_circuit_id {
            if let Some(c) = self.circuits.get(id) {
                if self.should_rotate(c) { self.circuits.remove(id); self.active_circuit_id = None; }
                else if self.health_checker.check(c) == CircuitHealth::Healthy { return Ok(id.clone()); }
            }
        }
        for (id, c) in &self.circuits {
            if self.health_checker.check(c) == CircuitHealth::Healthy { self.active_circuit_id = Some(id.clone()); return Ok(id.clone()); }
        }
        if self.circuits.len() >= self.max_circuits {
            if let Some(oldest) = self.circuits.keys().next().cloned() { self.circuits.remove(&oldest); }
        }
        let circuit = self.builder.build_circuit().await?;
        let id = format!("circuit_{}", self.circuits.len());
        self.circuits.insert(id.clone(), circuit);
        self.active_circuit_id = Some(id.clone());
        Ok(id)
    }

    pub fn get_random_circuit(&self) -> Option<String> {
        let healthy: Vec<_> = self.circuits.iter()
            .filter(|(_, c)| self.health_checker.check(c) == CircuitHealth::Healthy)
            .map(|(id, _)| id.clone()).collect();
        if healthy.is_empty() { return None; }
        Some(healthy[rand::random::<usize>() % healthy.len()].clone())
    }

    pub async fn send_via_circuit(&mut self, circuit_id: &str, payload: &FinalPayload) -> Result<Vec<u8>, AbyssError> {
        let circuit = self.circuits.get_mut(circuit_id).ok_or(AbyssError::NoActiveCircuit)?;
        circuit.message_count += 1;
        circuit.last_used = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let _packet = self.builder.build_packet(circuit, payload)?;
        Ok(vec![])
    }
}
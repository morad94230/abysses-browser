use crate::circuit::builder::OnionCircuit;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CircuitHealth {
    Healthy,
    Degraded,
    Failing,
    Dead,
}

pub struct CircuitHealthChecker {
    pub degraded_latency_threshold: u64,
    pub dead_latency_threshold: u64,
    pub failing_threshold: u32,
    pub dead_threshold: u32,
}

impl Default for CircuitHealthChecker {
    fn default() -> Self {
        Self {
            degraded_latency_threshold: 500,
            dead_latency_threshold: 5000,
            failing_threshold: 3,
            dead_threshold: 5,
        }
    }
}

impl CircuitHealthChecker {
    pub fn check(&self, circuit: &OnionCircuit) -> CircuitHealth {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if circuit.expires_at < now {
            return CircuitHealth::Dead;
        }
        if now - circuit.last_used > 300 && circuit.message_count > 0 {
            return CircuitHealth::Degraded;
        }
        CircuitHealth::Healthy
    }
}

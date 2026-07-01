use crate::circuit::manager::CircuitManager;
use crate::protocol::onion::FinalPayload;
use rand::Rng;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

pub struct CoverTrafficGenerator {
    pub mean_interval_ms: u64,
    pub circuit_manager: Arc<Mutex<CircuitManager>>,
    pub running: bool,
}

impl CoverTrafficGenerator {
    pub fn new(mean_interval_ms: u64, circuit_manager: Arc<Mutex<CircuitManager>>) -> Self {
        Self {
            mean_interval_ms,
            circuit_manager,
            running: false,
        }
    }

    pub async fn run(&mut self) {
        self.running = true;
        let mut rng = rand::thread_rng();
        while self.running {
            let base = self.mean_interval_ms as f64;
            let jitter = rng.r#gen::<f64>() * 1.0 + 0.5;
            let delay_ms = (-rng.r#gen::<f64>().ln() * base * jitter) as u64;
            sleep(Duration::from_millis(delay_ms)).await;
            let circuit_id = {
                let manager = self.circuit_manager.lock().await;
                manager.get_random_circuit()
            };
            let circuit_id = match circuit_id {
                Some(id) => id,
                None => {
                    let mut manager = self.circuit_manager.lock().await;
                    match manager.get_or_create_circuit().await {
                        Ok(id) => id,
                        Err(_) => continue,
                    }
                }
            };
            let cover = FinalPayload::CoverTraffic {
                nonce: rng.r#gen::<[u8; 32]>(),
                padding: vec![0u8; 900],
            };
            let mut manager = self.circuit_manager.lock().await;
            let _ = manager.send_via_circuit(&circuit_id, &cover).await;
        }
    }

    pub fn stop(&mut self) {
        self.running = false;
    }
}

use crate::protocol::pheromone::PheromoneTable;
use std::collections::HashMap;
use rand::Rng;

pub struct SimulatedNetwork {
    pub nodes: HashMap<String, SimulatedNode>,
    pub messages: Vec<SimulatedMessage>,
}

pub struct SimulatedNode {
    pub id: String,
    pub online: bool,
    pub pheromones: PheromoneTable,
    pub latency: u64,
}

#[derive(Clone, Debug)]
pub struct SimulatedMessage {
    pub id: String,
    pub from: String,
    pub to: String,
    pub path: Vec<String>,
    pub success: bool,
    pub latency: u64,
}

impl SimulatedNetwork {
    pub fn new() -> Self {
        Self { nodes: HashMap::new(), messages: vec![] }
    }

    pub fn add_node(&mut self, id: &str) {
        self.nodes.insert(id.to_string(), SimulatedNode {
            id: id.to_string(),
            online: true,
            pheromones: PheromoneTable::default(),
            latency: rand::thread_rng().r#gen::<u64>() % 180 + 20,
        });
    }

    pub fn kill_random(&mut self, pct: f64) {
        for node in self.nodes.values_mut() {
            if rand::thread_rng().r#gen::<f64>() < pct { node.online = false; }
        }
    }

    pub fn send(&mut self, from: &str, to: &str) -> SimulatedMessage {
        let mut path = vec![from.to_string()];
        let mut current = from.to_string();
        let mut total_latency = 0u64;
        let mut success = true;
        let mut rng = rand::thread_rng();
        for _ in 0..3 {
            if let Some(node) = self.nodes.get(&current) {
                if !node.online { success = false; break; }
                let exclude: Vec<String> = path.clone();
                let mut pt = node.pheromones.clone();
                let next = pt.select_relay(&exclude);
                if let Some(next_hop) = next {
                    if let Some(next_node) = self.nodes.get(&next_hop) {
                        if !next_node.online { success = false; break; }
                        total_latency += node.latency + next_node.latency + rng.r#gen::<u64>() % 45 + 5;
                        path.push(next_hop.clone());
                        current = next_hop;
                    } else { success = false; break; }
                } else { success = false; break; }
            } else { success = false; break; }
        }
        let msg = SimulatedMessage {
            id: format!("m{}", rng.r#gen::<u64>()),
            from: from.to_string(),
            to: to.to_string(),
            path,
            success,
            latency: total_latency,
        };
        self.messages.push(msg.clone());
        msg
    }
}
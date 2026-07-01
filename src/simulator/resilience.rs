use crate::simulator::network::{SimulatedNetwork, SimulatedMessage};

pub struct ResilienceTest {
    pub network: SimulatedNetwork,
}

impl ResilienceTest {
    pub fn new(num_nodes: usize) -> Self {
        let mut net = SimulatedNetwork::new();
        for i in 0..num_nodes { net.add_node(&format!("n{}", i)); }
        Self { network: net }
    }

    pub fn run(&mut self, pct: f64, msgs: usize) -> ResilienceReport {
        let nodes: Vec<String> = self.network.nodes.keys().cloned().collect();
        let mut before = vec![];
        for _ in 0..msgs / 2 {
            let a = &nodes[rand::random::<usize>() % nodes.len()];
            let b = &nodes[rand::random::<usize>() % nodes.len()];
            if a != b { before.push(self.network.send(a, b)); }
        }
        self.network.kill_random(pct);
        let mut after = vec![];
        for _ in 0..msgs / 2 {
            let a = &nodes[rand::random::<usize>() % nodes.len()];
            let b = &nodes[rand::random::<usize>() % nodes.len()];
            if a != b { after.push(self.network.send(a, b)); }
        }
        let before_rate = before.iter().filter(|m| m.success).count() as f64 / before.len().max(1) as f64;
        let after_rate = after.iter().filter(|m| m.success).count() as f64 / after.len().max(1) as f64;
        ResilienceReport {
            before_rate,
            after_rate,
            resilient: after_rate > 0.5,
            nodes_total: self.network.nodes.len(),
            nodes_killed: self.network.nodes.values().filter(|n| !n.online).count(),
        }
    }
}

pub struct ResilienceReport {
    pub before_rate: f64,
    pub after_rate: f64,
    pub resilient: bool,
    pub nodes_total: usize,
    pub nodes_killed: usize,
}

impl ResilienceReport {
    pub fn print(&self) {
        println!("Nodes: {} total, {} killed", self.nodes_total, self.nodes_killed);
        println!("Before: {:.1}% After: {:.1}%", self.before_rate * 100.0, self.after_rate * 100.0);
        println!("Resilience: {}", if self.resilient { "PASS" } else { "FAIL" });
    }
}
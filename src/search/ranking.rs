use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct TrustGraph {
    pub nodes: HashMap<String, TrustNode>,
}

#[derive(Clone, Debug)]
pub struct TrustNode {
    pub peer_id: String,
    pub trust_score: f64,
    pub endorsements: HashMap<String, f64>,
}

impl TrustGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn add_endorsement(&mut self, from: &str, to: &str, level: f64) {
        let node = self
            .nodes
            .entry(from.to_string())
            .or_insert_with(|| TrustNode {
                peer_id: from.to_string(),
                trust_score: 0.5,
                endorsements: HashMap::new(),
            });
        node.endorsements.insert(to.to_string(), level);

        self.nodes
            .entry(to.to_string())
            .or_insert_with(|| TrustNode {
                peer_id: to.to_string(),
                trust_score: 0.5,
                endorsements: HashMap::new(),
            });
    }

    pub fn calculate_trust(&mut self, peer: &str) -> f64 {
        let mut visited = HashMap::new();
        self.dfs_trust(peer, 1.0, 3, &mut visited);
        *visited.get(peer).unwrap_or(&0.5)
    }

    fn dfs_trust(&self, peer: &str, weight: f64, depth: usize, visited: &mut HashMap<String, f64>) {
        if depth == 0 || weight < 0.01 {
            return;
        }
        let current = visited.entry(peer.to_string()).or_insert(0.0);
        *current += weight;

        if let Some(node) = self.nodes.get(peer) {
            for (endorsed, level) in &node.endorsements {
                self.dfs_trust(endorsed, weight * level * 0.5, depth - 1, visited);
            }
        }
    }
}

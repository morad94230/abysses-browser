use std::collections::HashMap;
use std::time::Instant;
use rand::Rng;

#[derive(Clone, Debug)]
pub struct PheromoneScore {
    pub latency_ms: f64,
    pub success_rate: f64,
    pub success_count: u64,
    pub failure_count: u64,
    pub last_updated: Instant,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NeighborState { Healthy, Degraded, Unreachable, Blacklisted }

#[derive(Clone, Debug)]
pub struct PheromoneTable {
    pub scores: HashMap<String, PheromoneScore>,
    pub states: HashMap<String, NeighborState>,
    pub evaporation_rate: f64,
    pub exploration_threshold: f64,
    pub min_neighbors: usize,
    pub max_neighbors: usize,
    pub blacklist_threshold: u32,
}

impl Default for PheromoneTable {
    fn default() -> Self {
        Self {
            scores: HashMap::new(),
            states: HashMap::new(),
            evaporation_rate: 0.01,
            exploration_threshold: 0.15,
            min_neighbors: 20,
            max_neighbors: 200,
            blacklist_threshold: 5,
        }
    }
}

impl PheromoneScore {
    pub fn new() -> Self {
        Self { latency_ms: 100.0, success_rate: 0.5, success_count: 0, failure_count: 0, last_updated: Instant::now() }
    }

    pub fn composite_score(&self) -> f64 {
        let recency = (Instant::now().duration_since(self.last_updated).as_secs_f64() / 3600.0).min(1.0).max(0.1);
        self.success_rate * (-self.latency_ms / 1000.0).exp() * recency
    }
}

impl PheromoneTable {
    pub fn add_neighbor(&mut self, peer_id: String) {
        if self.scores.len() >= self.max_neighbors { self.evict_weakest(); }
        self.scores.entry(peer_id.clone()).or_insert_with(PheromoneScore::new);
        self.states.insert(peer_id, NeighborState::Healthy);
    }

    pub fn select_relay(&mut self, exclude: &[String]) -> Option<String> {
        let available: Vec<String> = self.scores.keys()
            .filter(|k| !exclude.contains(k))
            .filter(|k| self.states.get(*k) != Some(&NeighborState::Blacklisted))
            .cloned().collect();
        if available.is_empty() { return None; }
        let mut rng = rand::thread_rng();
        if rng.r#gen::<f64>() < self.exploration_threshold {
            return Some(available[rng.r#gen::<usize>() % available.len()].clone());
        }
        let scores: Vec<f64> = available.iter()
            .map(|id| self.scores.get(id).map(|s| s.composite_score()).unwrap_or(0.0)).collect();
        let total: f64 = scores.iter().sum();
        if total == 0.0 { return Some(available[rng.r#gen::<usize>() % available.len()].clone()); }
        let mut cum = 0.0;
        let target = rng.r#gen::<f64>() * total;
        for (i, s) in scores.iter().enumerate() { cum += s; if cum >= target { return Some(available[i].clone()); } }
        Some(available.last().unwrap().clone())
    }

    pub fn deposit_success(&mut self, relay: &str, latency_ms: f64) {
        if let Some(s) = self.scores.get_mut(relay) {
            s.success_count += 1;
            s.success_rate = s.success_count as f64 / (s.success_count + s.failure_count) as f64;
            s.latency_ms = 0.3 * latency_ms + 0.7 * s.latency_ms;
            s.last_updated = Instant::now();
            self.states.insert(relay.to_string(), NeighborState::Healthy);
        }
    }

    pub fn deposit_failure(&mut self, relay: &str) {
        if let Some(s) = self.scores.get_mut(relay) {
            s.failure_count += 1;
            s.success_rate = s.success_count as f64 / (s.success_count + s.failure_count) as f64;
            s.last_updated = Instant::now();
            if s.failure_count >= self.blacklist_threshold as u64 {
                self.states.insert(relay.to_string(), NeighborState::Blacklisted);
            }
        }
    }

    fn evict_weakest(&mut self) {
        let weakest = self.scores.iter()
            .map(|(id, s)| (id.clone(), s.composite_score()))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        if let Some((id, _)) = weakest { self.scores.remove(&id); self.states.remove(&id); }
    }
}
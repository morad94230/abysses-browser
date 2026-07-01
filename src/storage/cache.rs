use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Fragment {
    pub index: u8,
    pub data: Vec<u8>,
    pub hash: [u8; 32],
    pub root_hash: [u8; 32],
}

pub struct OrganicCache {
    pub fragments: HashMap<[u8; 32], Vec<Fragment>>,
    pub max_capacity_mb: usize,
    pub current_usage_mb: usize,
}

impl OrganicCache {
    pub fn new(max_capacity_mb: usize) -> Self {
        Self { fragments: HashMap::new(), max_capacity_mb, current_usage_mb: 0 }
    }

    pub fn has_all(&self, root_hash: &[u8; 32]) -> bool {
        self.fragments.get(root_hash).map(|f| f.len() >= 10).unwrap_or(false)
    }

    pub fn get(&self, root_hash: &[u8; 32]) -> Option<Vec<Fragment>> {
        self.fragments.get(root_hash).cloned()
    }

    pub fn store(&mut self, root_hash: [u8; 32], fragments: Vec<Fragment>) {
        let needed_mb = fragments.iter().map(|f| f.data.len()).sum::<usize>() / (1024 * 1024);
        while self.current_usage_mb + needed_mb > self.max_capacity_mb && !self.fragments.is_empty() {
            if let Some(oldest) = self.fragments.keys().next().cloned() {
                if let Some(removed) = self.fragments.remove(&oldest) {
                    let freed = removed.iter().map(|f| f.data.len()).sum::<usize>() / (1024 * 1024);
                    self.current_usage_mb = self.current_usage_mb.saturating_sub(freed);
                }
            }
        }
        self.current_usage_mb += needed_mb;
        self.fragments.insert(root_hash, fragments);
    }
}
use blake3::Hasher;

pub struct MerkleTree {
    pub leaves: Vec<[u8; 32]>,
    pub root: [u8; 32],
}

impl MerkleTree {
    pub fn new(leaves: &[[u8; 32]]) -> Self {
        let mut current: Vec<[u8; 32]> = leaves.to_vec();
        if current.len() % 2 == 1 {
            current.push(*current.last().unwrap());
        }
        while current.len() > 1 {
            let mut next = Vec::new();
            for i in (0..current.len()).step_by(2) {
                let left = current[i];
                let right = if i + 1 < current.len() {
                    current[i + 1]
                } else {
                    left
                };
                let mut hasher = Hasher::new();
                hasher.update(&left);
                hasher.update(&right);
                next.push(*hasher.finalize().as_bytes());
            }
            current = next;
        }
        Self {
            leaves: leaves.to_vec(),
            root: current[0],
        }
    }

    pub fn root(&self) -> [u8; 32] {
        self.root
    }
}

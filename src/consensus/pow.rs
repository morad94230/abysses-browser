use blake3::Hasher;

pub struct LightPow;

impl LightPow {
    pub fn verify(hash: &[u8; 32], nonce: u64, difficulty: u8) -> bool {
        let mut hasher = Hasher::new();
        hasher.update(hash);
        hasher.update(&nonce.to_le_bytes());
        let result = hasher.finalize();
        let bytes = result.as_bytes();
        for i in 0..(difficulty as usize / 8) {
            if bytes[i] != 0 {
                return false;
            }
        }
        if difficulty % 8 != 0 {
            let mask = 0xFFu8 << (8 - difficulty % 8);
            if bytes[difficulty as usize / 8] & mask != 0 {
                return false;
            }
        }
        true
    }

    pub fn mine(hash: &[u8; 32], difficulty: u8) -> u64 {
        let mut nonce = 0u64;
        loop {
            if Self::verify(hash, nonce, difficulty) {
                return nonce;
            }
            nonce += 1;
        }
    }
}

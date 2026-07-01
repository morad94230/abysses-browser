use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use x25519_dalek::{StaticSecret, PublicKey as X25519PublicKey};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct NodeIdentity {
    pub ed25519_keypair: Keypair,
    pub peer_id: String,
}

#[derive(Clone, Debug)]
pub struct EphemeralIdentity {
    pub x25519_secret: StaticSecret,
    pub x25519_public: X25519PublicKey,
    pub created_at: u64,
    pub suggested_expires_at: u64,
    pub active_circuits: usize,
}

#[derive(Debug, Clone)]
pub struct IdentityWallet {
    pub tab_id: u64,
    pub ephemeral: EphemeralIdentity,
    pub pseudonym: String,
}

impl NodeIdentity {
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let keypair = Keypair::generate(&mut csprng);
        let peer_id = hex::encode(keypair.public.to_bytes());
        Self {
            ed25519_keypair: keypair,
            peer_id,
        }
    }

    pub fn sign(&self, message: &[u8]) -> ed25519_dalek::Signature {
        self.ed25519_keypair.sign(message)
    }
}

impl EphemeralIdentity {
    pub fn generate() -> Self {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = X25519PublicKey::from(&secret);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            x25519_secret: secret,
            x25519_public: public,
            created_at: now,
            suggested_expires_at: now + 600,
            active_circuits: 0,
        }
    }

    pub fn can_expire(&self) -> bool {
        self.active_circuits == 0
    }

    pub fn prolong_if_needed(&mut self) {
        if self.active_circuits > 0 {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            self.suggested_expires_at = now + 600;
        }
    }
}
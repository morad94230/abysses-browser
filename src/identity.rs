use std::time::{SystemTime, UNIX_EPOCH};

use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

#[derive(Clone, Debug)]
pub struct NodeIdentity {
    pub ed25519_keypair: SigningKey,
    pub peer_id: String,
}

pub struct EphemeralIdentity {
    pub x25519_secret: StaticSecret,
    pub x25519_public: X25519PublicKey,
    pub created_at: u64,
    pub suggested_expires_at: u64,
    pub active_circuits: usize,
}

impl Clone for EphemeralIdentity {
    fn clone(&self) -> Self {
        Self {
            x25519_secret: StaticSecret::from(self.x25519_secret.to_bytes()),
            x25519_public: self.x25519_public,
            created_at: self.created_at,
            suggested_expires_at: self.suggested_expires_at,
            active_circuits: self.active_circuits,
        }
    }
}

impl std::fmt::Debug for EphemeralIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EphemeralIdentity")
            .field("x25519_public", &self.x25519_public)
            .field("created_at", &self.created_at)
            .field("active_circuits", &self.active_circuits)
            .finish()
    }
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
        let keypair = SigningKey::generate(&mut csprng);
        let vk = VerifyingKey::from(&keypair);
        let peer_id = hex::encode(vk.to_bytes());
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
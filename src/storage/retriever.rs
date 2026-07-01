use crate::storage::cache::OrganicCache;
use crate::storage::erasure::{Content, Fragmenter, DATA_SHARDS};
use ed25519_dalek::VerifyingKey;

pub struct SiteRetriever {
    pub cache: OrganicCache,
}

impl SiteRetriever {
    pub fn new(cache: OrganicCache) -> Self {
        Self { cache }
    }

    pub async fn retrieve(&mut self, root_hash: &[u8; 32]) -> Result<Content, String> {
        let fragments = self.cache.get(root_hash).ok_or("Not in cache")?;
        if fragments.len() < DATA_SHARDS {
            return Err("Not enough fragments".to_string());
        }
        // Conversion des fragments cache vers fragments erasure
        let era_fragments: Vec<crate::storage::erasure::Fragment> = fragments
            .iter()
            .map(|f| crate::storage::erasure::Fragment {
                index: f.index,
                data: f.data.clone(),
                hash: f.hash,
                root_hash: f.root_hash,
                ttl: 86400,
                signature: vec![],
                shard_index: f.index as usize,
            })
            .collect();
        // Pour le MVP, on ignore la vérification de signature
        let reconstructed = Fragmenter::reconstruct_fragments(&era_fragments)?;
        Ok(reconstructed)
    }
}

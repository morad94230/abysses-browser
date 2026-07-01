use crate::storage::erasure::{Fragmenter, Content, Fragment, DATA_SHARDS};
use crate::storage::cache::OrganicCache;

pub struct SiteRetriever {
    pub cache: OrganicCache,
}

impl SiteRetriever {
    pub fn new(cache: OrganicCache) -> Self {
        Self { cache }
    }

    pub async fn retrieve(&mut self, root_hash: &[u8; 32], key: &ed25519_dalek::PublicKey) -> Result<Content, String> {
        let fragments = self.cache.get(root_hash).ok_or("Not in cache")?;
        if fragments.len() < DATA_SHARDS {
            return Err("Not enough fragments".to_string());
        }
        let content = Fragmenter::reconstruct(&fragments, key).map_err(|e| format!("{:?}", e))?;
        Ok(content)
    }
}
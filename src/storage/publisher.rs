use crate::storage::erasure::{Fragmenter, Content, ContentMetadata};
use crate::storage::cache::OrganicCache;
use crate::identity::NodeIdentity;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct SitePublisher {
    pub identity: NodeIdentity,
    pub cache: OrganicCache,
}

impl SitePublisher {
    pub fn new(identity: NodeIdentity, cache: OrganicCache) -> Self {
        Self { identity, cache }
    }

    pub async fn publish(&mut self, name: &str, html: &[u8], desc: &str) -> Result<([u8; 32], usize), String> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let meta = ContentMetadata {
            name: name.to_string(),
            description: desc.to_string(),
            created_at: now,
            expires_at: now + 86400 * 30,
            owner: self.identity.peer_id.clone(),
            mime_type: "text/html".to_string(),
            tags: vec![],
            original_size: html.len(),
        };
        let content = Content { data: html.to_vec(), metadata: meta };
        let (fragments, root_hash) = Fragmenter::fragment(&content, &self.identity.ed25519_keypair).map_err(|e| e)?;
        let cache_fragments: Vec<crate::storage::cache::Fragment> = fragments.iter().map(|f| {
            crate::storage::cache::Fragment {
                index: f.index,
                data: f.data.clone(),
                hash: f.hash,
                root_hash: f.root_hash,
            }
        }).collect();
        self.cache.store(root_hash, cache_fragments);
        Ok((root_hash, fragments.len()))
    }
}
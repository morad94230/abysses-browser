use crate::search::index::{PageEntry, SearchIndex};
use rand::Rng;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

pub struct OrganicCrawler {
    pub index: Arc<Mutex<SearchIndex>>,
    pub running: bool,
    pub crawl_interval_seconds: u64,
}

impl OrganicCrawler {
    pub fn new(index: Arc<Mutex<SearchIndex>>) -> Self {
        Self {
            index,
            running: false,
            crawl_interval_seconds: 60,
        }
    }

    pub async fn run(&mut self) {
        self.running = true;
        let mut rng = rand::thread_rng();

        while self.running {
            let delay = rng.gen_range(30..120);
            sleep(Duration::from_secs(delay)).await;

            let mut index = self.index.lock().await;
            self.crawl_sites(&mut index).await;
        }
    }

    async fn crawl_sites(&self, index: &mut SearchIndex) {
        let entry = PageEntry {
            url: format!("site-{}.abyss", rand::random::<u16>()),
            title: "Abysses Site".to_string(),
            description: "A decentralized darkweb site".to_string(),
            keywords: vec![
                "abysses".to_string(),
                "darkweb".to_string(),
                "decentralized".to_string(),
            ],
            content_hash: [0u8; 32],
            last_crawled: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            trust_score: 0.5,
        };
        index.add_page(entry);
    }

    pub fn stop(&mut self) {
        self.running = false;
    }
}

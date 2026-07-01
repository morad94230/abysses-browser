use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PageEntry {
    pub url: String,
    pub title: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub content_hash: [u8; 32],
    pub last_crawled: u64,
    pub trust_score: f64,
}

pub struct SearchIndex {
    pub pages: HashMap<String, PageEntry>,
    pub keyword_index: HashMap<String, Vec<String>>,
}

impl SearchIndex {
    pub fn new() -> Self {
        Self {
            pages: HashMap::new(),
            keyword_index: HashMap::new(),
        }
    }

    pub fn add_page(&mut self, entry: PageEntry) {
        let url = entry.url.clone();
        for keyword in &entry.keywords {
            self.keyword_index
                .entry(keyword.clone())
                .or_insert_with(Vec::new)
                .push(url.clone());
        }
        self.pages.insert(url, entry);
    }

    pub fn search(&self, query: &str) -> Vec<&PageEntry> {
        let terms: Vec<&str> = query.split_whitespace().collect();
        let mut results: HashMap<&str, f64> = HashMap::new();

        for term in &terms {
            if let Some(urls) = self.keyword_index.get(*term) {
                for url in urls {
                    if let Some(page) = self.pages.get(url) {
                        let score = page.trust_score * 0.7 + 0.3;
                        *results.entry(url).or_insert(0.0) += score;
                    }
                }
            }
        }

        let mut sorted: Vec<(&str, f64)> = results.iter().map(|(k, v)| (*k, *v)).collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        sorted
            .iter()
            .filter_map(|(url, _)| self.pages.get(url))
            .collect()
    }
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Navigate { tab_id: u64, url: String },
    Publish { tab_id: u64, name: String, content: Vec<u8>, mime_type: String },
    Resolve { request_id: String, name: String },
    RotateIdentity { tab_id: u64 },
    GetTopology,
    Search { query: String, tab_id: u64 },
    Ping,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    NavigationResult {
        tab_id: u64,
        url: String,
        status: u16,
        content: Vec<u8>,
        mime_type: String,
    },
    NameResolved {
        request_id: String,
        name: String,
        root_hash: Option<String>,
        owner: Option<String>,
    },
    IdentityRotated {
        tab_id: u64,
        pseudonym: String,
        public_key: String,
    },
    TopologyUpdate {
        nodes: Vec<TopologyNode>,
        edges: Vec<TopologyEdge>,
    },
    SearchResult {
        tab_id: u64,
        results: Vec<SearchResultItem>,
    },
    Error { code: String, message: String },
    Pong,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TopologyNode {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub size: f64,
    pub color: String,
    pub label: String,
    pub node_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TopologyEdge {
    pub from: String,
    pub to: String,
    pub strength: f64,
    pub color: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResultItem {
    pub url: String,
    pub title: String,
    pub description: String,
    pub trust_score: f64,
}
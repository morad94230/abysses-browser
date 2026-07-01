mod api;
mod circuit;
mod consensus;
mod cover_traffic;
mod error;
mod identity;
mod protocol;
mod search;
mod simulator;
mod storage;
mod types;

use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::{error, info};

pub struct AbyssNode {
    pub config: types::NodeConfig,
    pub identity: identity::NodeIdentity,
    pub pheromone_table: Arc<Mutex<protocol::pheromone::PheromoneTable>>,
    pub circuit_builder: circuit::builder::CircuitBuilder,
    pub search_index: Arc<Mutex<search::index::SearchIndex>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("🌊 Abysses Node v0.1.0 starting...");

    let config = types::NodeConfig::default();
    let node_identity = identity::NodeIdentity::generate();
    let pheromone_table = Arc::new(Mutex::new(protocol::pheromone::PheromoneTable::default()));
    let circuit_builder = circuit::builder::CircuitBuilder::new(pheromone_table.clone());
    let search_index = Arc::new(Mutex::new(search::index::SearchIndex::new()));

    let mut crawler = search::crawler::OrganicCrawler::new(search_index.clone());
    tokio::spawn(async move {
        crawler.run().await;
    });

    let node = Arc::new(AbyssNode {
        config: config.clone(),
        identity: node_identity,
        pheromone_table,
        circuit_builder,
        search_index,
    });

    info!("WebSocket: ws://127.0.0.1:{}", config.websocket_port);
    info!("Proxy: http://127.0.0.1:{}", config.proxy_port);
    info!("Search engine: active");
    info!("Node online — you are the network");

    let node_proxy = node.clone();
    let node_ws = node.clone();

    tokio::spawn(async move {
        if let Err(e) = api::proxy::start_proxy(node_proxy.config.proxy_port).await {
            error!("Proxy error: {}", e);
        }
    });

    api::ws_server::start_websocket_with_search(node_ws).await?;

    Ok(())
}

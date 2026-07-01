mod error;
mod types;
mod identity;
mod protocol;
mod circuit;
mod storage;
mod consensus;
mod api;
mod cover_traffic;
mod simulator;

use tracing::info;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("🌊 Abysses Node v0.1.0 starting...");

    let config = types::NodeConfig::default();
    let _node_identity = identity::NodeIdentity::generate();
    let pheromone_table = Arc::new(Mutex::new(protocol::pheromone::PheromoneTable::default()));
    let _circuit_builder = circuit::builder::CircuitBuilder::new(pheromone_table);

    info!("WebSocket: ws://127.0.0.1:{}", config.websocket_port);
    info!("Proxy: http://127.0.0.1:{}", config.proxy_port);
    info!("Node online — you are the network");

    api::proxy::start_proxy(config.proxy_port).await?;

    Ok(())
}
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

mod api;
mod circuit;
mod consensus;
mod cover_traffic;
mod error;
mod identity;
mod network;
mod protocol;
mod search;
mod simulator;
mod storage;
mod types;

use crate::error::AbyssError;
use crate::types::{NodeConfig, NodeState};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Abysses Browser node...");

    // Create node configuration
    let config = NodeConfig::default();
    info!("Node configuration loaded");

    // Initialize node state
    let node_state = Arc::new(RwLock::new(NodeState::new(&config)));
    info!("Node state initialized");

    // Initialize identity
    let identity = identity::NodeIdentity::new();
    info!("Node identity: {}", hex::encode(&identity.public_key()[..8]));

    // Initialize storage
    let cache = storage::cache::Cache::new(1000);
    info!("Storage cache initialized");

    // Initialize circuit manager
    let circuit_manager = circuit::manager::CircuitManager::new();
    info!("Circuit manager initialized");

    // Initialize pheromone tables
    let pheromone_tables = protocol::pheromone::PheromoneTables::new();
    info!("Pheromone tables initialized");

    // Initialize consensus chain
    let chain = consensus::chain::Chain::new();
    info!("Consensus chain initialized");

    // Initialize search engine
    let search_engine = search::index::SearchIndex::new();
    info!("Search engine initialized");

    // Start API server
    info!("Starting API server on 127.0.0.1:9000...");
    let api_handle = tokio::spawn(async {
        if let Err(e) = api::ws_server::start_websocket_server().await {
            error!("WebSocket server error: {}", e);
        }
    });

    // Start proxy
    info!("Starting HTTP proxy on 127.0.0.1:9001...");
    let proxy_handle = tokio::spawn(async {
        if let Err(e) = api::proxy::start_proxy().await {
            error!("Proxy server error: {}", e);
        }
    });

    // Start cover traffic generator
    info!("Starting cover traffic generator...");
    let cover_traffic_handle = tokio::spawn(async {
        cover_traffic::generate_cover_traffic().await;
    });

    info!("Abysses Browser node running. Press Ctrl+C to stop.");

    // Wait for shutdown
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received");

    Ok(())
}

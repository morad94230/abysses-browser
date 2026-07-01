use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{error, info};

use crate::api::websocket::{ClientMessage, ServerMessage, SearchResultItem};
use crate::search::index::SearchIndex;
use crate::AbyssNode;

pub async fn start_websocket_with_search(
    node: Arc<AbyssNode>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("127.0.0.1:{}", node.config.websocket_port);
    let listener = TcpListener::bind(&addr).await?;
    info!("📡 WebSocket server listening on ws://{}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let node = node.clone();
        tokio::spawn(handle_connection(stream, node));
    }

    Ok(())
}

async fn handle_connection(stream: TcpStream, node: Arc<AbyssNode>) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("WebSocket handshake failed: {}", e);
            return;
        }
    };

    info!("WebSocket client connected");
    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    let response = handle_message(client_msg, &node).await;
                    let json = serde_json::to_string(&response).unwrap();
                    if let Err(e) = write.send(Message::Text(json)).await {
                        error!("WebSocket send error: {}", e);
                        break;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket client disconnected");
                break;
            }
            Ok(Message::Ping(data)) => {
                let _ = write.send(Message::Pong(data)).await;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

async fn handle_message(msg: ClientMessage, node: &AbyssNode) -> ServerMessage {
    match msg {
        ClientMessage::Search { query, tab_id } => {
            info!("Search request: '{}'", query);
            let index = node.search_index.lock().await;
            let results = index.search(&query);
            let items: Vec<SearchResultItem> = results
                .iter()
                .map(|p| SearchResultItem {
                    url: p.url.clone(),
                    title: p.title.clone(),
                    description: p.description.clone(),
                    trust_score: p.trust_score,
                })
                .collect();
            ServerMessage::SearchResult { tab_id, results: items }
        }
        // Conserver les autres messages (simplifiés ici, on peut les rajouter plus tard)
        _ => ServerMessage::Error {
            code: "NOT_IMPLEMENTED".into(),
            message: "Feature not yet available via WebSocket".into(),
        },
    }
}
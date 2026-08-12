use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{error, info};

use crate::api::websocket::{ClientMessage, SearchResultItem, ServerMessage};
use crate::search::index::SearchIndex;

pub async fn start_websocket_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:9000";
    let listener = TcpListener::bind(&addr).await?;
    info!("WebSocket server listening on ws://{}", addr);

    let search_index = Arc::new(Mutex::new(SearchIndex::new()));

    while let Ok((stream, _)) = listener.accept().await {
        let search_index = search_index.clone();
        tokio::spawn(handle_connection(stream, search_index));
    }

    Ok(())
}

async fn handle_connection(
    stream: TcpStream,
    search_index: Arc<Mutex<SearchIndex>>,
) {
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
                    let response = handle_message(client_msg, &search_index).await;
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

async fn handle_message(
    msg: ClientMessage,
    search_index: &Arc<Mutex<SearchIndex>>,
) -> ServerMessage {
    match msg {
        ClientMessage::Search { query, tab_id } => {
            info!("Search request: '{}'", query);
            let index = search_index.lock().await;
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
            ServerMessage::SearchResult {
                tab_id,
                results: items,
            }
        }
        _ => ServerMessage::Error {
            code: "NOT_IMPLEMENTED".into(),
            message: "Feature not yet available via WebSocket".into(),
        },
    }
}

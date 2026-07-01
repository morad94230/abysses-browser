use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeState {
    Initializing,
    Bootstrapping,
    Online,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub websocket_port: u16,
    pub proxy_port: u16,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            websocket_port: 9000,
            proxy_port: 8080,
        }
    }
}

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AbyssError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("Decryption error: {0}")]
    Decryption(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("No active circuit")]
    NoActiveCircuit,
    #[error("Insufficient neighbors: {0}")]
    InsufficientNeighbors(usize),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Consensus error: {0}")]
    ConsensusError(String),
}

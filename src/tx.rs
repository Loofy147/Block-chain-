use serde::{Serialize, Deserialize};

/// Represents a transaction in the blockchain.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    /// The sender's public key.
    pub from: Vec<u8>,
    /// The recipient's public key.
    pub to: Vec<u8>,
    /// The amount to transfer.
    pub amount: u64,
    /// A sequence number to prevent replay attacks.
    pub nonce: u64,
    /// The cryptographic signature of the transaction.
    /// Stored as raw bytes for simplicity in this MVP.
    pub signature: Vec<u8>,
}

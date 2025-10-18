use serde::{Serialize, Deserialize};
use crate::tx::Transaction;

/// Represents the header of a block.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BlockHeader {
    /// The hash of the parent block.
    pub parent_hash: Vec<u8>,
    /// The Merkle root of the transactions in the block.
    pub merkle_root: Vec<u8>,
    /// The timestamp of when the block was created.
    pub timestamp: u64,
    /// A random nonce, typically used in Proof-of-Work.
    pub nonce: u64,
    /// The public key of the block proposer.
    pub proposer: Vec<u8>,
}

/// Represents a block in the blockchain.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// The block header.
    pub header: BlockHeader,
    /// The list of transactions included in the block.
    pub txs: Vec<Transaction>,
    /// The cached hash of the block.
    pub hash: Vec<u8>,
}

use anyhow::Result;
use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sled::Db;
use std::time::Duration;
use std::{thread, vec};

// -----------------------------
// Types
// -----------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeader {
    pub parent_hash: String,
    pub merkle_root: String,
    pub timestamp: i64,
    pub nonce: u64,
    pub difficulty: u32,
    pub miner: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub txs: Vec<Transaction>,
    pub hash: String,
}

// -----------------------------
// Helpers
// -----------------------------

fn hash_block_header(h: &BlockHeader) -> String {
    let serialized = serde_json::to_string(h).expect("serialize header");
    let mut hasher = Sha256::new();
    hasher.update(serialized.as_bytes());
    hex::encode(hasher.finalize())
}

fn hash_block(block: &Block) -> String {
    let serialized = serde_json::to_string(block).expect("serialize block");
    let mut hasher = Sha256::new();
    hasher.update(serialized.as_bytes());
    hex::encode(hasher.finalize())
}

fn merkle_root(txs: &Vec<Transaction>) -> String {
    // very simple merkle-like: hash concatenation pairs until single hash
    if txs.is_empty() {
        return "".to_string();
    }
    let mut leaves: Vec<String> = txs
        .iter()
        .map(|t| {
            let s = serde_json::to_string(t).unwrap();
            let mut h = Sha256::new();
            h.update(s.as_bytes());
            hex::encode(h.finalize())
        })
        .collect();

    while leaves.len() > 1 {
        let mut next = Vec::new();
        for pair in leaves.chunks(2) {
            if pair.len() == 1 {
                next.push(pair[0].clone());
            } else {
                let mut h = Sha256::new();
                h.update(pair[0].as_bytes());
                h.update(pair[1].as_bytes());
                next.push(hex::encode(h.finalize()));
            }
        }
        leaves = next;
    }
    leaves[0].clone()
}

// -----------------------------
// Storage wrapper (sled)
// -----------------------------

struct ChainDB {
    db: Db,
}

impl ChainDB {
    fn open(path: &str) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    fn save_block(&self, block: &Block) -> Result<()> {
        let key = block.hash.clone();
        let val = serde_json::to_vec(block)?;
        self.db.insert(key.as_bytes(), val)?;
        // store latest height reference
        self.db.insert(b"latest", block.hash.as_bytes())?;
        self.db.flush()?;
        Ok(())
    }

    fn get_latest(&self) -> Result<Option<Block>> {
        if let Some(v) = self.db.get(b"latest")? {
            let hash = String::from_utf8(v.to_vec())?;
            if let Some(bv) = self.db.get(hash.as_bytes())? {
                let block: Block = serde_json::from_slice(&bv)?;
                return Ok(Some(block));
            }
        }
        Ok(None)
    }
}

// -----------------------------
// Mining / PoW
// -----------------------------

fn meets_difficulty(hex_hash: &str, difficulty: u32) -> bool {
    // simple target: count leading zero nibbles (hex characters)
    let needed = (difficulty as usize) / 4; // approximate
    hex_hash.starts_with(&"0".repeat(needed))
}

fn mine_block(header_template: &BlockHeader, txs: &Vec<Transaction>) -> Block {
    let mut header = header_template.clone();
    loop {
        header.nonce = rand::thread_rng().gen();
        let mut header_for_hash = header.clone();
        // recompute merkle root in case txs changed
        header_for_hash.merkle_root = merkle_root(txs);
        let h = hash_block_header(&header_for_hash);
        if meets_difficulty(&h, header_for_hash.difficulty) {
            let block = Block {
                header: header_for_hash,
                txs: txs.clone(),
                hash: h.clone(),
            };
            return block;
        }
    }
}

// -----------------------------
// Application (MVP node)
// -----------------------------

fn make_genesis(difficulty: u32) -> Block {
    let header = BlockHeader {
        parent_hash: String::from("0"),
        merkle_root: String::from(""),
        timestamp: Utc::now().timestamp(),
        nonce: 0,
        difficulty,
        miner: String::from("genesis"),
    };
    let txs: Vec<Transaction> = vec![];
    let mut header_for_hash = header.clone();
    header_for_hash.merkle_root = merkle_root(&txs);
    let h = hash_block_header(&header_for_hash);
    Block {
        header: header_for_hash,
        txs,
        hash: h,
    }
}

fn main() -> Result<()> {
    println!("PoW MVP node (single-process).\nStarting...");

    // Open DB
    let chain_db = ChainDB::open("./chain_db")?;

    // Difficulty: number of leading zero bits approximation (use small for testing)
    let difficulty: u32 = 12; // tune this: higher -> slower

    // If no chain, write genesis
    if chain_db.get_latest()?.is_none() {
        let genesis = make_genesis(difficulty);
        chain_db.save_block(&genesis)?;
        println!("Saved genesis: {}", genesis.hash);
    }

    // Simple mempool of random txs for demo
    let mut mempool: Vec<Transaction> = Vec::new();

    // Spawn a miner loop (single-threaded for MVP)
    loop {
        // populate mempool with a few random txs
        for _ in 0..3 {
            let t = Transaction {
                from: format!("addr_{}", rand::thread_rng().gen::<u16>()),
                to: format!("addr_{}", rand::thread_rng().gen::<u16>()),
                amount: rand::thread_rng().gen_range(1..1000),
            };
            mempool.push(t);
        }

        // get parent
        let parent = chain_db.get_latest()?.expect("latest exists");
        let header = BlockHeader {
            parent_hash: parent.hash.clone(),
            merkle_root: String::new(),
            timestamp: Utc::now().timestamp(),
            nonce: 0,
            difficulty,
            miner: String::from("miner_1"),
        };

        println!("Mining new block on parent {}...", &parent.hash[..8]);
        let block = mine_block(&header, &mempool);
        let block_hash = block.hash.clone();
        chain_db.save_block(&block)?;
        println!("Mined block {} with {} txs (nonce={})", &block_hash[..12], block.txs.len(), block.header.nonce);

        // clear mempool
        mempool.clear();

        // wait a bit to avoid busy loop in demo (adjust as needed)
        thread::sleep(Duration::from_secs(1));
    }
}

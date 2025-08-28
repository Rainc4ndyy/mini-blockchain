use crate::transaction::Transaction;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub difficulty: usize,
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let tx_list = self
            .transactions
            .iter()
            .map(|tx| tx.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        write!(
            f,
            "Block #{}\n----------------\nTimestamp: {}\nDifficulty: {}\nTransactions:\n{}\n\nPrev. Hash: {}...\n      Hash: {}...\n     Nonce: {}\n",
            self.index,
            DateTime::from_timestamp(self.timestamp, 0)
                .map(|dt| dt.to_rfc2822())
                .unwrap_or_default(),
            self.difficulty,
            tx_list,
            &self.previous_hash[..10],
            &self.hash[..10],
            self.nonce
        )
    }
}

impl Block {
    pub fn new(
        index: u64,
        transactions: Vec<Transaction>,
        previous_hash: String,
        difficulty: usize,
    ) -> Self {
        Block {
            index,
            timestamp: Utc::now().timestamp(),
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
            difficulty,
        }
    }

    pub fn mine(&mut self) {
        let prefix = "0".repeat(self.difficulty);
        loop {
            let hash_data = self.prepare_hash_data();
            let mut hasher = Sha256::new();
            hasher.update(hash_data);
            let new_hash = format!("{:x}", hasher.finalize());

            if new_hash.starts_with(&prefix) {
                self.hash = new_hash;
                return;
            }
            self.nonce += 1;
        }
    }

    fn prepare_hash_data(&self) -> String {
        serde_json::to_string(&(
            &self.index,
            &self.timestamp,
            &self.transactions,
            &self.previous_hash,
            &self.nonce,
            &self.difficulty,
        ))
        .unwrap()
    }
}
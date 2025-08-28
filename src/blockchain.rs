use crate::block::Block;
use crate::transaction::{PublicKey, Transaction};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

const MINING_REWARD: u64 = 100;
const DIFFICULTY_ADJUSTMENT_INTERVAL: u64 = 10;
const TARGET_BLOCK_TIME_SECS: i64 = 30;

#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub mempool: Vec<Transaction>,
    pub difficulty: usize,
}

impl Blockchain {
    pub fn new() -> Result<Self> {
        let mut genesis_block = Block::new(0, vec![], "0".to_string(), 2);
        genesis_block.mine();

        Ok(Blockchain {
            chain: vec![genesis_block],
            mempool: vec![],
            difficulty: 2,
        })
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<()> {
        if !transaction.is_valid() {
            bail!("Transaction has a bad signature. It's probably fraudulent.");
        }
        self.mempool.push(transaction);
        Ok(())
    }

    pub fn mine_pending_transactions(&mut self, miner_address: PublicKey) -> Result<()> {
        if self.mempool.is_empty() {
            println!("[INFO] Mempool is empty. Mining a block with only the reward transaction.");
        }

        let reward_tx = Transaction::new_coinbase(miner_address, MINING_REWARD);

        let mut transactions_for_block = self.mempool.clone();
        transactions_for_block.insert(0, reward_tx);

        self.adjust_difficulty();

        let previous_hash = self.chain.last().unwrap().hash.clone();
        let mut new_block = Block::new(
            self.chain.len() as u64,
            transactions_for_block,
            previous_hash,
            self.difficulty,
        );

        println!("[INFO] Starting Proof-of-Work for new block...");
        new_block.mine();

        self.chain.push(new_block);
        self.mempool.clear();
        Ok(())
    }

    pub fn get_balance(&self, address: &PublicKey) -> i64 {
        let mut balance = 0i64;
        for block in &self.chain {
            for tx in &block.transactions {
                if tx.destination == *address {
                    balance += tx.amount as i64;
                }
                if let Some(source) = &tx.source {
                    if *source == *address {
                        balance -= tx.amount as i64;
                    }
                }
            }
        }
        balance
    }

    fn adjust_difficulty(&mut self) {
        let latest_block = self.chain.last().unwrap();
        if latest_block.index > 0 && latest_block.index % DIFFICULTY_ADJUSTMENT_INTERVAL == 0 {
            let interval_start_block =
                &self.chain[(latest_block.index - DIFFICULTY_ADJUSTMENT_INTERVAL) as usize];
            let time_taken = latest_block.timestamp - interval_start_block.timestamp;
            let expected_time = (DIFFICULTY_ADJUSTMENT_INTERVAL as i64) * TARGET_BLOCK_TIME_SECS;

            if time_taken < expected_time / 2 {
                self.difficulty += 1;
                println!(
                    "[INFO] Mining is getting too fast. Increasing difficulty to {}.",
                    self.difficulty
                );
            } else if time_taken > expected_time * 2 && self.difficulty > 1 {
                self.difficulty -= 1;
                println!(
                    "[INFO] Mining is too slow. Decreasing difficulty to {}.",
                    self.difficulty
                );
            }
        }
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];
            if current_block.previous_hash != previous_block.hash {
                return false;
            }
            for tx in &current_block.transactions {
                if !tx.is_valid() {
                    return false;
                }
            }
        }
        true
    }
}
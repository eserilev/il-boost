use alloy::{primitives::B256, rpc::types::Block};
use types::Transaction;

pub mod types;

pub fn get_censored_transactions(transactions: Vec<Transaction>, block: &Block<alloy::rpc::types::Transaction>) -> Vec<B256> {

    let mut censored_transactions = vec![];
    let mut gas_left = block.header.gas_limit - block.header.gas_used;

    for tx in transactions {
        if let Some(max_priority_fee_per_gas) = tx.max_priority_fee_per_gas {
            if max_priority_fee_per_gas > 0 && gas_left > 0 {
                gas_left -= tx.gas;
                censored_transactions.push(tx.tx_hash);
            }
        }
    }

    censored_transactions

}
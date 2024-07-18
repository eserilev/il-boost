use std::sync::Arc;

use alloy::{network::TransactionResponse, primitives::B256};
use reth_transaction_pool::{test_utils::MockTransaction, ValidPoolTransaction};
use serde::{Deserialize, Serialize};
use ssz_types::{FixedVector, VariableList};
use ssz_types::typenum::U32;
use tree_hash_derive::TreeHash;

type MaxInclusionListLength = U32;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, TreeHash)]
pub struct InclusionList {
    pub slot: u64,
    pub validator_index: usize,
    pub transactions: FixedVector<B256, MaxInclusionListLength>,
}

impl InclusionList {
    pub fn new(slot: u64,  validator_index: usize, transactions: Vec<B256>) -> Self {   
        Self {
            slot,
            validator_index,
            transactions: transactions.into()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InclusionProof {
    transaction_hashes: VariableList<B256, MaxInclusionListLength>,
    generalized_indices: VariableList<u64, MaxInclusionListLength>,
    merkle_hashes: Vec<B256>,
}

impl InclusionProof {
    pub fn verify(&self) -> bool {
        // TODO
        true
    }
}

pub struct Transaction {
    pub is_eip4844: bool,
    pub gas_limit: u128,
    pub gas: u128,
    pub max_priority_fee_per_gas: Option<u128>,
    pub tx_hash: B256,
}

impl From<Arc<ValidPoolTransaction<MockTransaction>>> for Transaction {
    fn from(value: Arc<ValidPoolTransaction<MockTransaction>>) -> Self {
        Self {
            tx_hash: value.hash().clone(),
            is_eip4844: value.is_eip4844(),
            gas: value.gas_limit().into(),
            gas_limit: value.gas_limit().into(),
            max_priority_fee_per_gas: Some(value.priority_fee_or_price())
        }
    }
}

impl From<alloy::rpc::types::Transaction> for Transaction {
    fn from(value: alloy::rpc::types::Transaction) -> Self {
        Self {
            tx_hash: value.tx_hash(),
            is_eip4844: false,
            gas: value.gas,
            gas_limit: value.gas,
            max_priority_fee_per_gas: value.max_priority_fee_per_gas
        }
    }
}
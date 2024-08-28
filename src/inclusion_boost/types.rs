use std::collections::HashMap;
use std::sync::Arc;

use alloy::hex::ToHexExt;
use alloy::primitives::{keccak256, Bytes};
use alloy::rpc::types::beacon::{BlsPublicKey, BlsSignature};
use alloy::{network::TransactionResponse, primitives::B256};
use ethereum_consensus::ssz::prelude::List;
use parking_lot::RwLock;
use reth_transaction_pool::PoolTransaction;
use reth_transaction_pool::{test_utils::MockTransaction, ValidPoolTransaction};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use ssz_types::typenum::{U1, U32};
use ssz_types::{FixedVector, VariableList};
use tree_hash_derive::TreeHash;

/// The BLS Domain Separator used in Ethereum 2.0.

type MaxInclusionListLength = U1;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct InclusionListDelegateSignedMessage {
  pub message: InclusionListDelegateMessage,
  pub signature: BlsSignature
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, TreeHash)]
pub struct InclusionListDelegateMessage {
  pub preconfer_pubkey: BlsPublicKey,
  pub slot_number: u64,
  pub chain_id: u64,
  pub gas_limit: u64
}


#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, TreeHash)]
pub struct InclusionList {
    pub slot: u64,
    pub validator_index: usize,
    pub constraints: FixedVector<FixedVector<Constraint, MaxInclusionListLength>, MaxInclusionListLength>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, TreeHash)]
pub struct Constraint {
    pub tx: [u8; 32],
}

impl Serialize for Constraint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut item = serializer.serialize_struct("Constraint", 1)?;
        item.serialize_field("tx", &self.tx.encode_hex())?;
        item.end()
    }
}

impl InclusionList {
    pub fn new(slot: u64, validator_index: usize, transactions: Vec<Constraint>) -> Self {
       
        let list_of_lists = vec![transactions.into()];

        Self {
            slot,
            validator_index,
            constraints: list_of_lists.into(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct InclusionRequest {
    pub message: InclusionList,
    pub signature: BlsSignature,
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

#[derive(Debug)]
pub struct Transaction {
    pub is_eip4844: bool,
    pub gas_limit: u128,
    pub gas: u128,
    pub max_priority_fee_per_gas: Option<u128>,
    pub tx_hash: B256,
    pub bytes: Bytes,
    pub index: Option<u64>,
}

impl From<Arc<ValidPoolTransaction<MockTransaction>>> for Transaction {
    fn from(value: Arc<ValidPoolTransaction<MockTransaction>>) -> Self {
        Self {
            tx_hash: value.hash().clone(),
            is_eip4844: value.is_eip4844(),
            gas: value.gas_limit().into(),
            gas_limit: value.gas_limit().into(),
            max_priority_fee_per_gas: Some(value.priority_fee_or_price()),
            bytes: value.transaction.get_input().into(),
            index: None,
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
            max_priority_fee_per_gas: value.max_priority_fee_per_gas,
            bytes: value.input,
            index: value.transaction_index,
        }
    }
}

pub struct InclusionBoostCache {
    pub block_cache: Arc<RwLock<HashMap<u64, Vec<B256>>>>,
    pub inclusion_list_cache: Arc<RwLock<HashMap<u64, InclusionList>>>,
}

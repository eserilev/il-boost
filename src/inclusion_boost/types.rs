use std::collections::HashMap;
use std::sync::Arc;

use alloy::consensus::{SignableTransaction, TxEip1559, TxEnvelope, TxLegacy};
use alloy::primitives::Signature;
use alloy::rpc::types::beacon::{BlsPublicKey, BlsSignature};
use alloy::rpc::types::AccessList;
use alloy::{network::TransactionResponse, primitives::B256};
use parking_lot::RwLock;
use reth_transaction_pool::{test_utils::MockTransaction, ValidPoolTransaction};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use serde_utils::hex;
use ssz_types::typenum::{U1, U1000, U4, U512};
use ssz_types::{FixedVector, VariableList};
use tree_hash::TreeHash;
use tree_hash_derive::TreeHash;

/// The BLS Domain Separator used in Ethereum 2.0.

type MaxInclusionListLength = U1;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct InclusionListDelegateSignedMessage {
    pub message: InclusionListDelegateMessage,
    pub signature: BlsSignature,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, TreeHash)]
pub struct InclusionListDelegateMessage {
    pub preconfer_pubkey: BlsPublicKey,
    pub slot_number: u64,
    pub chain_id: u64,
    pub gas_limit: u64,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, TreeHash)]
pub struct InclusionList {
    pub slot: u64,
    pub validator_index: usize,
    pub constraints:
        FixedVector<FixedVector<Constraint, MaxInclusionListLength>, MaxInclusionListLength>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct Constraint {
    pub payload: ssz_types::VariableList<u8, U1000>,
    pub hash: tree_hash::Hash256,
}

impl TreeHash for Constraint {
    fn tree_hash_type() -> tree_hash::TreeHashType {
        tree_hash::TreeHashType::List
    }

    fn tree_hash_packed_encoding(&self) -> tree_hash::PackedEncoding {
        unreachable!("Should not be packed")
    }

    fn tree_hash_packing_factor() -> usize {
        unreachable!("Should not be packed")
    }

    fn tree_hash_root(&self) -> tree_hash::Hash256 {
        let root = self.payload.tree_hash_root();
        tree_hash::mix_in_length(&root, self.payload.len())
    }
}

impl Serialize for Constraint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut item = serializer.serialize_struct("Constraint", 1)?;
        item.serialize_field("tx", &hex::encode(self.payload.to_vec().clone()))?;
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
    pub envelope: TxEnvelope,
}

impl From<Arc<ValidPoolTransaction<MockTransaction>>> for Transaction {
    fn from(value: Arc<ValidPoolTransaction<MockTransaction>>) -> Self {
        todo!()
    }
}

impl From<alloy::rpc::types::Transaction> for Transaction {
    fn from(value: alloy::rpc::types::Transaction) -> Self {
        let mempool_signature = value.signature.expect("Signature must be set");
        let parity: bool = mempool_signature.y_parity.map(|p| p.0).unwrap_or(false);

        let signature = Signature::from_scalars_and_parity(
            mempool_signature.r.into(),
            mempool_signature.s.into(),
            parity,
        )
        .expect("Invalid tx signature in mempool");

        // For simplicity, support only eip-1559 and legacy txes
        let envelope = match value.gas_price {
            Some(price) => {
                // legacy tx
                let tx = TxLegacy {
                    chain_id: value.chain_id,
                    nonce: value.nonce,
                    gas_price: price,
                    gas_limit: value.gas(),
                    to: value.to().expect("Transaction to field must be set").into(),
                    value: value.value,
                    input: value.input,
                };

                TxEnvelope::Legacy(tx.into_signed(signature))
            }
            None => {
                // eip-1559 tx
                let tx = TxEip1559 {
                    chain_id: value.chain_id.expect("EIP-1559 tx should have chain id"),
                    nonce: value.nonce,
                    gas_limit: value.gas(),
                    max_fee_per_gas: value
                        .max_fee_per_gas
                        .expect("EIP-1559 tx must have max fee per gas"),
                    max_priority_fee_per_gas: value
                        .max_priority_fee_per_gas
                        .expect("EIP-1559 tx must have max priority fee per gas"),
                    to: value.to().expect("Transaction to field must be set").into(),
                    value: value.value,
                    // do not support access lists for simplicity
                    access_list: AccessList(vec![]),
                    input: value.input,
                };
                TxEnvelope::Eip1559(tx.into_signed(signature))
            }
        };

        Self { envelope }
    }
}

pub struct InclusionBoostCache {
    pub block_cache: Arc<RwLock<HashMap<u64, Vec<B256>>>>,
    pub inclusion_list_cache: Arc<RwLock<HashMap<u64, InclusionList>>>,
}

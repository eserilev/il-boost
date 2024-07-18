use alloy::primitives::{ B256, U256};
use ethereum_consensus::crypto::{PublicKey, Signature};
use ethereum_consensus::deneb::mainnet::{BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES};
use ethereum_consensus::deneb::ExecutionPayloadHeader;
use serde::{Deserialize, Serialize};
use ssz_types::typenum::U32;
use ssz_types::{FixedVector, VariableList};
use tree_hash_derive::TreeHash;

type MaxInclusionListLength = U32;


#[derive(Debug, Serialize, Deserialize)]
pub struct SignedExecutionPayloadHeaderWithProof {
    header: SignedExecutionPayloadHeader,
    proof: InclusionProof,
}


#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SignedExecutionPayloadHeader {
    pub message: BuilderBid,
    pub signature: Signature,
}


#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BuilderBid {
    header: ExecutionPayloadHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    pub value: U256,
    pub pubkey: PublicKey,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct InclusionProof {
    transaction_hashes: VariableList<B256, MaxInclusionListLength>,
    generalized_indices: VariableList<u64, MaxInclusionListLength>,
    merkle_hashes: Vec<B256>,
}
use alloy::primitives::{FixedBytes, B256, U256};
use ethereum_consensus::crypto::{PublicKey, Signature};
use ethereum_consensus::deneb::mainnet::{BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES};
use ethereum_consensus::deneb::ExecutionPayloadHeader;
use serde::{Deserialize, Serialize};
use tree_hash::Hash256;
use tree_hash_derive::TreeHash;
use tree_hash::TreeHash;
use ssz_types::VariableList;
use ssz_types::typenum::U32;
use merkle_proof::MerkleTree;


type MaxInclusionListLength = U32;

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize, TreeHash)]
pub struct InclusionList {
    pub slot: u64,
    pub validator_index: u64,
    pub transaction: B256
}

pub struct SignedExecutionPayloadHeaderWithProof {
    header: SignedExecutionPayloadHeader,
    proof: InclusionProof
}

pub struct SignedExecutionPayloadHeader {
    pub message: BuilderBid,
    pub signature: Signature,
}

pub struct BuilderBid {
    header: ExecutionPayloadHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    pub value: U256,
    pub pubkey: PublicKey,
}

pub struct InclusionProof {
  transaction_hashes: VariableList<B256, MaxInclusionListLength>,
  generalized_indices: VariableList<u64, MaxInclusionListLength>,
  merkle_hashes: MerkleTree,
}
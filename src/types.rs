use alloy::{primitives::Address, rpc::types::beacon::{payload::ExecutionPayloadHeader, BlsSignature}};
use serde::{Deserialize, Serialize};
use tree_hash_derive::TreeHash;

pub struct ExecutionPayloadHeaderWithProof {
    pub header: ExecutionPayloadHeader,
    pub proof: InclusionProof
}

pub struct SignedBeaconBlock {
    pub beacon_block: BeaconBlock,
    pub signature: BlsSignature
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize, TreeHash)]
pub struct BeaconBlock {

}

pub struct InclusionProof {
}

pub struct InclusionListSummaryEntry {
    pub address: Address,
    pub gas_limit: u64
}
pub struct InclusionListSummary {
    pub slot: u64,
    pub proposer_index: u64,
    pub summary: Vec<InclusionListSummaryEntry>
}
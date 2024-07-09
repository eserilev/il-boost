use alloy::{primitives::Address, rpc::types::beacon::{payload::ExecutionPayloadHeader, BlsSignature}};
use serde::{Deserialize, Serialize};
use tree_hash::Hash256;
use tree_hash_derive::TreeHash;
use ethereum_consensus::deneb::Transaction;

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

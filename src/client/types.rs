use alloy::primitives::B256;
use serde::{Deserialize, Serialize};
use tree_hash::Hash256;
use tree_hash_derive::TreeHash;
use tree_hash::TreeHash;
use ssz_types::VariableList;
use ssz_types::typenum::U32;

type MaxInclusionListLength = U32;

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize, TreeHash)]
pub struct InclusionList {
    pub slot: u64,
    pub validator_index: u64,
    pub transaction: B256
}
use alloy::rpc::types::beacon::BlsPublicKey;
use cb_common::pbs::RelayEntry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct InclusionListConfig {
}
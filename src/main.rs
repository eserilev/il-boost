
use alloy::{primitives::FixedBytes, rpc::types::beacon::BlsPublicKey};
use config::InclusionListConfig;
use cb_common::{commit::request::SignRequest, config::{load_module_config, StartModuleConfig}, utils::initialize_tracing_log};
use mock_relay::MockRelay;
use tree_hash::Hash256;

mod config;
mod mock_relay;
mod types;
mod client;




#[tokio::main]
async fn main() -> Result<(), ()> {
    initialize_tracing_log();

    let config = load_module_config::<InclusionListConfig>().expect("failed to load config");

    // a beacon node delegates its proposer rights to commit-boost
    let pubkey = BlsPublicKey::default();

    let relay = MockRelay::new();

    Ok(())
}


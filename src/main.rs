use std::{collections::HashMap, sync::Arc};

use cb_common::{config::load_module_config, utils::initialize_tracing_log};
use config::InclusionListConfig;

use inclusion_boost::{
    error::InclusionListBoostError, sidecar::InclusionSideCar, types::InclusionBoostCache,
};

use alloy::{
    primitives::B256,
    providers::{ProviderBuilder, RootProvider},
    transports::http::Http,
};
use parking_lot::RwLock;

mod config;
mod inclusion_boost;
mod lookahead;
mod test;
mod pbs;


#[tokio::main]
async fn main() -> Result<(), InclusionListBoostError> {
    initialize_tracing_log();

    let config = load_module_config::<InclusionListConfig>().expect("failed to load config");
    let eth_provider: RootProvider<Http<reqwest::Client>> =
        ProviderBuilder::new().on_http(config.extra.execution_api.parse().unwrap());
    let cache = Arc::new(InclusionBoostCache {
        block_cache: Arc::new(RwLock::new(HashMap::new())),
        inclusion_list_cache: Arc::new(RwLock::new(HashMap::new())),
    });

    let inclusion_sidecar = InclusionSideCar::new(config, eth_provider, cache);

    inclusion_sidecar.run().await?;

    Ok(())
}

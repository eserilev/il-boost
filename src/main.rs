use std::{collections::HashMap, fs, sync::Arc};

use cb_common::{
    config::{load_pbs_custom_config, load_commit_module_config, StaticModuleConfig},
    utils::initialize_tracing_log,
};
use cb_pbs::{PbsService, PbsState};
use config::InclusionListConfig;
use serde::Deserialize;

use inclusion_boost::{
    error::InclusionListBoostError, sidecar::InclusionSideCar, types::InclusionBoostCache,
};
use types::MainConfig;

use crate::pbs::InclusionBoostApi;
use alloy::{
    providers::{ProviderBuilder, RootProvider},
    transports::http::Http,
};
use parking_lot::RwLock;

mod config;
mod inclusion_boost;
mod lookahead;
mod pbs;
mod test;
mod types;

#[tokio::main]
async fn main() -> Result<(), InclusionListBoostError> {
    // parse_toml();
    let config = load_commit_module_config::<InclusionListConfig>().expect("failed to load config");
    let _ = initialize_tracing_log(&config.id);
  
    let eth_provider: RootProvider<Http<reqwest::Client>> =
        ProviderBuilder::new().on_http(config.extra.execution_api.parse().unwrap());
    let cache = Arc::new(InclusionBoostCache {
        block_cache: Arc::new(RwLock::new(HashMap::new())),
        inclusion_list_cache: Arc::new(RwLock::new(HashMap::new())),
    });

    let (pbs_module, pbs_module_custom_data) = load_pbs_custom_config::<InclusionListConfig>().expect("failed to load pbs config");

    let state = PbsState::new(pbs_module).with_data(pbs_module_custom_data);

    let inclusion_sidecar =
        InclusionSideCar::new(config, eth_provider, cache, state.config.pbs_config.port);

    let pbs_server = tokio::spawn(async move {
        let _ = PbsService::run::<InclusionListConfig, InclusionBoostApi>(state).await;
    });

    let il_sidecar = tokio::spawn(async move {
        let _ = inclusion_sidecar.run().await;
    });


    let _ = tokio::join!(pbs_server, il_sidecar);

    Ok(())
}

fn parse_toml() {
    let config_str = fs::read_to_string("./cb-config.toml").expect("Failed to read config file");

    let config: MainConfig = toml::from_str(&config_str).expect("Failed to parse config file");

    std::env::set_var("CB_MODULE_ID", config.modules.first().unwrap().id.clone());
    std::env::set_var("CB_SIGNER_JWT", config.modules.first().unwrap().id.clone());
    std::env::set_var("SIGNER_SERVER", "2000");
    std::env::set_var("CB_CONFIG", "./config.toml");
}

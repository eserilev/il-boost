
use std::{collections::HashMap, sync::Arc};

use client::commit_boost::CommitBoostClient;
use config::InclusionListConfig;
use cb_common::{config::load_module_config, utils::initialize_tracing_log};
use error::InclusionListBoostError;
use futures::StreamExt;
use lookahead::LookaheadProvider;
use mev_share_sse::EventClient;
use ethers::{providers::{Http, Middleware, Provider}, types::{BlockNumber, H256}};
use parking_lot::RwLock;

mod config;
mod client;
mod lookahead;
mod error;
mod pool;


#[derive(Debug, Clone, serde::Deserialize)]
struct HeadEvent {
    slot: u64,
    block: H256,
    epoch_transition: bool,
}

struct BlockCache {
    cache: Arc<RwLock<HashMap<u64, Vec<H256>>>>
}

#[tokio::main]
async fn main() -> Result<(), InclusionListBoostError> {
    initialize_tracing_log();

    let config = load_module_config::<InclusionListConfig>().expect("failed to load config");
    let block_cache = BlockCache {
        cache: Arc::new(RwLock::new(HashMap::new())),
    };
    
    let eth_provider = Arc::new(Provider::<Http>::try_from(config.extra.execution_api).map_err(|_| {
        "Failed to fetch eth provider".to_string()
    })?);
    
    let commit_boost_client = CommitBoostClient::new("http://localhost:33950/").await.map_err(|_| {
        "Failed to initialize the commit boost client.".to_string()
    })?;
    
    let lookahead_provider = LookaheadProvider::new(&config.extra.beacon_api);
    let mut lookahead = lookahead_provider.get_current_lookahead().await?;
    let lookahead_size = lookahead.len();
    tracing::info!(lookahead_size, "Proposer lookahead fetched");

    let client = EventClient::default();
    let target = format!("{}/eth/v1/events?topics=head", config.extra.beacon_api);
    let mut sub = client.subscribe::<HeadEvent>(&target).await?;

    while let Some(event) = sub.next().await {

        let event = event?;

        if event.epoch_transition {
            lookahead = lookahead_provider.get_current_lookahead().await?;
            tracing::info!("Epoch transition, fetched new proposer lookahead...");
        }

        let Some(next_proposer) = lookahead.iter().find(|duty| duty.slot == event.slot + 1) else {
            tracing::info!("At end of epoch, waiting");
            continue;
        };

        if let Some(block) = eth_provider.get_block(BlockNumber::Latest).await.map_err(|_| "Failed to get current block".to_string())? {
            let transactions = block.transactions;
            let block_number = block.number;
            if let Some(block_number) = block_number {
                // insert the new block into the cache
                block_cache.cache.write().insert(block_number.as_u64(), transactions);

                // prune an old block from the cache
                block_cache.cache.write().remove(&(block_number.as_u64() - 10));
            };
        };
    }

    Ok(())
}


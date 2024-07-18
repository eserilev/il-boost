use std::{collections::HashMap, sync::Arc};

use cb_common::{config::{load_module_config, StartModuleConfig}, utils::initialize_tracing_log};
use client::{commit_boost::CommitBoostClient};
use config::InclusionListConfig;
use error::InclusionListBoostError;

use futures::StreamExt;
use inclusion_list::{get_censored_transactions, types::{InclusionList, Transaction}};
use lookahead::LookaheadProvider;
use mev_share_sse::EventClient;
use parking_lot::RwLock;
use alloy::{
    eips::BlockId, primitives::B256, providers::{ext::TxPoolApi, Provider, ProviderBuilder}
};
use tracing::trace;

mod client;
mod config;
mod error;
mod lookahead;
mod pool;
mod test;
mod inclusion_list;

#[derive(Debug, Clone, serde::Deserialize)]
struct HeadEvent {
    slot: String,
    block: B256,
    epoch_transition: bool,
}


struct InclusionBoostCache {
    block_cache: Arc<RwLock<HashMap<u64, Vec<B256>>>>,
    inclusion_list_cache: Arc<RwLock<HashMap<u64, InclusionList>>>,
}

#[tokio::main]
async fn main() -> Result<(), InclusionListBoostError> {
    initialize_tracing_log();

    // std::env::set_var("CB_MODULE_ID", "test");
    // std::env::set_var("CB_SIGNER_JWT", "test");
    // std::env::set_var("SIGNER_SERVER", "test");
    // std::env::set_var("CB_CONFIG", "./config.toml");
    // let config = load_module_config::<InclusionListConfig>().expect("failed to load config");

    let config = InclusionListConfig {
        beacon_api: "http://127.0.0.1:62136".to_string(),
        execution_api: "http://127.0.0.1:62128".to_string()
    };

    let il_boost_cache = InclusionBoostCache {
        block_cache: Arc::new(RwLock::new(HashMap::new())),
        inclusion_list_cache: Arc::new(RwLock::new(HashMap::new())),
    };

    let provider = ProviderBuilder::new().on_http(config.execution_api.parse().unwrap());

    // let eth_provider = Arc::new(
    //     Provider::<Http>::try_from(config.extra.execution_api)
    //         .map_err(|_| "Failed to fetch eth provider".to_string())?,
    // );

    let commit_boost_client = CommitBoostClient::new("http://localhost:33950/")
        .await
        .map_err(|_| "Failed to initialize the commit boost client.".to_string())?;

    let lookahead_provider = LookaheadProvider::new(&config.beacon_api);
    let mut lookahead = lookahead_provider.get_current_lookahead().await?;
    let lookahead_size = lookahead.len();
    tracing::info!(lookahead_size, "Proposer lookahead fetched");

    let client = EventClient::default();
    let target = format!("{}/eth/v1/events?topics=head", config.beacon_api);
    let mut sub = client.subscribe::<HeadEvent>(&target).await?;

    while let Some(event) = sub.next().await {
        let event = event?;

        if event.epoch_transition {
            lookahead = lookahead_provider.get_current_lookahead().await?;
            tracing::info!("Epoch transition, fetched new proposer lookahead...");
        }

        let Some(next_proposer) = lookahead.iter().find(|duty| duty.slot == u64::from_str_radix(&event.slot, 10).unwrap() + 1) else {
            tracing::info!("At end of epoch, waiting");
            continue;
        };
        let Some(block) = provider.get_block(BlockId::latest(), alloy::rpc::types::BlockTransactionsKind::Full).await.unwrap() else { continue; };
        println!("block num {:?}", block.header.number);
        println!("event {:?}", event.slot);
        println!("transactions {:?}", block.transactions.len());

        let tx_pool = provider.txpool_content().await.unwrap();

        let mut censored_transactions = vec![];

        for (_, transactions) in tx_pool.pending {
            let transactions = transactions
                .iter()
                .map(|(_, tx)| tx.clone().into())
                .collect::<Vec<Transaction>>();

            censored_transactions.append(&mut get_censored_transactions(transactions, &block));
        }

        let inclusion_list = if censored_transactions.len() > 0 {
            Some(InclusionList::new(
                next_proposer.slot, 
                next_proposer.validator_index,
                censored_transactions
            ))
        } else {
            None
        };

        commit_boost_client.submit_inclusion_list(inclusion_list.as_ref()).await;
        let transactions = block.transactions.as_hashes();
    
        let block_number = block.header.number;
           
        if let Some(block_number) = block_number {
            if let Some(transactions) = transactions {
                // insert the new block into the cache
                il_boost_cache
                    .block_cache
                    .write()
                    .insert(block_number, transactions.to_vec());

                    // prune an old block from the cache
                il_boost_cache
                    .block_cache
                    .write()
                    .remove(&(block_number - 10));
            }
            
            tracing::info!(?inclusion_list);
            let Some(inclusion_list) = inclusion_list else { continue; };

            // TODO need to prune this list eventually
            il_boost_cache
                .inclusion_list_cache
                .write()
                .insert(next_proposer.slot, inclusion_list);
        };
    }

    Ok(())
}


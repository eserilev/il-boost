use std::{collections::HashMap, sync::Arc, thread::sleep, time::Duration};

use alloy::{
    providers::{ext::TxPoolApi, Provider, RootProvider},
    rpc::types::{beacon::events::HeadEvent, Block},
    transports::http::Http,
};

use cb_common::config::StartCommitModuleConfig;
use futures::StreamExt;
use mev_share_sse::EventClient;

use crate::{
    config::InclusionListConfig,
    inclusion_boost::types::InclusionList,
    lookahead::{error::LookaheadError, LookaheadProvider},
};

use super::{
    error::InclusionListBoostError,
    types::{InclusionBoostCache, Transaction},
    InclusionBoost,
};

pub struct InclusionSideCar {
    inclusion_boost: InclusionBoost,
    eth_provider: RootProvider<Http<reqwest::Client>>,
    cache: Arc<InclusionBoostCache>,
    il_config: InclusionListConfig,
}

impl InclusionSideCar {
    pub fn new(
        config: StartCommitModuleConfig<InclusionListConfig>,
        eth_provider: RootProvider<alloy::transports::http::Http<reqwest::Client>>,
        cache: Arc<InclusionBoostCache>,
    ) -> Self {
        let inclusion_boost = InclusionBoost::new(
            config.id.to_string(),
            config.signer_client,
            HashMap::new(),
            config.extra.clone().relay, // TODO get from config
        );

        Self {
            inclusion_boost,
            eth_provider,
            cache,
            il_config: config.extra,
        }
    }

    pub async fn run(&mut self) -> Result<(), InclusionListBoostError> {
        let lookahead_provider = LookaheadProvider::new(&self.il_config.beacon_api);
        sleep(Duration::from_secs(60));
        let pubkeys = self.inclusion_boost.signer_client.get_pubkeys().await?;

        for p in pubkeys.consensus {
            let index = get_validator_index(&self.il_config.beacon_api, &p.to_string()).await?;
            if let Some(validator_index) = index {
                println!("validator_index {}", validator_index);
                self.inclusion_boost
                    .validator_keys
                    .insert(validator_index as usize, p.into());
            }
        }

        let mut lookahead = lookahead_provider.get_current_lookahead().await?;
        let mut next_lookahead = lookahead_provider.get_next_epoch_lookahead().await?;

        for future_proposer in next_lookahead {
            let res = self
                .inclusion_boost
                .delegate_inclusion_list_authority(
                    future_proposer.validator_index,
                    future_proposer.slot,
                )
                .await;
            println!("{:?}", res);
        }

        let lookahead_size = lookahead.len();
        tracing::info!(lookahead_size, "Initial proposer lookahead fetched");

        let event_client = EventClient::default();
        let target = format!("{}/eth/v1/events?topics=head", self.il_config.beacon_api);
        let mut sub = event_client.subscribe::<HeadEvent>(&target).await?;

        while let Some(head_event) = sub.next().await {
            let head_event = head_event?;

            if head_event.epoch_transition {
                lookahead = lookahead_provider.get_current_lookahead().await?;
                next_lookahead = lookahead_provider.get_next_epoch_lookahead().await?;
                for future_proposer in next_lookahead {
                    let res = self
                        .inclusion_boost
                        .delegate_inclusion_list_authority(
                            future_proposer.validator_index,
                            future_proposer.slot,
                        )
                        .await;
                    println!("{:?}", res);
                }
                tracing::info!("Epoch transition, fetched new proposer lookahead...");
            }

            // Get the next slots proposer
            let Some(next_proposer) = lookahead
                .iter()
                .find(|duty| duty.slot == &head_event.slot + 1)
            else {
                tracing::info!("At end of epoch, waiting");
                continue;
            };

            // TODO check if next slots proposer is ours
            let block_number = self.get_block_number_by_slot(head_event.slot - 1).await?;

            let Some(block_number) = block_number else {
                continue;
            };

            let Some(latest_block) = self.get_block_by_number(block_number).await? else {
                continue;
            };

            tracing::info!(
                block_number = latest_block.header.number,
                transaction_count = latest_block.transactions.len(),
                current_slot = head_event.slot,
                "Fetched latest block"
            );

            // TODO we'll probably want to cache the inclusion list so we can validate merkle proofs later

            let Some(inclusion_list) = self
                .build_inclusion_list(
                    &latest_block,
                    next_proposer.slot,
                    next_proposer.validator_index,
                )
                .await?
            else {
                tracing::info!(
                    block_number = latest_block.header.number,
                    current_slot = head_event.slot,
                    "No inclusion for slot"
                );
                continue;
            };

            self.inclusion_boost
                .submit_inclusion_list_to_relay(next_proposer.validator_index, inclusion_list)
                .await?;
        }

        tracing::info!("Finished running sidecar");

        Ok(())
    }

    async fn get_block_by_number(
        &self,
        block_number: u64,
    ) -> Result<Option<Block>, InclusionListBoostError> {
        self.eth_provider
            .get_block_by_number(alloy::eips::BlockNumberOrTag::Number(block_number), true)
            .await
            .map_err(|e| e.into())
    }

    async fn get_block_number_by_slot(
        &self,
        slot: u64,
    ) -> Result<Option<u64>, InclusionListBoostError> {
        tracing::info!(slot, "Get block number by slot");

        let url = format!(
            "{}/eth/v1/beacon/blocks/{}",
            self.il_config.beacon_api, slot
        );
        let res = reqwest::get(url).await?;
        let json: serde_json::Value = serde_json::from_str(&res.text().await?)?;

        let Some(block_number) = json.pointer("/data/message/body/execution_payload/block_number")
        else {
            return Ok(None);
        };

        let Some(block_number_str) = block_number.as_str() else {
            return Ok(None);
        };
        Ok(Some(block_number_str.parse::<u64>()?))
    }

    /// Builds an inclusion list for slot N by comparing pending transactions in the mem pool
    /// with the block from slot N - 1
    async fn build_inclusion_list(
        &self,
        latest_block: &Block,
        slot: u64,
        validator_index: usize,
    ) -> Result<Option<InclusionList>, InclusionListBoostError> {
        let mut pending_txs = vec![];
        let tx_pool = self.eth_provider.txpool_content().await?;

        tracing::info!(
            transaction_count = tx_pool.pending.len(),
            "Fetched pending transactions from the local memory pool"
        );

        for (_, transactions) in tx_pool.pending {
            let transactions = transactions
                .iter()
                .map(|(_, tx)| tx.clone().into())
                .collect::<Vec<Transaction>>();

            pending_txs.extend(transactions);
        }

        let filtered_transactions =
            InclusionBoost::get_filtered_transactions(&pending_txs, latest_block);

        tracing::info!(
            transaction_count = filtered_transactions.len(),
            "Identified a list of potentially filtered transactions"
        );

        // if filtered_transactions.len() == 0 {
        //     return Ok(None);
        // };

        Ok(Some(InclusionList::new(
            slot,
            validator_index,
            filtered_transactions,
        )))
    }
}

async fn get_validator_index(
    beacon_url: &str,
    validator_pubkey: &str,
) -> Result<Option<u64>, LookaheadError> {
    let url = format!("{beacon_url}/eth/v1/beacon/states/head/validators?id={validator_pubkey}");
    let res = reqwest::get(url).await?;
    let json: serde_json::Value = serde_json::from_str(&res.text().await?)?;

    let Some(validator_index) = json.pointer("/data/0/index") else {
        return Ok(None);
    };

    let Some(validator_index_str) = validator_index.as_str() else {
        return Ok(None);
    };

    Ok(Some(validator_index_str.parse::<u64>()?))
}

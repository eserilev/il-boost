use std::{collections::HashMap, sync::Arc};

use alloy::{
    eips::BlockId,
    providers::{ext::TxPoolApi, Provider, RootProvider},
    rpc::types::{beacon::events::HeadEvent, Block, BlockTransactionsKind},
    transports::http::Http,
};

use cb_common::config::StartModuleConfig;
use futures::StreamExt;
use mev_share_sse::EventClient;

use crate::{
    config::InclusionListConfig, inclusion_boost::types::InclusionList,
    lookahead::LookaheadProvider,
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
        config: StartModuleConfig<InclusionListConfig>,
        eth_provider: RootProvider<alloy::transports::http::Http<reqwest::Client>>,
        cache: Arc<InclusionBoostCache>,
        mev_port: u16,
    ) -> Self {
        let inclusion_boost = InclusionBoost::new(
            config.id,
            config.signer_client,
            HashMap::new(),
            mev_port.to_string(), // TODO get from config
        );

        Self {
            inclusion_boost,
            eth_provider,
            cache,
            il_config: config.extra,
        }
    }

    pub async fn run(&self) -> Result<(), InclusionListBoostError> {
        let lookahead_provider = LookaheadProvider::new(&self.il_config.beacon_api);

        let mut lookahead = lookahead_provider.get_current_lookahead().await?;
        let lookahead_size = lookahead.len();
        tracing::info!(lookahead_size, "Initial proposer lookahead fetched");

        let event_client = EventClient::default();
        let target = format!("{}/eth/v1/events?topics=head", self.il_config.beacon_api);
        let mut sub = event_client.subscribe::<HeadEvent>(&target).await?;

        while let Some(head_event) = sub.next().await {
            let head_event = head_event?;

            if head_event.epoch_transition {
                lookahead = lookahead_provider.get_current_lookahead().await?;
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

            let Some(latest_block) = self.get_latest_block().await? else {
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
                continue;
            };

            self.inclusion_boost
                .submit_inclusion_list_to_relay(next_proposer.validator_index, inclusion_list)
                .await?;
        }

        Ok(())
    }

    async fn get_latest_block(&self) -> Result<Option<Block>, InclusionListBoostError> {
        self.eth_provider
            .get_block(BlockId::latest(), BlockTransactionsKind::Full)
            .await
            .map_err(|e| e.into())
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

        let censored_transactions =
            InclusionBoost::get_censored_transactions(&pending_txs, latest_block);

        tracing::info!(
            transaction_count = censored_transactions.len(),
            "Identified a list of potentially censored transactions"
        );

        if censored_transactions.len() == 0 {
            return Ok(None);
        };

        Ok(Some(InclusionList::new(
            slot,
            validator_index,
            censored_transactions,
        )))
    }
}

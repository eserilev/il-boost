use std::collections::HashMap;

use alloy::{
    primitives::B256,
    rpc::types::{
        beacon::{BlsPublicKey, BlsSignature},
        Block,
    },
};
use cb_common::commit::{client::SignerClient, error::SignerClientError, request::SignRequest};
use error::InclusionListBoostError;
use serde::Serialize;
use tree_hash::TreeHash;
use types::{InclusionList, InclusionRequest, Transaction};

pub mod error;
pub mod sidecar;
pub mod types;

const CONSTRAINTS_PATH: &str = "constraints/v1/set_constraints";

/// Implements an inclusion list flavor
/// of commit-boost
pub struct InclusionBoost {
    pub module_id: String,
    pub signer_client: SignerClient,
    pub validator_keys: HashMap<usize, BlsPublicKey>,
    pub relay_client: reqwest::Client,
    pub relay_url: String,
}

impl InclusionBoost {
    pub fn new(
        module_id: String,
        signer_client: SignerClient,
        validator_keys: HashMap<usize, BlsPublicKey>,
        relay_url: String,
    ) -> Self {
        Self {
            module_id,
            signer_client,
            validator_keys,
            relay_client: reqwest::Client::new(),
            relay_url,
        }
    }

    /// Calculate which transactions may be censored from a list of transactions by
    /// comparing if any of these transactions could have made it into `block`
    pub fn get_censored_transactions(
        transactions: &Vec<Transaction>,
        block: &Block<alloy::rpc::types::Transaction>,
    ) -> Vec<B256> {
        let mut censored_transactions = vec![];
        let mut gas_left = block.header.gas_limit - block.header.gas_used;

        for tx in transactions {
            if let Some(max_priority_fee_per_gas) = tx.max_priority_fee_per_gas {
                if max_priority_fee_per_gas > 0 && gas_left > 0 {
                    gas_left -= tx.gas;
                    censored_transactions.push(tx.tx_hash);
                }
            }
        }

        censored_transactions
    }

    /// Submit the inclusion list to the relay
    /// This using the commit-boost signing module to sign the list
    /// And then forwards the signed list to the constraints API
    pub async fn submit_inclusion_list_to_relay(
        &self,
        validator_index: usize,
        inclusion_list: InclusionList,
    ) -> Result<Option<()>, InclusionListBoostError> {
        let Some(validator_key) = self.validator_keys.get(&validator_index) else {
            return Ok(None);
        };

        let signature = self
            .sign_inclusion_list(&inclusion_list, *validator_key)
            .await?;

        self.post_inclusion_request(signature, inclusion_list)
            .await?;

        Ok(Some(()))
    }

    /// Sign an inclusion list via the commit-boost signing module
    async fn sign_inclusion_list(
        &self,
        inclusion_list: &InclusionList,
        validator_key: BlsPublicKey,
    ) -> Result<BlsSignature, SignerClientError> {
        let inclusion_list_root = inclusion_list.tree_hash_root();
        let sign_request = SignRequest::builder(self.module_id.clone(), validator_key)
            .with_root(inclusion_list_root.into());

        self.signer_client.request_signature(&sign_request).await
    }

    /// Post a signed inclusion list to a relay
    async fn post_inclusion_request(
        &self,
        signature: BlsSignature,
        inclusion_list: InclusionList,
    ) -> Result<Option<()>, InclusionListBoostError> {
        let url = format!("{}{CONSTRAINTS_PATH}", self.relay_url);
        println!("{}", url);

        let request = InclusionRequest {
            message: inclusion_list,
            signature,
        };
      
        println!("{}",  serde_json::to_string(&request).unwrap());

        tracing::info!(url, payload=?request, "POST request sent");

        let response = self.relay_client.post(url).json(&request).send().await?;

        println!("{:?}", response);

        let status = response.status();
        let response_bytes = response.bytes().await?;

        if !status.is_success() {
            let err = String::from_utf8_lossy(&response_bytes).into_owned();
            tracing::error!(err, "failed to get signature");
            return Ok(None);
        }

        Ok(Some(()))
    }
}

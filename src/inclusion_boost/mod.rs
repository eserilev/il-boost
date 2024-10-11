use std::{collections::HashMap, time::Duration};

use alloy::eips::eip2718::Encodable2718;
use alloy::rpc::types::{
    beacon::{BlsPublicKey, BlsSignature},
    Block,
};
use cb_common::commit::{client::SignerClient, error::SignerClientError, request::SignRequest};
use error::InclusionListBoostError;
use tree_hash::TreeHash;
use types::{
    Constraint, InclusionList, InclusionListDelegateMessage, InclusionListDelegateSignedMessage,
    InclusionRequest, Transaction,
};

pub mod error;
pub mod sidecar;
pub mod types;

const CONSTRAINTS_PATH: &str = "/eth/v1/builder/set_constraints";
const DELEGATE_PATH: &str = "/eth/v1/builder/elect_preconfer";

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

    /// Calculate which transactions may be filtered from a list of transactions by
    /// comparing if any of these transactions could have made it into `block`
    pub fn get_filtered_transactions(
        transactions: &Vec<Transaction>,
        block: &Block<alloy::rpc::types::Transaction>,
    ) -> Vec<Constraint> {
        let mut filtered_transactions = vec![];
        for tx in transactions {
            println!("Including transaction {:?}", tx);
            let encoded = tx.envelope.encoded_2718();
            filtered_transactions.push(Constraint {
                payload: encoded.into(),
                hash: *tx.envelope.tx_hash(),
            });
        }

        println!("Filtered transactions: {:?}", filtered_transactions);
        filtered_transactions
    }

    pub async fn delegate_inclusion_list_authority(
        &self,
        validator_index: usize,
        slot: u64,
    ) -> Result<Option<()>, InclusionListBoostError> {
        let Some(validator_key) = self.validator_keys.get(&validator_index) else {
            return Ok(None);
        };

        tracing::info!(
            validator_index,
            slot,
            "Delegating inclusion list building responsibilities to IL Boost"
        );

        let message = InclusionListDelegateMessage {
            preconfer_pubkey: validator_key.clone(),
            slot_number: slot,
            chain_id: 7014190335,
            gas_limit: u64::MAX,
        };

        let message_root = message.tree_hash_root();
        let sign_request = SignRequest::builder(*validator_key).with_root(message_root.into());

        let signature = self.signer_client.request_signature(&sign_request).await?;

        let signed_message = InclusionListDelegateSignedMessage { message, signature };

        self.post_inclusion_delegate_request(signed_message).await
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

        tracing::info!(validator_index, "Submitting inclusion list to relay");

        let signature = self
            .sign_inclusion_list(&inclusion_list, *validator_key)
            .await?;

        tracing::info!("Inclusion list signed");

        match self.post_inclusion_request(signature, inclusion_list).await {
            Ok(res) => println!("{:?}", res),
            Err(e) => return Err(e),
        };

        tracing::info!("Inclusion list sent");

        Ok(Some(()))
    }

    /// Sign an inclusion list via the commit-boost signing module
    async fn sign_inclusion_list(
        &self,
        inclusion_list: &InclusionList,
        validator_key: BlsPublicKey,
    ) -> Result<BlsSignature, SignerClientError> {
        let inclusion_list_root = inclusion_list.tree_hash_root();
        let sign_request =
            SignRequest::builder(validator_key.into()).with_root(inclusion_list_root.into());

        self.signer_client.request_signature(&sign_request).await
    }

    async fn post_inclusion_delegate_request(
        &self,
        signed_message: InclusionListDelegateSignedMessage,
    ) -> Result<Option<()>, InclusionListBoostError> {
        let url = format!("{}{DELEGATE_PATH}", self.relay_url);

        tracing::info!(url, payload=?signed_message, "POST request sent");

        let response = match self
            .relay_client
            .post(url)
            .timeout(Duration::from_secs(10))
            .json(&signed_message)
            .send()
            .await
        {
            Ok(res) => res,
            Err(e) => {
                println!("{:?}", e);
                return Err(e.into());
            }
        };

        let status = response.status();
        let response_bytes = response.bytes().await?;

        if !status.is_success() {
            let err = String::from_utf8_lossy(&response_bytes).into_owned();
            tracing::error!(err, "failed to get signature");
            return Ok(None);
        }

        Ok(Some(()))
    }

    /// Post a signed inclusion list to a relay
    async fn post_inclusion_request(
        &self,
        signature: BlsSignature,
        inclusion_list: InclusionList,
    ) -> Result<Option<()>, InclusionListBoostError> {
        let url = format!("{}{CONSTRAINTS_PATH}", self.relay_url);

        let request = InclusionRequest {
            message: inclusion_list,
            signature,
        };

        tracing::info!(url, payload=?request, "POST request sent");

        let response = match self
            .relay_client
            .post(url)
            .timeout(Duration::from_secs(10))
            .json(&request)
            .send()
            .await
        {
            Ok(res) => {
                println!(
                    "Relay response code for inclusion list: {}",
                    res.status().as_u16()
                );
                res
            }
            Err(e) => {
                println!("{:?}", e);
                return Err(e.into());
            }
        };

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

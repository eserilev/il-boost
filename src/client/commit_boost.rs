use std::sync::Arc;
use cb_common::commit::request::SignRequest;
use parking_lot::RwLock;
use alloy::rpc::types::beacon::{BlsPublicKey, BlsSignature};
use tree_hash::TreeHash;

use super::types::InclusionList;


const ID: &str = "inclusion-list-boost";
// TODO add actual routes
const COMMIT_BOOST_API: &str = "commit-boost-api-url";
const PUBKEYS_PATH: &str = "pubkeys-path";
const SIGN_REQUEST_PATH: &str= "sign-request-path";

#[derive(Debug, Clone)]
pub struct CommitBoostClient {
    url: String,
    client: reqwest::Client,
    pubkeys: Arc<RwLock<Vec<BlsPublicKey>>>,
}

#[derive(Debug)]
pub enum CommitBoostError {
    Reqwest(reqwest::Error),
}

impl CommitBoostClient {
    pub async fn new(url: impl Into<String>) -> Result<Self, CommitBoostError> {
        let client = Self {
            url: url.into(),
            client: reqwest::Client::new(),
            pubkeys: Arc::new(RwLock::new(Vec::new())),
        };

        let mut this = client.clone();
        tokio::spawn(async move {
            this.load_pubkeys().await.expect("failed to load pubkeys");
        });

        Ok(client)
    }

    pub async fn load_pubkeys(&mut self) -> Result<(), CommitBoostError> {
        loop {
            let url = format!("{}{COMMIT_BOOST_API}{PUBKEYS_PATH}", self.url);

            tracing::info!(url, "Loading signatures from commit_boost");

            let response = match self.client.get(url).send().await {
                Ok(res) => res,
                Err(e) => {
                    tracing::error!(err = ?e, "failed to get public keys from commit-boost, retrying...");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    continue;
                }
            };

            let status = response.status();
            let response_bytes = response.bytes().await.expect("failed to get bytes");

            if !status.is_success() {
                let err = String::from_utf8_lossy(&response_bytes).into_owned();
                tracing::error!(err, ?status, "failed to get public keys, retrying...");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }

            let pubkeys: Vec<BlsPublicKey> =
                serde_json::from_slice(&response_bytes).expect("failed deser");

            {
                let mut pk = self.pubkeys.write();
                *pk = pubkeys;
                return Ok(());
            } // drop write lock
        }
    }

    // TODO: error handling
    pub async fn sign_inclusion_list(&self, inclusion_list: &InclusionList) -> Option<BlsSignature> {
        let root = inclusion_list.tree_hash_root();
        let request =
            SignRequest::builder(ID, *self.pubkeys.read().first().expect("pubkeys loaded"))
                .with_root(root.into());

        let url = format!("{}{COMMIT_BOOST_API}{SIGN_REQUEST_PATH}", self.url);

        tracing::debug!(url, ?request, "Requesting signature from commit_boost");

        let response = reqwest::Client::new()
            .post(url)
            .json(&request)
            .send()
            .await
            .expect("failed to get request");

        let status = response.status();
        let response_bytes = response.bytes().await.expect("failed to get bytes");

        if !status.is_success() {
            let err = String::from_utf8_lossy(&response_bytes).into_owned();
            tracing::error!(err, "failed to get signature");
            return None;
        }

        serde_json::from_slice(&response_bytes).expect("failed deser")
    }
}

#[cfg(test)]
mod tests {
    use alloy::{
        hex, network::{EthereumWallet, NetworkWallet, TransactionBuilder}, node_bindings::Anvil, primitives::{B256, U256}, providers::{Provider, ProviderBuilder}, rpc::types::TransactionRequest, signers::local::PrivateKeySigner
    };
    
    use ethers::core::k256::ecdsa::signature::Signer;
    use tree_hash::Hash256;

    use super::*;

    #[tokio::test]
    async fn test_commit_boost_signature() {
        let _ = tracing_subscriber::fmt::try_init();

        // Spin up a forked Anvil node.
        // Ensure `anvil` is available in $PATH.
        let anvil = Anvil::new().try_spawn().unwrap();

        let client = CommitBoostClient::new("http://localhost:33950")
            .await
            .unwrap();

        // Create a provider.
        let rpc_url = anvil.endpoint().parse().unwrap();
        let _provider = ProviderBuilder::new().on_http(rpc_url);

        // Set up signer from the first default Anvil account (Alice).
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        let wallet = EthereumWallet::from(signer);

        // Create two users, Alice and Bob.
        let alice = anvil.addresses()[0];
        let bob = anvil.addresses()[1];

        // Build a transaction to send 100 wei from Alice to Bob.
        // The `from` field is automatically filled to the first signer's address (Alice).
        let tx = TransactionRequest::default()
            .with_from(alice)
            .with_to(bob)
            .with_nonce(0)
            .with_chain_id(anvil.chain_id())
            .with_value(U256::from(100))
            .with_gas_limit(21_000)
            .with_max_priority_fee_per_gas(1_000_000_000)
            .with_max_fee_per_gas(20_000_000_000);

        // Build and sign the transaction using the `EthereumWallet` with the provided wallet.
        let signed = tx.build(&wallet).await.unwrap();

        let message = InclusionList {
            slot: 20,
            validator_index: 1,
            transaction: signed.tx_hash().clone()
        };
        
        let signature = client.sign_inclusion_list(&message).await.unwrap();

        println!("Message signed, signature: {signature}");
        assert!(true==false);
        // assert!(signature.verify(&message.tree_hash_root(), &client.pubkeys.read().first().unwrap()));
    }
}
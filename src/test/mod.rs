mod test {

    use std::{collections::HashMap, convert::Infallible, net::SocketAddr};

    use alloy::rpc::types::{
        beacon::{BlsPublicKey, BlsSignature},
        Block,
    };
    use cb_common::{commit::request::SignRequest, signer::Signer};
    use hyper::{
        service::{make_service_fn, service_fn},
        Body, Request, Response, Server,
    };
    use reth_transaction_pool::{
        test_utils::{MockTransactionFactory, TestPoolBuilder},
        TransactionOrigin, TransactionPool,
    };
    use tokio::task::JoinHandle;
    use tree_hash::TreeHash;

    use cb_common::commit::client::SignerClient;

    use crate::inclusion_boost::{
        types::{InclusionList, Transaction},
        InclusionBoost,
    };
    const ID: &str = "DA_COMMIT";
    struct MockRelay {
        server_handle: Option<JoinHandle<()>>,
    }

    impl MockRelay {
        pub async fn new(port: u16) -> Self {
            // Define the address for the server
            let addr = SocketAddr::from(([127, 0, 0, 1], port));

            // Create a service
            let make_svc = make_service_fn(|_conn| async {
                Ok::<_, Infallible>(service_fn(MockRelay::handle_request))
            });

            // Create the server
            let server = Server::bind(&addr).serve(make_svc);

            // Run the server
            let server_future = async move {
                if let Err(e) = server.await {
                    eprintln!("Server error: {}", e);
                }
            };

            MockRelay {
                server_handle: Some(tokio::spawn(server_future)),
            }
        }

        async fn handle_request(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
            let signature = BlsSignature::default();
            Ok(Response::new(Body::from(signature.as_slice().to_owned())))
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    pub async fn build_mock_inclusion_list_request() {
        // TODO load via config
        let mock_signer_client = SignerClient::new(format!("127.0.0.1:20000"), "DA_COMMIT");

        let _ = MockRelay::new(33950).await;

        let mut mock_validator_pubkeys = HashMap::new();
        let pubkey_result = mock_signer_client.get_pubkeys().await.unwrap();
        let pubkey = pubkey_result.consensus.first().unwrap();
        mock_validator_pubkeys.insert(1, pubkey.clone());

        let inclusion_module = InclusionBoost::new(
            ID.to_string(),
            mock_signer_client,
            mock_validator_pubkeys,
            "http://localhost:33950/".to_string(),
        );

        let txpool = TestPoolBuilder::default();
        let mut mock_tx_factory = MockTransactionFactory::default();
        let transaction = mock_tx_factory.create_eip1559();
        let added_result = txpool
            .add_transaction(TransactionOrigin::Local, transaction.transaction.clone())
            .await;
        let hash = transaction.transaction.get_hash();
        assert_eq!(added_result.unwrap(), hash);

        let transactions = txpool
            .all_transactions()
            .pending
            .iter()
            .map(|t| t.clone().into())
            .collect::<Vec<Transaction>>();

        let mut mock_previous_block: Block<alloy::rpc::types::Transaction> = Block::default();
        mock_previous_block.header.gas_limit = u128::MAX;

        assert_eq!(transactions.len(), 1);

        let censored_transactions =
            InclusionBoost::get_censored_transactions(&transactions, &mock_previous_block);

        assert_eq!(censored_transactions.len(), 1);

        let mock_inclusion_list = InclusionList::new(1, 1, censored_transactions);

        let response = inclusion_module
            .submit_inclusion_list_to_relay(1, mock_inclusion_list)
            .await
            .unwrap();

        assert_eq!(response, Some(()))
    }
}

mod test {

    use std::{collections::HashMap,  net::SocketAddr};
    use alloy::rpc::types::Block;
    use axum::{response::IntoResponse, routing::{post, IntoMakeService}, Json, Router};
    use hyper::StatusCode;
    use reth_transaction_pool::{test_utils::{MockTransactionFactory, TestPoolBuilder}, TransactionOrigin, TransactionPool};
    use tokio::{net::TcpListener, task::JoinHandle};
    use tree_hash::TreeHash;

    use cb_common::commit::client::SignerClient;

    use crate::inclusion_boost::{
        types::{InclusionList, Transaction},
        InclusionBoost,
    };
    const ID: &str = "DA_COMMIT";
    struct MockRelay {
        tpc_listener: TcpListener,
        service: IntoMakeService<Router>
    }

    impl MockRelay {
        pub async fn new(port: u16) -> Self {
            let app = Router::new()
            .route("/", post(MockRelay::handle_request));
    
        // Define an address to bind the server to
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        println!("Listening on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        let service = app.into_make_service();
    


            MockRelay {
                tpc_listener: listener,
                service
            }
        }

        async fn handle_request(Json(payload): Json<InclusionList>) -> impl IntoResponse {
            (StatusCode::OK, Json(payload))
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    pub async fn build_mock_inclusion_list_request() {
        // TODO load via config
        let mock_signer_client = SignerClient::new(format!("127.0.0.1:20000"), "DA_COMMIT");

        let mock_relay = MockRelay::new(33950).await;

        // Run the server
        axum::serve(mock_relay.tpc_listener, mock_relay.service)
            .await
            .unwrap();

        let mut mock_validator_pubkeys = HashMap::new();
        let pubkey_result = mock_signer_client.get_pubkeys().await.unwrap();
        let pubkey = pubkey_result.consensus.first().unwrap();
        mock_validator_pubkeys.insert(1, pubkey.clone());

        let inclusion_module = InclusionBoost::new(
            ID.to_string(),
            mock_signer_client,
            mock_validator_pubkeys,
            "http://localhost:33950/".to_string(),
            // "http://0xaa58208899c6105603b74396734a6263cc7d947f444f396a90f7b7d3e65d102aec7e5e5291b27e08d02c50a050825c2f@18.192.244.122:4040/".to_string(),
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

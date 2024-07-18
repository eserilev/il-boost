use alloy::rpc::types::{beacon::BlsPublicKey, Block};
use reth_transaction_pool::{
    test_utils::{MockTransactionFactory, TestPoolBuilder},
    TransactionOrigin, TransactionPool,
};
use cb_common::{commit::request::SignRequest, signer::Signer};
use tree_hash::TreeHash;

use cb_common::commit::client::SignerClient;


use crate::inclusion_list::{get_censored_transactions, types::{InclusionList, Transaction}};
const ID: &str = "DA_COMMIT";

#[tokio::test(flavor = "multi_thread")]
pub async fn build_mock_inclusion_list_request() {

    let mock_signer = Signer::new_random();

    let mock_signer_client = SignerClient::new(
        format!("127.0.0.1:20000"),
        "DA_COMMIT",
    );

    let txpool = TestPoolBuilder::default();
    let mut mock_tx_factory = MockTransactionFactory::default();
    let transaction = mock_tx_factory.create_eip1559();
    let added_result = txpool.add_transaction(TransactionOrigin::Local, transaction.transaction.clone()).await;
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

    let censored_transactions = get_censored_transactions(transactions, &mock_previous_block);

    assert_eq!(censored_transactions.len(), 1);

    let mock_inclusion_list = InclusionList::new(1, 1, censored_transactions);

    let pubkeys = mock_signer_client.get_pubkeys().await.unwrap();

    let inclusion_list_request = SignRequest::builder(ID, pubkeys.consensus.first().unwrap().clone())
        .with_root(mock_inclusion_list.tree_hash_root().into());

    let signature = mock_signer_client
        .request_signature(&inclusion_list_request).await.unwrap();

    println!("{:?}", signature);

    // TODO forward this constraints message to the bolt API

}


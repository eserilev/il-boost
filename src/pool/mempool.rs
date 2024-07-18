use std::sync::Arc;
use reth_transaction_pool::{
    blobstore::InMemoryBlobStore, validate::ValidTransaction, CoinbaseTipOrdering,
    EthPooledTransaction, Pool, PoolTransaction, TransactionOrigin, TransactionPool,
    TransactionValidationOutcome, TransactionValidator, ValidPoolTransaction,
    test_utils::{MockTransactionFactory}
    
    
};

use ethers::{core::types::Block, types::H256};

use crate::inclusion_list::types::InclusionList;

pub struct MemoryPool {
    pool: Pool<OkValidator, CoinbaseTipOrdering<EthPooledTransaction>, InMemoryBlobStore>,
}

impl MemoryPool {
    pub fn new() -> Self {
        let pool = reth_transaction_pool::Pool::new(
            OkValidator::default(),
            CoinbaseTipOrdering::default(),
            InMemoryBlobStore::default(),
            Default::default(),
        );
        MemoryPool {
            pool
        }
    }

    // pub async fn create_transaction(&self) {
    //     let mut mock_tx_factory = MockTransactionFactory::default();
    //     let transaction = mock_tx_factory.create_eip1559();
    //     self.pool.add_transaction(TransactionOrigin::Local, transaction).await;
    // }

    pub fn get_inclusion_list(
        &self,
        slot: u64,
        validator_index: usize,
        previous_block: &Block<H256>,
    ) -> Option<InclusionList> {
        if previous_block.gas_used == previous_block.gas_limit {
            return None;
        }
        let mut censored_transactions = vec![];
        let mut gas_left = previous_block.gas_limit - previous_block.gas_used;
        for transaction in self.pool.all_transactions().pending {
            if transaction.priority_fee_or_price() > 0
                && !transaction.is_eip4844()
                && gas_left > transaction.gas_limit().into()
            {
                gas_left -= transaction.gas_limit().into();
                censored_transactions.push(transaction.hash().clone());
            }
        }

        Some(InclusionList {
            slot,
            validator_index,
            transactions: censored_transactions.into()
        })

        // iterate through all pending and queued transactions in the mempool and filter for potentially censored txs
    }
}

/// A transaction validator that determines all transactions to be valid.
///
/// An actual validator impl like
/// [TransactionValidationTaskExecutor](reth_transaction_pool::TransactionValidationTaskExecutor)
/// would require up to date db access.
///
/// CAUTION: This validator is not safe to use since it doesn't actually validate the transaction's
/// properties such as chain id, balance, nonce, etc.
#[derive(Default)]
#[non_exhaustive]
struct OkValidator;

impl TransactionValidator for OkValidator {
    type Transaction = EthPooledTransaction;

    async fn validate_transaction(
        &self,
        _origin: TransactionOrigin,
        transaction: Self::Transaction,
    ) -> TransactionValidationOutcome<Self::Transaction> {
        // Always return valid
        TransactionValidationOutcome::Valid {
            balance: transaction.cost(),
            state_nonce: transaction.nonce(),
            transaction: ValidTransaction::Valid(transaction),
            propagate: false,
        }
    }
}

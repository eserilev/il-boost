use reth_transaction_pool::{blobstore::InMemoryBlobStore, validate::ValidTransaction, CoinbaseTipOrdering, EthPooledTransaction, Pool, PoolTransaction, TransactionOrigin, TransactionPool, TransactionValidationOutcome, TransactionValidator};

pub struct MemoryPool {
    pool: Pool<OkValidator, CoinbaseTipOrdering<EthPooledTransaction>, InMemoryBlobStore>,
}

impl MemoryPool {
    pub fn new() {
        let pool = reth_transaction_pool::Pool::new(
            OkValidator::default(),
            CoinbaseTipOrdering::default(),
            InMemoryBlobStore::default(),
            Default::default(),
        );
    }

    pub fn get_censored_transactions(&self) {
        let x = self.pool.all_transactions();

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
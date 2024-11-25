use narwhal::p2plane::traits::Message;
use crate::transaction::Transaction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMessage {
    pub transaction: Transaction,
    pub timestamp: u64,
}

impl TransactionMessage {
    pub fn new(transaction: Transaction) -> Self {
        Self {
            transaction,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

impl Message for TransactionMessage {
    fn protocol_id(&self) -> &'static str {
        "/narwhal/transaction/1.0.0"
    }
}

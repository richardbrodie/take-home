use crate::{Error, Transaction, TransactionType};

#[derive(Debug, Default, serde::Serialize, Clone, Copy)]
pub struct AccountState {
    client: u16,
    available: f32,
    held: f32,
    total: f32,
    locked: bool,
}

#[derive(Debug, Clone, Copy)]
struct ProcessedTransaction {
    tx: u32,
    amount: f32,
    latest_state: TransactionType,
}
impl From<&Transaction> for ProcessedTransaction {
    fn from(value: &Transaction) -> Self {
        Self {
            tx: value.tx,
            amount: value.amount.unwrap(),
            latest_state: value.kind,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Account {
    id: u16,
    balance: f32,
    held: f32,
    locked: bool,
    history: Vec<ProcessedTransaction>,
}
impl Account {
    pub fn new(id: u16) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }
    pub fn state(&self) -> AccountState {
        AccountState {
            client: self.id,
            available: self.balance - self.held,
            held: self.held,
            total: self.balance,
            locked: self.locked,
        }
    }
    fn available(&self) -> f32 {
        self.balance - self.held
    }
    fn find_transaction(&mut self, tx: u32) -> Option<&mut ProcessedTransaction> {
        self.history.iter_mut().find(|h| h.tx == tx)
    }
    pub fn process(&mut self, cur_tx: &Transaction) -> Result<(), Error> {
        match cur_tx.kind {
            TransactionType::Deposit => {
                let am = cur_tx.amount.unwrap();
                if am <= 0.0 {
                    return Err(Error::IncorrectAmount);
                }
                self.balance += am;
                self.history.push(cur_tx.into());
            }
            TransactionType::Withdrawal => {
                let am = cur_tx.amount.unwrap();
                if am <= 0.0 {
                    return Err(Error::IncorrectAmount);
                }
                if self.available() < am {
                    return Err(Error::InsufficientBalance);
                }
                self.balance -= am;
                self.history.push(cur_tx.into());
            }
            TransactionType::Dispute => {
                // find the transaction matching this tx
                let Some(hist_tx) = self.find_transaction(cur_tx.tx) else {
                    tracing::info!(
                        id = self.id,
                        tx = cur_tx.tx,
                        "dispute failed, original tx not found",
                    );
                    return Ok(());
                };

                hist_tx.latest_state = TransactionType::Dispute;
                self.held += hist_tx.amount;
            }
            TransactionType::Resolve => {
                let Some(hist_tx) = self.find_transaction(cur_tx.tx) else {
                    tracing::info!(
                        id = self.id,
                        tx = cur_tx.tx,
                        "resolve failed, original tx not found",
                    );
                    return Ok(());
                };
                if hist_tx.latest_state != TransactionType::Dispute {
                    tracing::info!(
                        id = self.id,
                        tx = cur_tx.tx,
                        "resolve failed, original tx not under dispute",
                    );
                    return Ok(());
                }

                hist_tx.latest_state = TransactionType::Resolve;
                self.held -= hist_tx.amount;
            }
            TransactionType::Chargeback => {
                let Some(hist_tx) = self.find_transaction(cur_tx.tx) else {
                    tracing::info!(
                        id = self.id,
                        tx = cur_tx.tx,
                        "chargeback failed, original tx not found",
                    );
                    return Ok(());
                };
                if hist_tx.latest_state != TransactionType::Dispute {
                    tracing::info!(
                        id = self.id,
                        tx = cur_tx.tx,
                        "chargeback failed, original tx not under dispute",
                    );
                    return Ok(());
                }
                hist_tx.latest_state = TransactionType::Chargeback;
                let am = hist_tx.amount;

                self.held -= am;
                self.balance -= am;
                self.locked = true;
            }
        }
        Ok(())
    }
}

use serde::Serializer;

use crate::{Error, Transaction, TransactionType};

fn truncate_precision<S>(v: &f32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // 4 decimal places = 10^4
    let y = (v * 10000.0).round() / 10000.0;
    serializer.serialize_f32(y)
}

#[derive(Debug, Default, serde::Serialize, Clone, Copy, PartialEq)]
pub struct AccountState {
    client: u16,
    #[serde(serialize_with = "truncate_precision")]
    available: f32,
    #[serde(serialize_with = "truncate_precision")]
    held: f32,
    #[serde(serialize_with = "truncate_precision")]
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
            amount: value.amount.unwrap_or(0.0), // TODO: is this ok?
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
                let am = cur_tx.amount.ok_or(Error::MissingAmount)?;
                if am <= 0.0 {
                    return Err(Error::IncorrectAmount);
                }
                self.balance += am;
                self.history.push(cur_tx.into());
            }
            TransactionType::Withdrawal => {
                let am = cur_tx.amount.ok_or(Error::MissingAmount)?;
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

#[cfg(test)]
mod tests {
    use crate::account::AccountState;
    use crate::{Transaction, TransactionType};

    use super::Account;

    fn transaction(t: TransactionType, a: Option<f32>, c: u16, tx: u32) -> Transaction {
        Transaction {
            kind: t,
            amount: a,
            client: c,
            tx,
        }
    }
    fn state(c: u16, a: f32, h: f32, t: f32, l: bool) -> AccountState {
        AccountState {
            client: c,
            available: a,
            held: h,
            total: t,
            locked: l,
        }
    }

    #[test]
    fn single_deposit() {
        let mut a = Account::new(0);
        let t = transaction(TransactionType::Deposit, Some(5.0), 0, 0);
        assert_eq!(a.available(), 0.0);
        let r = a.process(&t);
        assert!(r.is_ok());
        assert_eq!(a.available(), 5.0)
    }

    #[test]
    fn single_witdrawal() {
        let mut a = Account::new(0);
        let t = transaction(TransactionType::Withdrawal, Some(5.0), 0, 0);
        assert_eq!(a.available(), 0.0);
        let r = a.process(&t);
        assert!(r.is_err());
        assert_eq!(a.available(), 0.0);
    }

    #[test]
    fn deposit_then_witdrawal() {
        let mut a = Account::new(0);
        let ts = vec![
            transaction(TransactionType::Deposit, Some(5.0), 0, 0),
            transaction(TransactionType::Withdrawal, Some(3.0), 0, 0),
        ];
        for t in ts {
            let r = a.process(&t);
            assert!(r.is_ok());
        }
        assert_eq!(a.available(), 2.0);
    }

    #[test]
    fn negative_amounts() {
        let mut a = Account::new(0);
        let ts = vec![
            transaction(TransactionType::Deposit, Some(-5.0), 0, 0),
            transaction(TransactionType::Withdrawal, Some(-3.0), 0, 0),
        ];
        for t in ts {
            let r = a.process(&t);
            assert!(r.is_err());
        }
        assert_eq!(a.available(), 0.0);
    }

    #[test]
    fn dispute() {
        let mut a = Account::new(0);
        let ts = vec![
            transaction(TransactionType::Deposit, Some(5.0), 0, 0),
            transaction(TransactionType::Deposit, Some(5.0), 0, 1),
            transaction(TransactionType::Dispute, None, 0, 0),
        ];
        for t in ts {
            let r = a.process(&t);
            assert!(r.is_ok());
        }
        assert_eq!(a.state(), state(0, 5.0, 5.0, 10.0, false));
    }

    #[test]
    fn resolve() {
        let mut a = Account::new(0);
        let ts = vec![
            transaction(TransactionType::Deposit, Some(5.0), 0, 0),
            transaction(TransactionType::Deposit, Some(5.0), 0, 1),
            transaction(TransactionType::Dispute, None, 0, 0),
            transaction(TransactionType::Deposit, Some(5.0), 0, 2),
        ];
        for t in ts {
            let r = a.process(&t);
            assert!(r.is_ok());
        }
        assert_eq!(a.state(), state(0, 10.0, 5.0, 15.0, false));

        let _ = a.process(&transaction(TransactionType::Resolve, None, 0, 0));
        assert_eq!(a.state(), state(0, 15.0, 0.0, 15.0, false));
    }

    #[test]
    fn chargeback() {
        let mut a = Account::new(0);
        let ts = vec![
            transaction(TransactionType::Deposit, Some(5.0), 0, 0),
            transaction(TransactionType::Deposit, Some(5.0), 0, 1),
            transaction(TransactionType::Dispute, None, 0, 0),
            transaction(TransactionType::Deposit, Some(5.0), 0, 2),
        ];
        for t in ts {
            let r = a.process(&t);
            assert!(r.is_ok());
        }
        assert_eq!(a.state(), state(0, 10.0, 5.0, 15.0, false));

        let _ = a.process(&transaction(TransactionType::Chargeback, None, 0, 0));
        assert_eq!(a.state(), state(0, 10.0, 0.0, 10.0, true));
    }
}

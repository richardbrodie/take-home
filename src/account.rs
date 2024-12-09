use crate::{Error, Transaction, TransactionType};

#[derive(Debug, Default, serde::Serialize, Clone, Copy)]
pub struct AccountState {
    client: u16,
    available: f32,
    held: f32,
    total: f32,
    locked: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Account {
    id: u16,
    balance: f32,
    held: f32,
    locked: bool,
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
    pub fn process(&mut self, t: Transaction) -> Result<(), Error> {
        match t.kind {
            TransactionType::Deposit => {
                if t.amount <= 0.0 {
                    return Err(Error::IncorrectAmount);
                }
                self.balance += t.amount;
            }
            TransactionType::Withdrawal => {
                if t.amount <= 0.0 {
                    return Err(Error::IncorrectAmount);
                }
                if self.available() < t.amount {
                    return Err(Error::InsufficientBalance);
                }
                self.balance -= t.amount;
            }
            _ => (),
        }
        Ok(())
    }
}

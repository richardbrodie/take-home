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
}

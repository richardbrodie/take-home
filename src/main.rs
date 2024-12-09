use std::collections::HashMap;
use std::fs::File;

use csv::{Trim, Writer};

use self::account::Account;

mod account;

#[derive(Debug, serde::Deserialize)]
struct Transaction {
    #[serde(rename = "type")]
    kind: TransactionType,
    client: u16,
    tx: u32,
    amount: f32,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug)]
enum Error {
    InsufficientBalance,
    IncorrectAmount,
}

fn main() {
    // initialize account "database"
    let mut clients: HashMap<u16, Account> = HashMap::new();

    // start reading input
    let file = File::open("data/transactions.csv").unwrap();
    let mut rdr = csv::ReaderBuilder::new().trim(Trim::All).from_reader(file);
    for row in rdr.deserialize() {
        let r: Transaction = row.unwrap();
        let a = clients.entry(r.client).or_insert(Account::new(r.client));

        if let Err(e) = a.process(r) {
            // handle error
        }
    }

    // write calculated account states
    let mut wtr = Writer::from_writer(vec![]);
    for a in clients.values() {
        wtr.serialize(a.state()).unwrap();
    }
    let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
    println!("{}", data);
}

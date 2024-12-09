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

fn main() {
    let file = File::open("data/transactions.csv").unwrap();
    let mut rdr = csv::ReaderBuilder::new().trim(Trim::All).from_reader(file);
    for row in rdr.deserialize() {
        let r: Transaction = row.unwrap();
        println!(
            "|{:?}| id: {}, client: {}, amount: {}",
            r.kind, r.tx, r.client, r.amount
        );
    }

    let mut wtr = Writer::from_writer(vec![]);
    for i in 0..5 {
        let a = Account::new(i);
        wtr.serialize(a.state()).unwrap();
    }
    let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
    println!("{}", data);
}

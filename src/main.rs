use std::fs::File;

use csv::Trim;

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
}

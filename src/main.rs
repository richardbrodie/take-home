use std::collections::HashMap;
use std::env;
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
    #[serde(default)]
    amount: Option<f32>,
}

#[derive(Debug, serde::Deserialize, Clone, Copy, PartialEq)]
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
    MissingAmount,
    IncorrectArguments,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self)
    }
}
impl std::error::Error for Error {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(Box::new(Error::IncorrectArguments));
    }

    // start reading input
    let input_file = &args[1];
    let file = File::open(input_file)?;
    let mut rdr = csv::ReaderBuilder::new().trim(Trim::All).from_reader(file);

    // initialize account "database"
    let mut clients: HashMap<u16, Account> = HashMap::new();

    // start parsing transactions
    for row in rdr.deserialize() {
        let r: Transaction = row?;
        let a = clients.entry(r.client).or_insert(Account::new(r.client));

        if let Err(e) = a.process(&r) {
            tracing::error!("failed transaction: {:?}: {:?}", r, e);
        }
    }

    // write calculated account states
    let mut wtr = Writer::from_writer(std::io::stdout());
    for a in clients.values() {
        wtr.serialize(a.state())?;
    }
    wtr.flush()?;

    Ok(())
}

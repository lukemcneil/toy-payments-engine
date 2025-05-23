use std::{collections::HashMap, env, fs::File};

use anyhow::Error;
use csv::{ReaderBuilder, Trim};

type ClientID = u16;
type TxID = u32;

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, serde::Deserialize, Clone)]
struct Transaction {
    #[serde(rename = "type")]
    transaction_type: TransactionType,
    client: ClientID,
    tx: TxID,
    amount: f32,
}

#[derive(Debug)]
struct DepositInfo {
    amount: f32,
    disputed: bool,
}

#[derive(Debug)]
struct ClientInfo {
    available: f32,
    held: f32,
    locked: bool,
    deposits: HashMap<TxID, DepositInfo>,
}

#[derive(Debug)]
struct Clients {
    client_data: HashMap<ClientID, ClientInfo>,
}

impl Clients {
    fn process_transaction(&mut self, tx: Transaction) -> Result<(), Error> {
        match tx.transaction_type {
            TransactionType::Deposit => match self.client_data.get_mut(&tx.client) {
                Some(client_info) => {
                    client_info.available += tx.amount;
                    client_info.deposits.insert(
                        tx.tx,
                        DepositInfo {
                            amount: tx.amount,
                            disputed: false,
                        },
                    );
                }
                None => {
                    let mut deposits = HashMap::new();
                    deposits.insert(
                        tx.tx,
                        DepositInfo {
                            amount: tx.amount,
                            disputed: false,
                        },
                    );
                    self.client_data.insert(
                        tx.client,
                        ClientInfo {
                            available: tx.amount,
                            held: 0.0,
                            locked: false,
                            deposits,
                        },
                    );
                }
            },
            TransactionType::Withdrawal => (),
            TransactionType::Dispute => todo!(),
            TransactionType::Resolve => todo!(),
            TransactionType::Chargeback => todo!(),
        };
        Ok(())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        0 | 1 => panic!("please include an input file name"),
        2 => (),
        _ => panic!("please just include 1 argument"),
    }
    let input_filename = &args[1];
    match process_transactions(input_filename) {
        Ok(()) => (),
        Err(e) => panic!("{}", e),
    }
}

fn process_transactions(input_filename: &str) -> Result<(), Error> {
    let transactions = get_transactions_from_csv(input_filename)?;
    let mut clients = Clients {
        client_data: HashMap::new(),
    };
    for tx in transactions {
        clients.process_transaction(tx)?
    }
    println!("{:#?}", clients);
    Ok(())
}

fn get_transactions_from_csv(filename: &str) -> Result<Vec<Transaction>, Error> {
    let file = File::open(filename)?;
    let mut rdr = ReaderBuilder::new().trim(Trim::All).from_reader(file);

    Ok(rdr.deserialize().collect::<Result<Vec<Transaction>, _>>()?)
}

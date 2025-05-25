use std::{collections::HashMap, env, fs::File, io};

use anyhow::{anyhow, Error};
use csv::{ReaderBuilder, Trim};
use rust_decimal::Decimal;

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
    amount: Option<Decimal>,
}

#[derive(Debug)]
struct DepositInfo {
    amount: Decimal,
    disputed: bool,
}

#[derive(Debug)]
struct ClientInfo {
    available: Decimal,
    held: Decimal,
    locked: bool,
    deposits: HashMap<TxID, DepositInfo>,
}

#[derive(Debug, serde::Serialize)]
struct ClientOutput {
    client: ClientID,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

impl ClientOutput {
    fn new(client_id: ClientID, client_info: &ClientInfo) -> Self {
        Self {
            client: client_id,
            available: client_info.available,
            held: client_info.held,
            total: client_info.available + client_info.held,
            locked: client_info.locked,
        }
    }
}

#[derive(Debug)]
struct Clients {
    client_data: HashMap<ClientID, ClientInfo>,
}

impl Clients {
    fn process_transaction(&mut self, tx: &Transaction) -> Result<(), Error> {
        let maybe_client = self.client_data.get_mut(&tx.client);
        if let Some(client_info) = &maybe_client {
            if client_info.locked {
                return Err(anyhow!(
                    "Cannot execute transaction because client is locked"
                ));
            }
        }
        match tx.transaction_type {
            TransactionType::Deposit => {
                let amount = match tx.amount {
                    Some(amount) => amount,
                    None => return Err(anyhow!("Need to include an amount with deposit")),
                };
                let deposit_info = DepositInfo {
                    amount: amount,
                    disputed: false,
                };
                match maybe_client {
                    Some(client_info) => {
                        client_info.available += amount;
                        client_info.deposits.insert(tx.tx, deposit_info);
                    }
                    None => {
                        let mut deposits = HashMap::new();
                        deposits.insert(tx.tx, deposit_info);
                        self.client_data.insert(
                            tx.client,
                            ClientInfo {
                                available: amount,
                                held: Decimal::ZERO,
                                locked: false,
                                deposits,
                            },
                        );
                    }
                }
            }
            TransactionType::Withdrawal => {
                if let Some(client_info) = maybe_client {
                    let amount = match tx.amount {
                        Some(amount) => amount,
                        None => return Err(anyhow!("Need to include an amount with deposit")),
                    };
                    if client_info.available >= amount {
                        client_info.available -= amount;
                    } else {
                        return Err(anyhow!("Cannot withdraw more than available in account"));
                    }
                } else {
                    return Err(anyhow!("Client does not exist"));
                }
            }
            TransactionType::Dispute => {
                if let Some(client_info) = maybe_client {
                    if let Some(deposit_info) = client_info.deposits.get_mut(&tx.tx) {
                        if !deposit_info.disputed {
                            client_info.held += deposit_info.amount;
                            client_info.available -= deposit_info.amount;
                            deposit_info.disputed = true;
                        } else {
                            return Err(anyhow!("Deposit is already being disputed"));
                        }
                    } else {
                        return Err(anyhow!("Deposit transaction does not exist"));
                    }
                } else {
                    return Err(anyhow!("Client does not exist"));
                }
            }
            TransactionType::Resolve => {
                if let Some(client_info) = maybe_client {
                    if let Some(deposit_info) = client_info.deposits.get_mut(&tx.tx) {
                        if deposit_info.disputed {
                            client_info.held -= deposit_info.amount;
                            client_info.available += deposit_info.amount;
                            deposit_info.disputed = false;
                        } else {
                            return Err(anyhow!("Deposit is not being disputed"));
                        }
                    } else {
                        return Err(anyhow!("Deposit transaction does not exist"));
                    }
                } else {
                    return Err(anyhow!("Client does not exist"));
                }
            }
            TransactionType::Chargeback => {
                if let Some(client_info) = maybe_client {
                    if let Some(deposit_info) = client_info.deposits.get_mut(&tx.tx) {
                        if deposit_info.disputed {
                            client_info.held -= deposit_info.amount;
                            deposit_info.disputed = false;
                            client_info.locked = true;
                        } else {
                            return Err(anyhow!("Deposit is not being disputed"));
                        }
                    } else {
                        return Err(anyhow!("Deposit transaction does not exist"));
                    }
                } else {
                    return Err(anyhow!("Client does not exist"));
                }
            }
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
    for tx in &transactions {
        if let Err(_e) = clients.process_transaction(tx) {
            // silently ignore errors in processing transactions for now
            // println!("Error occurred running transaction {:?}: {}", tx, _e);
        }
    }
    // println!("{:#?}", clients);
    print_client_info(&clients)?;
    Ok(())
}

fn print_client_info(clients: &Clients) -> Result<(), Error> {
    let mut wtr = csv::Writer::from_writer(io::stdout());
    for (client_id, client_info) in &clients.client_data {
        let client_output = ClientOutput::new(*client_id, client_info);
        wtr.serialize(client_output)?;
    }
    wtr.flush()?;
    Ok(())
}

fn get_transactions_from_csv(filename: &str) -> Result<Vec<Transaction>, Error> {
    let file = File::open(filename)?;
    let mut rdr = ReaderBuilder::new().trim(Trim::All).from_reader(file);

    Ok(rdr.deserialize().collect::<Result<Vec<Transaction>, _>>()?)
}

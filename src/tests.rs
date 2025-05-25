#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use rust_decimal::dec;

    use crate::*;

    #[test]
    fn test_running_executable() {
        let mut cmd = Command::cargo_bin("toy-payments-engine").unwrap();
        cmd.arg("transactions.csv");
        cmd.assert().success();
        let output = String::from_utf8(cmd.output().unwrap().stdout).unwrap();
        assert_eq!(output.lines().count(), 3);
        assert!(output.contains("client,available,held,total,locked"));
        assert!(output.contains("1,1.5,0,1.5,false"));
        assert!(output.contains("2,2,0,2,false"));
    }

    #[test]
    fn test_deposit() {
        let client_data = HashMap::new();
        let mut clients = Clients { client_data };
        clients
            .process_transaction(&Transaction {
                transaction_type: TransactionType::Deposit,
                client: 0,
                tx: 0,
                amount: Some(dec!(100)),
            })
            .unwrap();
        let client_info = clients.client_data.get(&0).unwrap();
        assert_eq!(client_info.available, dec!(100));
        assert_eq!(client_info.held, dec!(0));
        assert_eq!(client_info.locked, false);
    }

    #[test]
    fn test_withdraw() {
        let mut client_data = HashMap::new();
        client_data.insert(
            0,
            ClientInfo {
                available: dec!(100),
                held: dec!(0),
                locked: false,
                deposits: HashMap::new(),
            },
        );
        let mut clients = Clients { client_data };
        clients
            .process_transaction(&Transaction {
                transaction_type: TransactionType::Withdrawal,
                client: 0,
                tx: 0,
                amount: Some(dec!(30)),
            })
            .unwrap();
        let client_info = clients.client_data.get(&0).unwrap();
        assert_eq!(client_info.available, dec!(70));
        assert_eq!(client_info.held, dec!(0));
        assert_eq!(client_info.locked, false);
    }

    #[test]
    fn test_dispute() {
        let mut client_data = HashMap::new();
        let mut deposits = HashMap::new();
        deposits.insert(
            0,
            DepositInfo {
                amount: dec!(100),
                disputed: false,
            },
        );
        client_data.insert(
            0,
            ClientInfo {
                available: dec!(100),
                held: dec!(0),
                locked: false,
                deposits,
            },
        );
        let mut clients = Clients { client_data };
        clients
            .process_transaction(&Transaction {
                transaction_type: TransactionType::Dispute,
                client: 0,
                tx: 0,
                amount: None,
            })
            .unwrap();
        let client_info = clients.client_data.get(&0).unwrap();
        assert_eq!(client_info.available, dec!(0));
        assert_eq!(client_info.held, dec!(100));
        assert!(client_info.deposits.get(&0).unwrap().disputed);
        assert_eq!(client_info.locked, false);
    }

    #[test]
    fn test_resolve() {
        let mut client_data = HashMap::new();
        let mut deposits = HashMap::new();
        deposits.insert(
            0,
            DepositInfo {
                amount: dec!(100),
                disputed: true,
            },
        );
        client_data.insert(
            0,
            ClientInfo {
                available: dec!(0),
                held: dec!(100),
                locked: false,
                deposits,
            },
        );
        let mut clients = Clients { client_data };
        clients
            .process_transaction(&Transaction {
                transaction_type: TransactionType::Resolve,
                client: 0,
                tx: 0,
                amount: None,
            })
            .unwrap();
        let client_info = clients.client_data.get(&0).unwrap();
        assert_eq!(client_info.available, dec!(100));
        assert_eq!(client_info.held, dec!(0));
        assert!(!client_info.deposits.get(&0).unwrap().disputed);
        assert_eq!(client_info.locked, false);
    }

    #[test]
    fn test_chargeback() {
        let mut client_data = HashMap::new();
        let mut deposits = HashMap::new();
        deposits.insert(
            0,
            DepositInfo {
                amount: dec!(100),
                disputed: true,
            },
        );
        client_data.insert(
            0,
            ClientInfo {
                available: dec!(0),
                held: dec!(100),
                locked: false,
                deposits,
            },
        );
        let mut clients = Clients { client_data };
        clients
            .process_transaction(&Transaction {
                transaction_type: TransactionType::Chargeback,
                client: 0,
                tx: 0,
                amount: None,
            })
            .unwrap();
        let client_info = clients.client_data.get(&0).unwrap();
        assert_eq!(client_info.available, dec!(0));
        assert_eq!(client_info.held, dec!(0));
        assert!(!client_info.deposits.get(&0).unwrap().disputed);
        assert!(client_info.locked);
    }
}

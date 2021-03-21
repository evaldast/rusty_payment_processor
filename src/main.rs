use std::env;
use std::error::Error;
use std::io;

use rust_test::processor::PaymentProcessor;

fn main() -> Result<(), Box<dyn Error>> {
    let mut reader = csv::Reader::from_path(parse_input_path_argument())?;
    let mut processor = PaymentProcessor::new();

    for result in reader.deserialize() {
        match result {
            Ok(transaction) => {
                processor.process(transaction);
            }
            Err(e) => eprintln!("Deserialization error occured: {}", e),
        }
    }

    let mut writer = csv::Writer::from_writer(io::stdout());
    
    processor
        .get_accounts()
        .iter()
        .map(|(_, account)| account)
        .try_for_each(|account| writer.serialize(account))?;

    Ok(())
}

fn parse_input_path_argument() -> String {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        panic!("No arguments provided");
    }

    args[1].clone()
}

#[cfg(test)]
mod tests {
    use rust_test::{
        processor::PaymentProcessor, transaction::Transaction, transaction::TransactionType,
    };

    #[test]
    fn creates_account_on_transaction() {
        let mut processor = PaymentProcessor::new();
        let client_id = 5;

        let transaction = Transaction {
            client_id,
            amount: Some(2.25),
            transaction_type: TransactionType::Deposit,
            tx_id: 3,
        };

        processor.process(transaction);

        assert_eq!(processor.get_accounts().contains_key(&client_id), true)
    }

    #[test]
    fn rejects_negative_amount_transaction() {
        let mut processor = PaymentProcessor::new();
        let client_id = 5;

        let transaction = Transaction {
            client_id,
            amount: Some(-25.0),
            transaction_type: TransactionType::Deposit,
            tx_id: 3,
        };

        processor.process(transaction);

        let accounts = processor.get_accounts();
        let account = accounts.get(&client_id).unwrap();

        assert_eq!(account.get_total(), 0.0);
        assert_eq!(account.get_available(), 0.0);
        assert_eq!(account.get_held(), 0.0);
        assert_eq!(account.is_locked(), false);
    }

    #[test]
    fn can_deposit() {
        let mut processor = PaymentProcessor::new();
        let client_id = 10;
        let amount = 22.5;

        let transaction = Transaction {
            client_id,
            amount: Some(amount),
            transaction_type: TransactionType::Deposit,
            tx_id: 5,
        };

        processor.process(transaction);

        let accounts = processor.get_accounts();
        let account = accounts.get(&client_id).unwrap();

        assert_eq!(account.get_total(), amount);
        assert_eq!(account.get_available(), amount);
        assert_eq!(account.get_held(), 0.0);
        assert_eq!(account.is_locked(), false);
    }

    #[test]
    fn can_withdraw() {
        let mut processor = PaymentProcessor::new();
        let client_id = 11;
        let amount_deposit = 15.5;
        let amount_withdraw = 10.0;

        let transaction_deposit = Transaction {
            client_id,
            amount: Some(amount_deposit),
            transaction_type: TransactionType::Deposit,
            tx_id: 6,
        };

        processor.process(transaction_deposit);

        let transaction_withdraw = Transaction {
            client_id,
            amount: Some(amount_withdraw),
            transaction_type: TransactionType::Withdrawal,
            tx_id: 7,
        };

        processor.process(transaction_withdraw);

        let accounts = processor.get_accounts();
        let account = accounts.get(&client_id).unwrap();

        assert_eq!(account.get_total(), amount_deposit - amount_withdraw);
        assert_eq!(account.get_available(), amount_deposit - amount_withdraw);
        assert_eq!(account.get_held(), 0.0);
        assert_eq!(account.is_locked(), false);
    }

    #[test]
    fn cannot_withdraw_higher_amount_than_available() {
        let mut processor = PaymentProcessor::new();
        let client_id = 11;
        let amount_deposit = 15.5;
        let amount_withdraw = 16.0;

        let transaction_deposit = Transaction {
            client_id,
            amount: Some(amount_deposit),
            transaction_type: TransactionType::Deposit,
            tx_id: 6,
        };

        processor.process(transaction_deposit);

        let transaction_withdraw = Transaction {
            client_id,
            amount: Some(amount_withdraw),
            transaction_type: TransactionType::Withdrawal,
            tx_id: 7,
        };

        processor.process(transaction_withdraw);

        let accounts = processor.get_accounts();
        let account = accounts.get(&client_id).unwrap();

        assert_eq!(amount_withdraw > amount_deposit, true);
        assert_eq!(account.get_total(), amount_deposit);
        assert_eq!(account.get_available(), amount_deposit);
        assert_eq!(account.get_held(), 0.0);
        assert_eq!(account.is_locked(), false);
    }

    #[test]
    fn can_dispute() {
        let mut processor = PaymentProcessor::new();
        let client_id = 11;
        let amount_deposit = 25.5;
        let amount_withdraw = 10.0;
        let withdraw_tx_id = 7;

        let transaction_deposit = Transaction {
            client_id,
            amount: Some(amount_deposit),
            transaction_type: TransactionType::Deposit,
            tx_id: 6,
        };

        processor.process(transaction_deposit);

        let transaction_withdraw = Transaction {
            client_id,
            amount: Some(amount_withdraw),
            transaction_type: TransactionType::Withdrawal,
            tx_id: withdraw_tx_id,
        };

        processor.process(transaction_withdraw);

        let transaction_dispute = Transaction {
            client_id,
            amount: None,
            transaction_type: TransactionType::Dispute,
            tx_id: withdraw_tx_id,
        };

        processor.process(transaction_dispute);

        let accounts = processor.get_accounts();
        let account = accounts.get(&client_id).unwrap();

        assert_eq!(account.get_total(), amount_deposit - amount_withdraw);
        assert_eq!(account.get_available(), amount_deposit - amount_withdraw);
        assert_eq!(account.get_held(), amount_withdraw);
        assert_eq!(account.is_locked(), false);
    }

    #[test]
    fn can_resolve() {
        let mut processor = PaymentProcessor::new();
        let client_id = 11;
        let amount_deposit = 25.5;
        let amount_withdraw = 10.0;
        let withdraw_tx_id = 7;

        let transaction_deposit = Transaction {
            client_id,
            amount: Some(amount_deposit),
            transaction_type: TransactionType::Deposit,
            tx_id: 6,
        };

        processor.process(transaction_deposit);

        let transaction_withdraw = Transaction {
            client_id,
            amount: Some(amount_withdraw),
            transaction_type: TransactionType::Withdrawal,
            tx_id: withdraw_tx_id,
        };

        processor.process(transaction_withdraw);

        let transaction_dispute = Transaction {
            client_id,
            amount: None,
            transaction_type: TransactionType::Dispute,
            tx_id: withdraw_tx_id,
        };

        processor.process(transaction_dispute);

        let transaction_resolve = Transaction {
            client_id,
            amount: None,
            transaction_type: TransactionType::Resolve,
            tx_id: withdraw_tx_id,
        };

        processor.process(transaction_resolve);

        let accounts = processor.get_accounts();
        let account = accounts.get(&client_id).unwrap();

        assert_eq!(account.get_total(), amount_deposit);
        assert_eq!(account.get_available(), amount_deposit);
        assert_eq!(account.get_held(), 0.0);
        assert_eq!(account.is_locked(), false);
    }

    #[test]
    fn can_chargeback() {
        let mut processor = PaymentProcessor::new();
        let client_id = 11;
        let amount_deposit = 25.5;
        let deposit_tx_id = 7;

        let transaction_deposit = Transaction {
            client_id,
            amount: Some(amount_deposit),
            transaction_type: TransactionType::Deposit,
            tx_id: deposit_tx_id,
        };

        processor.process(transaction_deposit);

        let transaction_dispute = Transaction {
            client_id,
            amount: None,
            transaction_type: TransactionType::Dispute,
            tx_id: deposit_tx_id,
        };

        processor.process(transaction_dispute);

        let transaction_chargeback = Transaction {
            client_id,
            amount: None,
            transaction_type: TransactionType::Chargeback,
            tx_id: deposit_tx_id,
        };

        processor.process(transaction_chargeback);

        let accounts = processor.get_accounts();
        let account = accounts.get(&client_id).unwrap();

        assert_eq!(account.get_total(), 0.0);
        assert_eq!(account.get_available(), 0.0);
        assert_eq!(account.get_held(), 0.0);
        assert_eq!(account.is_locked(), true);
    }

    #[test]
    fn cannot_chargeback_withdraw() {
        let mut processor = PaymentProcessor::new();
        let client_id = 11;
        let amount_deposit = 25.5;
        let amount_withdraw = 12.25;
        let withdraw_tx_id = 7;

        let transaction_deposit = Transaction {
            client_id,
            amount: Some(amount_deposit),
            transaction_type: TransactionType::Deposit,
            tx_id: 6,
        };

        processor.process(transaction_deposit);

        let transaction_withdraw = Transaction {
            client_id,
            amount: Some(amount_withdraw),
            transaction_type: TransactionType::Withdrawal,
            tx_id: withdraw_tx_id,
        };

        processor.process(transaction_withdraw);

        let transaction_dispute = Transaction {
            client_id,
            amount: None,
            transaction_type: TransactionType::Dispute,
            tx_id: withdraw_tx_id,
        };

        processor.process(transaction_dispute);

        let transaction_chargeback = Transaction {
            client_id,
            amount: None,
            transaction_type: TransactionType::Chargeback,
            tx_id: withdraw_tx_id,
        };

        processor.process(transaction_chargeback);

        let accounts = processor.get_accounts();
        let account = accounts.get(&client_id).unwrap();

        assert_eq!(account.get_total(), amount_deposit - amount_withdraw);
        assert_eq!(account.get_available(), amount_deposit - amount_withdraw);
        assert_eq!(account.get_held(), amount_withdraw);
        assert_eq!(account.is_locked(), false);
    }
}

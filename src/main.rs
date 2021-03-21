use std::env;
use std::error::Error;

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

    processor.print_accounts();

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
    use rust_test::{processor::PaymentProcessor, transaction::Transaction, transaction::TransactionType};

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
}

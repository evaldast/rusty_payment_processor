use std::collections::HashMap;

use crate::account::Account;
use crate::transaction::Transaction;

pub struct PaymentProcessor {
    accounts: HashMap<u16, Account>
}

impl PaymentProcessor {
    pub fn new() -> PaymentProcessor {
        PaymentProcessor {
            accounts: HashMap::new()
        }
    }
    
    pub fn process(&mut self, transaction: Transaction) {
        match self.accounts.get_mut(&transaction.client_id) {
            Some(account) => match account.handle(transaction) {
                Ok(_) => {}
                Err(e) => eprintln!("Transaction error occured: {}", e),
            },
            None => {
                let client_id = transaction.client_id;
                let mut account = Account::new(client_id);

                match account.handle(transaction) {
                    Ok(_) => {}
                    Err(e) => eprintln!("Transaction error occured: {}", e),
                }

                self.accounts.insert(client_id, account);
            }
        };
    }

    pub fn print_accounts(&self) {
        for (_, account) in &self.accounts {
            println!("{}", account);
        }
    }

    pub fn get_accounts(&self) -> &HashMap<u16, Account> {
        &self.accounts
    }
}


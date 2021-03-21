use std::collections::HashMap;
use std::fmt;

use crate::transaction::{Transaction, TransactionType};

#[derive(Debug)]
pub enum OperationError {
    InsufficientBalance(u16, u32),
    InvalidData,
    TransactionNotFound(u16, u32),
    DisputeAlreadyUnderDispute(u16, u32),
    ResolveNotUnderDispute(u16, u32),
    ChargebackNotUnderDispute(u16, u32),
}

pub struct Account {
    client_id: u16,
    held: u64,
    total: u64,
    locked: bool,
    transactions: HashMap<u32, (bool, Transaction)>,
}

impl Account {
    pub fn handle(&mut self, transaction: Transaction) -> Result<&Account, OperationError> {
        match transaction.transaction_type {
            TransactionType::Deposit => self.deposit(transaction),
            TransactionType::Withdrawal => self.withdraw(transaction),
            TransactionType::Dispute => self.dispute(transaction),
            TransactionType::Resolve => self.resolve(transaction),
            TransactionType::Chargeback => self.chargeback(transaction),
        }
    }

    pub fn new(client_id: u16) -> Account {
        Account {
            client_id,
            held: 0,
            total: 0,
            locked: false,
            transactions: HashMap::new(),
        }
    }

    fn deposit(&mut self, transaction: Transaction) -> Result<&Account, OperationError> {
        match transaction.amount {
            Some(amount) => {
                self.total += get_amount_in_cent_parts(amount);

                self.transactions
                    .insert(transaction.tx_id, (false, transaction));

                Ok(self)
            }
            None => Err(OperationError::InvalidData),
        }
    }

    fn withdraw(&mut self, transaction: Transaction) -> Result<&Account, OperationError> {
        match transaction.amount {
            Some(amount) => {
                let amount_to_withdraw = get_amount_in_cent_parts(amount);

                if self.total < amount_to_withdraw {
                    return Err(OperationError::InsufficientBalance(transaction.client_id, transaction.tx_id));
                }

                self.total -= amount_to_withdraw;

                self.transactions
                    .insert(transaction.tx_id, (false, transaction));

                Ok(self)
            }
            None => Err(OperationError::InvalidData),
        }
    }

    fn dispute(&mut self, transaction: Transaction) -> Result<&Account, OperationError> {
        match self.transactions.get_mut(&transaction.tx_id) {
            Some((under_dispute, transaction)) => {
                if *under_dispute {
                    return Err(OperationError::DisputeAlreadyUnderDispute(transaction.client_id, transaction.tx_id));
                }

                *under_dispute = true;

                let amount_to_dispute = get_amount_in_cent_parts(transaction.amount.unwrap());

                self.held += amount_to_dispute;
            }
            None => return Err(OperationError::TransactionNotFound(transaction.client_id, transaction.tx_id)),
        }

        Ok(self)
    }

    fn resolve(&mut self, transaction: Transaction) -> Result<&Account, OperationError> {
        match self.transactions.get_mut(&transaction.tx_id) {
            Some((under_dispute, transaction)) => {
                if !*under_dispute {
                    return Err(OperationError::ResolveNotUnderDispute(transaction.client_id, transaction.tx_id));
                }

                *under_dispute = false;

                let amount_to_resolve = get_amount_in_cent_parts(transaction.amount.unwrap());

                self.held -= amount_to_resolve;
                self.total += amount_to_resolve;
            }
            None => return Err(OperationError::TransactionNotFound(transaction.client_id, transaction.tx_id)),
        }

        Ok(self)
    }

    fn chargeback(&mut self, transaction: Transaction) -> Result<&Account, OperationError> {
        match self.transactions.get(&transaction.tx_id) {
            Some((under_dispute, transaction)) => {
                if !*under_dispute {
                    return Err(OperationError::ChargebackNotUnderDispute(transaction.client_id, transaction.tx_id));
                }

                let amount_to_chargeback = get_amount_in_cent_parts(transaction.amount.unwrap());

                self.held -= amount_to_chargeback;
                self.total -= amount_to_chargeback;
                self.locked = true;
            }
            None => return Err(OperationError::TransactionNotFound(transaction.client_id, transaction.tx_id)),
        }

        Ok(self)
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{},{},{},{},{}",
            self.client_id,
            self.total / 10000 - self.held / 10000,
            self.held / 10000,
            (self.total as f32) / 10000.0,
            self.locked
        )
    }
}

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OperationError::InsufficientBalance(client_id, tx_id) => {
                write!(f, "Client {} Balance too low for withdraw operation {}", client_id, tx_id)
            }
            OperationError::TransactionNotFound(client_id, tx_id) => {
                write!(f, "Client {} No transaction found for dispute {}", client_id, tx_id)
            }
            OperationError::DisputeAlreadyUnderDispute(client_id, tx_id) => {
                write!(f, "Client {} Transaction is already under dispute for dispute {}", client_id, tx_id)
            }
            OperationError::ResolveNotUnderDispute(client_id, tx_id) => {
                write!(f, "Client {} Transaction is not under dispute for resolve {}", client_id, tx_id)
            }
            OperationError::ChargebackNotUnderDispute(client_id, tx_id) => {
                write!(f, "Client {} Transaction is not under dispute for chargeback {}", client_id, tx_id)
            }
            OperationError::InvalidData => {
                write!(f, "Invalid operation data")
            }
        }
    }
}

fn get_amount_in_cent_parts(amount: f32) -> u64 {
    (amount * 10000.0).round() as u64
}

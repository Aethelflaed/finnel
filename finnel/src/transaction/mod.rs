use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

const TRANSFER_DEBIT: &str = "Transfer debit";
const TRANSFER_CREDIT: &str = "Transfer credit";
const DEBIT: &str = "Debit";
const ATM_DEBIT: &str = "ATM debit";

pub enum Type {
    TransferDebit,
    TransferCredit,
    Debit,
    AtmDebit,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Type::TransferDebit => f.write_str(TRANSFER_DEBIT),
            Type::TransferCredit => f.write_str(TRANSFER_CREDIT),
            Type::Debit => f.write_str(DEBIT),
            Type::AtmDebit => f.write_str(ATM_DEBIT),
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub struct ParseTypeError;

impl Display for ParseTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Parse Type Error")
    }
}

impl FromStr for Type {
    type Err = ParseTypeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            TRANSFER_DEBIT => Ok(Type::TransferDebit),
            TRANSFER_CREDIT => Ok(Type::TransferCredit),
            DEBIT => Ok(Type::Debit),
            ATM_DEBIT => Ok(Type::AtmDebit),
            _ => Err(ParseTypeError),
        }
    }
}

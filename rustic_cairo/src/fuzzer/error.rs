use serde::{Deserialize, Serialize};
use strum::{Display, EnumVariantNames};

#[derive(Debug, Clone, EnumVariantNames, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Error {
    Abort { message: String },
    OutOfBound { message: String },
    OutOfGas { message: String },
    ArithmeticError { message: String },
    MemoryLimitExceeded { message: String },
    Unknown { message: String },
    // TODO Add more errors
}

use std::fmt;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Abort { message } => write!(f, "Abort: {}", message),
            Error::OutOfBound { message } => write!(f, "OutOfBound: {}", message),
            Error::OutOfGas { message } => write!(f, "OutOfGas: {}", message),
            Error::ArithmeticError { message } => write!(f, "ArithmeticError: {}", message),
            Error::MemoryLimitExceeded { message } => write!(f, "MemoryLimitExceeded: {}", message),
            Error::Unknown { message } => write!(f, "Unknown: {}", message),
            // TODO Add more errors
        }
    }
}

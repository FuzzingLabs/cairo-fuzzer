use super::error::Error;
use crate::mutator::types::Type;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, Clone, Debug, Eq)]
pub struct Crash {
    pub target_module: String,
    pub target_function: String,
    pub inputs: Vec<Type>,
    pub error: Error,
}

impl Crash {
    pub fn new(
        target_module: &str,
        target_function: &str,
        inputs: &Vec<Type>,
        error: &Error,
    ) -> Self {
        Self {
            target_module: target_module.to_string(),
            target_function: target_function.to_string(),
            inputs: inputs.clone(),
            error: error.clone(),
        }
    }
}

impl Hash for Crash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.error.hash(state);
        self.target_module.hash(state);
        self.target_function.hash(state);
    }
}

impl PartialEq for Crash {
    fn eq(&self, other: &Self) -> bool {
        self.error == other.error
            && self.target_module == other.target_module
            && self.target_function == other.target_function
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputCorpus {
    pub name: String,
    pub args: Vec<String>,
    pub inputs: Vec<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrashCorpus {
    pub name: String,
    pub args: Vec<String>,
    pub crashes: Vec<Vec<u8>>,
}
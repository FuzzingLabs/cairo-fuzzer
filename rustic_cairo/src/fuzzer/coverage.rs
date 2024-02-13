use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};

use serde::{Serialize, Deserialize};

use crate::mutator::types::Type;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coverage {
    pub inputs: Vec<Type>,
    pub data: Vec<CoverageData>
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct CoverageData {
    pub pc_ap: (u32, u32),
}

impl Hash for Coverage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl PartialEq for Coverage {
    fn eq(&self, other: &Self) -> bool {
        let mut state = DefaultHasher::new();
        self.data.hash(&mut state) == other.data.hash(&mut state)
    }
}

impl Eq for Coverage {}

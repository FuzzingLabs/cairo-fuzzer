use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use cairo_vm::types::relocatable::Relocatable;
use serde::{Deserialize, Serialize};

use crate::mutator::types::Type;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coverage {
    pub failure: bool,
    pub inputs: Vec<Type>,
    pub data: Vec<(Relocatable, usize)>,
}

/* #[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct CoverageData {
    pub pc_ap: (u32, u32),
} */

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

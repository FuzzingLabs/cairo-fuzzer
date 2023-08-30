use std::collections::hash_map::DefaultHasher;
use std::hash::{self, Hash, Hasher};

pub fn hash_vector<T: Hash>(vector: &[T]) -> u64 {
    let mut hasher = DefaultHasher::new();
    vector.hash(&mut hasher);
    hasher.finish()
}

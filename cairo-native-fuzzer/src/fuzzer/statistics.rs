use std::time::Instant;

/// The fuzzer statistics
pub struct FuzzerStats {
    pub total_executions: usize,
    pub start_time: Instant,
    pub crashes: usize,
}

impl Default for FuzzerStats {
    fn default() -> Self {
        Self {
            total_executions: 0,
            crashes: 0,
            start_time: Instant::now(),
        }
    }
}

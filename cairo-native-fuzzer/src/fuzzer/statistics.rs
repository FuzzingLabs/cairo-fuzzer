use std::time::Instant;

/// Cairo Fuzzer statistics
pub struct FuzzerStats {
    // Total fuzzer executions
    pub total_executions: usize,
    // Start time of the fuzzer
    pub start_time: Instant,
    // Total number of crashes
    pub crashes: usize,
}

impl Default for FuzzerStats {
    fn default() -> Self {
        Self {
            // Init the fuzzer statistics
            total_executions: 0,
            crashes: 0,
            start_time: Instant::now(),
        }
    }
}

pub struct Stats {
    pub crashes: u64,
    pub unique_crashes: u64,
    pub execs: u64,
    pub time_running: u64,
    pub execs_per_sec: u64,
    pub coverage_size: u64,
    pub secs_since_last_cov: u64,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            crashes: 0,
            unique_crashes: 0,
            time_running: 0,
            execs: 0,
            coverage_size: 0,
            execs_per_sec: 0,
            secs_since_last_cov: 0,
        }
    }
}

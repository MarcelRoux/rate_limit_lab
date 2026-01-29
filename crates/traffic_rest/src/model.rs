use std::time::Duration;

#[derive(Clone, Debug)]
pub struct TrafficProfile {
    pub target_url: String,
    pub duration: Duration,
    pub requests_per_second: u64,
    pub concurrency: usize,
    /// Header name expected by your M2.3 adapter (e.g., "x-api-key").
    pub key_header: String,
}

#[derive(Clone, Debug)]
pub enum KeyMode {
    Keyless,
    SingleKey(String),
    RoundRobin(Vec<String>),
}

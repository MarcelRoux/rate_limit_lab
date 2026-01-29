use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use reqwest::Client;
use tokio::{sync::Semaphore, task::JoinSet};

use crate::{
    client::{Observation, send_request},
    model::{KeyMode, TrafficProfile},
};

struct KeySelector {
    mode: KeyMode,
    idx: usize,
}

impl KeySelector {
    fn new(mode: KeyMode) -> Self {
        Self { mode, idx: 0 }
    }

    fn next_key(&mut self) -> Option<&str> {
        match &self.mode {
            KeyMode::Keyless => None,
            KeyMode::SingleKey(k) => Some(k.as_str()),
            KeyMode::RoundRobin(keys) => {
                if keys.is_empty() {
                    return None;
                }
                let k = keys[self.idx % keys.len()].as_str();
                self.idx = self.idx.wrapping_add(1);
                Some(k)
            }
        }
    }
}

pub async fn run_profile(profile: TrafficProfile, key_mode: KeyMode) -> Vec<Observation> {
    let client = Client::new();
    let sem = Arc::new(Semaphore::new(profile.concurrency.max(1)));

    let mut selector = KeySelector::new(key_mode);
    let mut joinset: JoinSet<Option<Observation>> = JoinSet::new();

    // Batch-per-tick pacing: tick at 1ms and issue N requests per tick.
    // This avoids relying on sub-millisecond timer granularity at high RPS.
    let tick = Duration::from_millis(1);
    let ticks_per_sec: u64 = 1_000;

    let rps = profile.requests_per_second;
    let base_per_tick: u64 = if rps == 0 { 0 } else { rps / ticks_per_sec };
    let remainder: u64 = if rps == 0 { 0 } else { rps % ticks_per_sec };
    let mut remainder_acc: u64 = 0;

    let mut ticker = tokio::time::interval(tick);

    let started = Instant::now();
    let deadline = started + profile.duration;

    while Instant::now() < deadline {
        ticker.tick().await;

        // Compute how many requests to issue this tick.
        // Distribute the remainder across ticks to match the requested RPS over time.
        let mut to_send = base_per_tick;
        remainder_acc += remainder;
        if remainder_acc >= ticks_per_sec {
            to_send += 1;
            remainder_acc -= ticks_per_sec;
        }

        for _ in 0..to_send {
            // Concurrency bound
            let permit = match sem.clone().acquire_owned().await {
                Ok(p) => p,
                Err(_) => break,
            };

            let url = profile.target_url.clone();
            let key_header = profile.key_header.clone();
            let key = selector.next_key().map(|s| s.to_string());
            let client = client.clone();

            joinset.spawn(async move {
                let _permit = permit;
                (send_request(&client, &url, &key_header, key.as_deref()).await).ok()
            });
        }
    }

    // Drain
    let mut out: Vec<Observation> = Vec::new();
    while let Some(res) = joinset.join_next().await {
        if let Ok(Some(obs)) = res {
            out.push(obs);
        }
    }

    out
}

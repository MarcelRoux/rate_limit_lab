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

    let interval = if profile.requests_per_second == 0 {
        // avoid div-by-zero: treat as "no traffic"
        Duration::from_secs(3600)
    } else {
        Duration::from_secs_f64(1.0 / profile.requests_per_second as f64)
    };
    let mut ticker = tokio::time::interval(interval);

    let started = Instant::now();
    let deadline = started + profile.duration;

    while Instant::now() < deadline {
        ticker.tick().await;

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

    // Drain
    let mut out: Vec<Observation> = Vec::new();
    while let Some(res) = joinset.join_next().await {
        if let Ok(Some(obs)) = res {
            out.push(obs);
        }
    }

    out
}

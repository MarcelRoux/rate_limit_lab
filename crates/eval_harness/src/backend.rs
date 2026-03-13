use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use crate::{model::BackendPreparation, util::run_process};

pub(crate) fn prepare_distributed_backend(redis_url_present: bool) -> BackendPreparation {
    if !redis_url_present {
        return BackendPreparation {
            ready: false,
            details: "REDIS_URL not set".to_string(),
        };
    }

    let redis_up = run_process("make", &["redis-up"]);
    if !redis_up.success {
        return BackendPreparation {
            ready: false,
            details: format!("`make redis-up` failed: {}", redis_up.details),
        };
    }

    let probe = run_process(
        "cargo",
        &[
            "test",
            "-p",
            "state_backend",
            "--features",
            "redis-tests",
            "redis_backend_fixed_window_allows_then_denies",
            "--",
            "--exact",
        ],
    );
    if !probe.success {
        return BackendPreparation {
            ready: false,
            details: format!("redis backend probe failed: {}", probe.details),
        };
    }

    BackendPreparation {
        ready: true,
        details: "redis backend ready via `make redis-up` and fixed-window probe".to_string(),
    }
}

pub(crate) fn compute_config_hash(
    selector: &str,
    mode: &str,
    repeat: u32,
    selected_ats: &[String],
    redis_url_present: bool,
) -> Result<String, serde_json::Error> {
    let value = serde_json::json!({
        "selector": selector,
        "mode": mode,
        "repeat": repeat,
        "selected_ats": selected_ats,
        "redis_url_present": redis_url_present
    });
    let serialized = serde_json::to_string(&value)?;
    let mut hasher = DefaultHasher::new();
    serialized.hash(&mut hasher);
    Ok(format!("{:016x}", hasher.finish()))
}

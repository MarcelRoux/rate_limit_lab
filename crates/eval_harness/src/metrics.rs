use std::path::Path;

use crate::{
    model::{Metrics, Reproducibility, TraceRecord},
    util::percentile,
};

pub(crate) fn score_metrics(traces: &[TraceRecord]) -> Result<Metrics, String> {
    if traces.is_empty() {
        return Err("cannot score metrics: no traces".to_string());
    }

    let total = traces.len() as f64;
    let mut deny_count = 0.0;
    let mut decision_ok = 0.0;
    let mut http_map_ok = 0.0;
    let mut retry_after_ok = 0.0;
    let mut retry_after_denies = 0.0;
    let mut latencies: Vec<f64> = Vec::with_capacity(traces.len());

    let mut min_start = i64::MAX;
    let mut max_end = i64::MIN;

    for t in traces {
        if t.decision == "deny" {
            deny_count += 1.0;
            retry_after_denies += 1.0;
            if t.retry_after_ms.unwrap_or(0) > 0 {
                retry_after_ok += 1.0;
            }
        }
        if t.error_code.is_none() && (t.decision == "allow" || t.decision == "deny") {
            decision_ok += 1.0;
        }

        let status_ok = if t.decision == "allow" {
            (200..300).contains(&t.http_status)
        } else {
            t.http_status == 429
        };
        if status_ok && t.error_code.is_none() {
            http_map_ok += 1.0;
        }

        latencies.push(t.latency_ms as f64);
        min_start = min_start.min(t.request_started_at);
        max_end = max_end.max(t.request_completed_at);
    }

    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let duration_ms = (max_end - min_start).max(1) as f64;
    let throughput_rps_observed = total / (duration_ms / 1000.0);

    let retry_after_accuracy = if retry_after_denies > 0.0 {
        retry_after_ok / retry_after_denies
    } else {
        1.0
    };

    Ok(Metrics {
        decision_accuracy: decision_ok / total,
        retry_after_accuracy,
        http_mapping_accuracy: http_map_ok / total,
        key_isolation_error_rate: 0.0,
        backend_error_policy_conformance: None,
        short_circuit_conformance: None,
        mode_transition_conformance: None,
        throughput_rps_observed,
        deny_ratio: deny_count / total,
        latency_ms_p50: percentile(&latencies, 0.50),
        latency_ms_p95: percentile(&latencies, 0.95),
        latency_ms_p99: percentile(&latencies, 0.99),
        latency_regression_pct: 0.0,
        per_key_allow_variance: None,
        per_key_deny_variance: None,
        global_target_drift_pct: None,
        artifact_completeness_rate: 0.0,
        one_command_success_rate: None,
        baseline_update_compliance_rate: None,
    })
}

pub(crate) fn compute_artifact_completeness(run_dir: &Path) -> f64 {
    let required = [
        "manifest.json",
        "preflight.json",
        "traces.jsonl",
        "summary.json",
        "triage.json",
    ];
    let present = required
        .iter()
        .filter(|name| run_dir.join(name).exists())
        .count() as f64;
    present / required.len() as f64
}

pub(crate) fn score_reproducibility(
    repeat: u32,
    traces: &[TraceRecord],
    selected_at_count: usize,
) -> Result<Reproducibility, String> {
    if repeat < 2 {
        return Ok(Reproducibility {
            repeat_runs: repeat,
            repeat_run_decision_delta_pp: 0.0,
            repeat_run_latency_p95_delta_pct: 0.0,
            gate_passed: true,
        });
    }
    if selected_at_count == 0 {
        return Err("cannot score reproducibility: no selected ATs".to_string());
    }

    let expected_per_attempt = selected_at_count;
    let expected_total = expected_per_attempt * repeat as usize;
    if traces.len() != expected_total {
        return Err(format!(
            "cannot score reproducibility: expected {expected_total} traces for repeat={repeat} and selected_ats={selected_at_count}, found {}",
            traces.len()
        ));
    }

    let mut allow_ratios = Vec::with_capacity(repeat as usize);
    let mut p95_values = Vec::with_capacity(repeat as usize);

    for attempt in 0..repeat as usize {
        let start = attempt * expected_per_attempt;
        let end = start + expected_per_attempt;
        let chunk = &traces[start..end];
        let allow_count = chunk.iter().filter(|t| t.decision == "allow").count() as f64;
        let ratio = allow_count / expected_per_attempt as f64;
        allow_ratios.push(ratio);

        let mut latencies = chunk
            .iter()
            .map(|t| t.latency_ms as f64)
            .collect::<Vec<_>>();
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        p95_values.push(percentile(&latencies, 0.95));
    }

    let first_allow = allow_ratios[0];
    let first_p95 = p95_values[0];
    let mut max_allow_delta_pp = 0.0;
    let mut max_latency_delta_pct = 0.0;

    for idx in 1..allow_ratios.len() {
        let delta_pp = (allow_ratios[idx] - first_allow).abs() * 100.0;
        if delta_pp > max_allow_delta_pp {
            max_allow_delta_pp = delta_pp;
        }

        let denom = if first_p95 > 0.0 { first_p95 } else { 1.0 };
        let delta_pct = ((p95_values[idx] - first_p95).abs() / denom) * 100.0;
        if delta_pct > max_latency_delta_pct {
            max_latency_delta_pct = delta_pct;
        }
    }

    Ok(Reproducibility {
        repeat_runs: repeat,
        repeat_run_decision_delta_pp: max_allow_delta_pp,
        repeat_run_latency_p95_delta_pct: max_latency_delta_pct,
        gate_passed: max_allow_delta_pp <= 0.5 && max_latency_delta_pct <= 15.0,
    })
}

pub(crate) fn validate_required_artifacts(run_dir: &Path) -> Result<(), String> {
    let required = [
        "manifest.json",
        "preflight.json",
        "traces.jsonl",
        "summary.json",
        "triage.json",
    ];
    for filename in required {
        let path = run_dir.join(filename);
        if !path.exists() {
            return Err(format!("missing required artifact: {}", path.display()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::model::TraceRecord;

    use super::score_reproducibility;

    #[test]
    fn reproducibility_gate_passes_for_stable_traces() {
        let traces = vec![
            TraceRecord {
                at_id: Some("AT-012".to_string()),
                trace_id: "t1".to_string(),
                scenario_id: "smoke_ready".to_string(),
                request_started_at: 1,
                request_completed_at: 2,
                key: "k".to_string(),
                http_status: 200,
                decision: "allow".to_string(),
                retry_after_ms: None,
                latency_ms: 1,
                backend_outcome: "none".to_string(),
                failure_policy: None,
                error_code: None,
            },
            TraceRecord {
                at_id: Some("AT-012".to_string()),
                trace_id: "t2".to_string(),
                scenario_id: "smoke_ready".to_string(),
                request_started_at: 3,
                request_completed_at: 4,
                key: "k".to_string(),
                http_status: 200,
                decision: "allow".to_string(),
                retry_after_ms: None,
                latency_ms: 1,
                backend_outcome: "none".to_string(),
                failure_policy: None,
                error_code: None,
            },
        ];

        let scored = score_reproducibility(2, &traces, 1).expect("reproducibility scoring");
        assert!(scored.gate_passed);
        assert_eq!(scored.repeat_run_decision_delta_pp, 0.0);
        assert_eq!(scored.repeat_run_latency_p95_delta_pct, 0.0);
    }
}

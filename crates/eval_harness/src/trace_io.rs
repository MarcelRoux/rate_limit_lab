use std::{fs, path::Path};

use crate::{
    at_engine::is_failure_at,
    model::{AtResult, TraceRecord},
};

pub(crate) fn build_traces(
    selector: &str,
    at_results: &[AtResult],
    repeat: u32,
    start_ts: i64,
) -> Vec<String> {
    let mut out = Vec::new();
    for attempt in 0..repeat {
        for (idx, result) in at_results.iter().enumerate() {
            let backend_outcome = if result.at_id == "AT-016" || result.at_id == "AT-017" {
                if result.status == "pass" {
                    "allow"
                } else if result.status == "fail" {
                    "error"
                } else {
                    "none"
                }
            } else {
                "none"
            };
            let (decision, http_status, retry_after_ms, error_code) = match result.status.as_str() {
                "pass" => (
                    "allow",
                    200,
                    serde_json::Value::Null,
                    serde_json::Value::Null,
                ),
                "skipped" => ("allow", 200, serde_json::Value::Null, "skipped".into()),
                "fail" => (
                    "deny",
                    429,
                    serde_json::Value::Null,
                    "execution_failed".into(),
                ),
                "not_implemented" => (
                    "deny",
                    429,
                    serde_json::Value::Null,
                    "not_implemented".into(),
                ),
                _ => (
                    "deny",
                    429,
                    serde_json::Value::Null,
                    "unknown_status".into(),
                ),
            };
            let value = serde_json::json!({
                "at_id": result.at_id,
                "trace_id": format!("trace-{}-{}-{}", start_ts, attempt, idx),
                "scenario_id": selector,
                "request_started_at": start_ts + (attempt as i64),
                "request_completed_at": start_ts + (attempt as i64) + 1,
                "key": "harness-key",
                "http_status": http_status,
                "decision": decision,
                "retry_after_ms": retry_after_ms,
                "latency_ms": 1,
                "backend_outcome": backend_outcome,
                "failure_policy": if is_failure_at(&result.at_id) {
                    "fail_closed".into()
                } else {
                    serde_json::Value::Null
                },
                "error_code": if result.at_id == "AT-025" {
                    "outage_short".into()
                } else if result.at_id == "AT-026" {
                    "outage_long".into()
                } else if result.at_id == "AT-027" {
                    "latency_spike".into()
                } else if result.at_id == "AT-028" {
                    "flapping".into()
                } else {
                    error_code
                }
            });
            out.push(value.to_string());
        }
    }
    out
}

pub(crate) fn read_traces(path: &Path) -> Result<Vec<TraceRecord>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read traces file: {e}"))?;
    let mut out = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let trace: TraceRecord = serde_json::from_str(line)
            .map_err(|e| format!("parse traces line {}: {e}", line_no + 1))?;
        out.push(trace);
    }
    Ok(out)
}

pub(crate) fn validate_trace_schema(path: &Path) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read traces file: {e}"))?;
    for (line_no, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value =
            serde_json::from_str(line).map_err(|e| format!("parse traces line {line_no}: {e}"))?;
        let required_keys = [
            "trace_id",
            "scenario_id",
            "decision",
            "http_status",
            "retry_after_ms",
            "latency_ms",
            "backend_outcome",
            "failure_policy",
            "error_code",
        ];
        for key in required_keys {
            if value.get(key).is_none() {
                return Err(format!(
                    "trace schema missing key `{key}` on line {}",
                    line_no + 1
                ));
            }
        }
    }
    Ok(())
}

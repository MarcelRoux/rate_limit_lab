use std::{
    collections::{BTreeMap, hash_map::DefaultHasher},
    env, fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

use chrono::Utc;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
#[command(
    name = "eval_harness",
    version,
    about = "Acceptance evaluation harness"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run acceptance evaluation for a profile or a single AT id.
    Run {
        /// Profile id, e.g. smoke_ready or full_matrix.
        #[arg(long)]
        profile: Option<String>,
        /// Single acceptance-test id, e.g. AT-004.
        #[arg(long)]
        at: Option<String>,
        /// Number of repeated attempts for reproducibility scoring.
        #[arg(long, default_value_t = 1)]
        repeat: u32,
    },
    /// Compile existing runs into aggregate reports.
    Compile {
        /// Input runs directory.
        #[arg(long)]
        input: PathBuf,
        /// Output reports directory.
        #[arg(long)]
        output: PathBuf,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    run_id: String,
    pipeline_id: String,
    timestamp_utc: String,
    mode: String,
    repeat: u32,
    config_hash: String,
    selected_ats: Vec<String>,
    environment: EnvironmentInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct EnvironmentInfo {
    redis_url_present: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Preflight {
    passed: bool,
    checks: Vec<PreflightCheck>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PreflightCheck {
    name: String,
    required: bool,
    passed: bool,
    details: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Summary {
    status: String,
    selected_ats: Vec<String>,
    at_results: Vec<AtResult>,
    reproducibility: Reproducibility,
    distributed: DistributedEvidence,
    metrics: Metrics,
}

#[derive(Debug, Serialize, Deserialize)]
struct AtResult {
    at_id: String,
    status: String,
    evidence: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Reproducibility {
    repeat_runs: u32,
    repeat_run_decision_delta_pp: f64,
    repeat_run_latency_p95_delta_pct: f64,
    gate_passed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct DistributedEvidence {
    backend_enabled: bool,
    at_016_status: String,
    at_017_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Metrics {
    decision_accuracy: f64,
    retry_after_accuracy: f64,
    http_mapping_accuracy: f64,
    key_isolation_error_rate: f64,
    backend_error_policy_conformance: Option<f64>,
    short_circuit_conformance: Option<f64>,
    mode_transition_conformance: Option<f64>,
    throughput_rps_observed: f64,
    deny_ratio: f64,
    latency_ms_p50: f64,
    latency_ms_p95: f64,
    latency_ms_p99: f64,
    latency_regression_pct: f64,
    per_key_allow_variance: Option<f64>,
    per_key_deny_variance: Option<f64>,
    global_target_drift_pct: Option<f64>,
    artifact_completeness_rate: f64,
    one_command_success_rate: Option<f64>,
    baseline_update_compliance_rate: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TraceRecord {
    trace_id: String,
    scenario_id: String,
    request_started_at: i64,
    request_completed_at: i64,
    key: String,
    http_status: u16,
    decision: String,
    retry_after_ms: Option<u64>,
    latency_ms: u64,
    backend_outcome: String,
    failure_policy: Option<String>,
    error_code: Option<String>,
}

#[derive(Debug, Serialize)]
struct CompiledSummary {
    runs_included: Vec<String>,
    at_totals: BTreeMap<String, AtTotals>,
    triage_label_counts: BTreeMap<String, u64>,
    metrics_overview: MetricsOverview,
    regression_summary: RegressionSummary,
    evidence_links: Vec<EvidenceLink>,
}

#[derive(Debug, Serialize)]
struct AtTotals {
    pass: u64,
    fail: u64,
    skipped: u64,
    not_implemented: u64,
}

#[derive(Debug, Serialize)]
struct MetricsOverview {
    avg_decision_accuracy: f64,
    avg_deny_ratio: f64,
    avg_latency_ms_p95: f64,
}

#[derive(Debug, Serialize)]
struct RegressionSummary {
    baseline_run_id: String,
    max_latency_p95_delta_pct_vs_baseline: f64,
}

#[derive(Debug, Serialize)]
struct EvidenceLink {
    run_id: String,
    manifest: String,
    preflight: String,
    traces: String,
    summary: String,
    triage: String,
    report_md: String,
    report_json: String,
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Run {
            profile,
            at,
            repeat,
        } => run_command(profile, at, repeat),
        Command::Compile { input, output } => compile_command(&input, &output),
    };

    if let Err(err) = result {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run_command(profile: Option<String>, at: Option<String>, repeat: u32) -> Result<(), String> {
    if repeat == 0 {
        return Err("repeat must be >= 1".to_string());
    }
    match (&profile, &at) {
        (Some(_), Some(_)) => return Err("use exactly one of --profile or --at".to_string()),
        (None, None) => return Err("missing selector: use --profile or --at".to_string()),
        _ => {}
    }

    let now = Utc::now();
    let selector = profile
        .clone()
        .unwrap_or_else(|| at.clone().unwrap_or_default());
    let run_id = format!("{}_{}", now.format("%Y%m%d_%H%M%S"), sanitize(&selector));

    let runs_dir = Path::new("evaluations/runs");
    let reports_dir = Path::new("evaluations/reports");
    fs::create_dir_all(runs_dir).map_err(|e| format!("create runs dir: {e}"))?;
    fs::create_dir_all(reports_dir).map_err(|e| format!("create reports dir: {e}"))?;

    let run_dir = runs_dir.join(&run_id);
    let snapshot_dir = run_dir.join("config_snapshot");
    let run_id_unique = !run_dir.exists();
    fs::create_dir_all(&snapshot_dir).map_err(|e| format!("create run dirs: {e}"))?;

    let selected_ats = select_ats(profile.clone(), at.clone());
    let redis_url_present = env::var("REDIS_URL").is_ok();
    let environment = EnvironmentInfo { redis_url_present };
    let mode = if at.is_some() {
        "single_at".to_string()
    } else {
        "profile".to_string()
    };

    let config_hash =
        compute_config_hash(&selector, &mode, repeat, &selected_ats, redis_url_present)
            .map_err(|e| format!("compute config hash: {e}"))?;

    let manifest = Manifest {
        run_id: run_id.clone(),
        pipeline_id: selector.clone(),
        timestamp_utc: now.to_rfc3339(),
        mode,
        repeat,
        config_hash,
        selected_ats: selected_ats.clone(),
        environment,
    };

    let preflight = build_preflight(
        &selector,
        run_id_unique,
        &run_dir,
        &selected_ats,
        redis_url_present,
    );
    write_json(run_dir.join("manifest.json"), &manifest)?;
    write_json(run_dir.join("preflight.json"), &preflight)?;
    if !preflight.passed {
        return Err("preflight failed; see evaluations/runs/<run_id>/preflight.json".to_string());
    }

    let mut at_results: Vec<AtResult> = Vec::new();
    for at_id in &selected_ats {
        let result = execute_at(at_id, redis_url_present);
        at_results.push(result);
    }

    let trace_lines = build_traces(&selector, &at_results, repeat, now.timestamp_millis());
    fs::write(run_dir.join("traces.jsonl"), trace_lines.join("\n") + "\n")
        .map_err(|e| format!("write traces.jsonl: {e}"))?;

    let traces = read_traces(&run_dir.join("traces.jsonl"))?;

    let at_016_status = at_results
        .iter()
        .find(|r| r.at_id == "AT-016")
        .map(|r| r.status.clone())
        .unwrap_or_else(|| "not_selected".to_string());
    let at_017_status = at_results
        .iter()
        .find(|r| r.at_id == "AT-017")
        .map(|r| r.status.clone())
        .unwrap_or_else(|| "not_selected".to_string());
    let distributed = DistributedEvidence {
        backend_enabled: redis_url_present,
        at_016_status,
        at_017_status,
    };

    let reproducibility = score_reproducibility(repeat, &traces, at_results.len())?;
    let metrics = score_metrics(&traces)?;
    let has_fail = at_results.iter().any(|r| r.status == "fail");
    let has_not_implemented = at_results.iter().any(|r| r.status == "not_implemented");
    let status = if has_fail || has_not_implemented {
        "fail"
    } else {
        "pass"
    };

    let mut summary = Summary {
        status: status.to_string(),
        selected_ats,
        at_results,
        reproducibility,
        distributed,
        metrics,
    };
    write_json(run_dir.join("summary.json"), &summary)?;

    let mut triage_labels: Vec<&str> = Vec::new();
    if has_not_implemented {
        triage_labels.push("MISSING_REQUIRED_EVIDENCE");
    }
    if has_fail {
        triage_labels.push("EXECUTION_FAILURE");
    }
    let triage = serde_json::json!({ "labels": triage_labels });
    write_json(run_dir.join("triage.json"), &triage)?;

    summary.metrics.artifact_completeness_rate = compute_artifact_completeness(run_dir.as_path());
    write_json(run_dir.join("summary.json"), &summary)?;

    validate_required_artifacts(&run_dir)?;
    validate_trace_schema(&run_dir.join("traces.jsonl"))?;

    let report_md_path = reports_dir.join(format!("run_{}.md", run_id));
    let report_json_path = reports_dir.join(format!("run_{}.json", run_id));

    let at_rows = summary
        .at_results
        .iter()
        .map(|r| format!("| {} | {} | {} |", r.at_id, r.status, r.evidence))
        .collect::<Vec<_>>()
        .join("\n");

    let report_md = format!(
        "# Acceptance Run {}\n\n## Status\n- Status: {}\n\n## Evidence Links\n- Manifest: {}\n- Preflight: {}\n- Traces: {}\n- Summary: {}\n- Triage: {}\n\n## Key Metrics\n- decision_accuracy: {}\n- http_mapping_accuracy: {}\n- deny_ratio: {}\n- latency_ms_p95: {}\n- throughput_rps_observed: {}\n- artifact_completeness_rate: {}\n\n## Reproducibility\n- gate_passed: {}\n- repeat_run_decision_delta_pp: {}\n- repeat_run_latency_p95_delta_pct: {}\n\n## AT Results\n| AT ID | Status | Evidence |\n|---|---|---|\n{}\n",
        run_id,
        summary.status,
        run_dir.join("manifest.json").display(),
        run_dir.join("preflight.json").display(),
        run_dir.join("traces.jsonl").display(),
        run_dir.join("summary.json").display(),
        run_dir.join("triage.json").display(),
        summary.metrics.decision_accuracy,
        summary.metrics.http_mapping_accuracy,
        summary.metrics.deny_ratio,
        summary.metrics.latency_ms_p95,
        summary.metrics.throughput_rps_observed,
        summary.metrics.artifact_completeness_rate,
        summary.reproducibility.gate_passed,
        summary.reproducibility.repeat_run_decision_delta_pp,
        summary.reproducibility.repeat_run_latency_p95_delta_pct,
        at_rows,
    );
    fs::write(&report_md_path, report_md).map_err(|e| format!("write report md: {e}"))?;

    let report_json = serde_json::json!({
        "run_id": run_id,
        "status": summary.status,
        "evidence": {
            "manifest": run_dir.join("manifest.json"),
            "preflight": run_dir.join("preflight.json"),
            "traces": run_dir.join("traces.jsonl"),
            "summary": run_dir.join("summary.json"),
            "triage": run_dir.join("triage.json")
        },
        "metrics": summary.metrics,
        "reproducibility": summary.reproducibility,
        "distributed": summary.distributed,
        "at_results": summary.at_results
    });
    write_json(report_json_path, &report_json)?;

    Ok(())
}

fn select_ats(profile: Option<String>, at: Option<String>) -> Vec<String> {
    match (profile, at) {
        (Some(p), None) if p == "smoke_ready" => vec![
            "AT-004", "AT-005", "AT-006", "AT-007", "AT-008", "AT-009", "AT-010", "AT-011",
            "AT-012", "AT-013", "AT-014", "AT-015", "AT-018", "AT-019", "AT-020", "AT-021",
            "AT-022", "AT-023", "AT-024", "AT-050",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        (Some(p), None) if p == "full_matrix" => vec![
            "AT-004", "AT-005", "AT-006", "AT-007", "AT-008", "AT-009", "AT-010", "AT-011",
            "AT-012", "AT-013", "AT-014", "AT-015", "AT-016", "AT-017", "AT-018", "AT-019",
            "AT-020", "AT-021", "AT-022", "AT-023", "AT-024", "AT-050",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        (Some(_), None) => vec![String::from("AT-004")],
        (None, Some(single)) => vec![single],
        _ => Vec::new(),
    }
}

fn build_preflight(
    selector: &str,
    run_id_unique: bool,
    run_dir: &Path,
    selected_ats: &[String],
    redis_url_present: bool,
) -> Preflight {
    let mut checks = Vec::new();

    checks.push(PreflightCheck {
        name: "selector_valid".to_string(),
        required: true,
        passed: !selector.is_empty(),
        details: "selector must be non-empty".to_string(),
    });
    checks.push(PreflightCheck {
        name: "run_id_unique".to_string(),
        required: true,
        passed: run_id_unique,
        details: "run directory must not already exist".to_string(),
    });
    checks.push(PreflightCheck {
        name: "artifact_paths_writable".to_string(),
        required: true,
        passed: run_dir.exists(),
        details: "run directory created successfully".to_string(),
    });

    let distributed_requested = selected_ats
        .iter()
        .any(|id| id == "AT-016" || id == "AT-017");
    checks.push(PreflightCheck {
        name: "redis_url_present_for_distributed".to_string(),
        required: false,
        passed: !distributed_requested || redis_url_present,
        details: if distributed_requested {
            "AT-016/AT-017 selected; REDIS_URL required for full distributed execution".to_string()
        } else {
            "distributed ATs not selected".to_string()
        },
    });

    let passed = checks.iter().filter(|c| c.required).all(|c| c.passed);
    Preflight { passed, checks }
}

fn execute_at(at_id: &str, redis_url_present: bool) -> AtResult {
    if (at_id == "AT-016" || at_id == "AT-017") && !redis_url_present {
        return AtResult {
            at_id: at_id.to_string(),
            status: "skipped".to_string(),
            evidence: "REDIS_URL not set".to_string(),
        };
    }
    AtResult {
        at_id: at_id.to_string(),
        status: "not_implemented".to_string(),
        evidence: "missing_required_evidence: AT execution not implemented".to_string(),
    }
}

fn build_traces(
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
                "failure_policy": serde_json::Value::Null,
                "error_code": error_code
            });
            out.push(value.to_string());
        }
    }
    out
}

fn read_traces(path: &Path) -> Result<Vec<TraceRecord>, String> {
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

fn score_metrics(traces: &[TraceRecord]) -> Result<Metrics, String> {
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

fn compute_artifact_completeness(run_dir: &Path) -> f64 {
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

fn score_reproducibility(
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

fn validate_required_artifacts(run_dir: &Path) -> Result<(), String> {
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

fn validate_trace_schema(path: &Path) -> Result<(), String> {
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

fn compute_config_hash(
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

fn compile_command(input: &Path, output: &Path) -> Result<(), String> {
    fs::create_dir_all(output).map_err(|e| format!("create output dir: {e}"))?;

    let mut run_dirs: Vec<PathBuf> = Vec::new();
    if input.exists() {
        for entry in fs::read_dir(input).map_err(|e| format!("read input dir: {e}"))? {
            let entry = entry.map_err(|e| format!("read dir entry: {e}"))?;
            if entry.path().is_dir() {
                run_dirs.push(entry.path());
            }
        }
    }
    run_dirs.sort();

    let mut runs_included = Vec::new();
    let mut at_totals: BTreeMap<String, AtTotals> = BTreeMap::new();
    let mut triage_counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut evidence_links: Vec<EvidenceLink> = Vec::new();
    let mut decision_acc_sum = 0.0;
    let mut deny_ratio_sum = 0.0;
    let mut p95_sum = 0.0;
    let mut metric_runs = 0.0;
    let mut first_p95: Option<(String, f64)> = None;
    let mut max_delta_vs_baseline: f64 = 0.0;

    for run_dir in run_dirs {
        let run_id = run_dir
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| "invalid run dir name".to_string())?
            .to_string();

        let manifest_path = run_dir.join("manifest.json");
        let summary_path = run_dir.join("summary.json");
        let triage_path = run_dir.join("triage.json");

        if !(manifest_path.exists() && summary_path.exists() && triage_path.exists()) {
            continue;
        }

        let summary_value: serde_json::Value = read_json(&summary_path)?;
        let triage_value: serde_json::Value = read_json(&triage_path)?;
        let parsed = parse_summary_for_compile(&summary_value);

        runs_included.push(run_id.clone());

        for at in parsed.at_results {
            let entry = at_totals.entry(at.at_id).or_insert(AtTotals {
                pass: 0,
                fail: 0,
                skipped: 0,
                not_implemented: 0,
            });
            match at.status.as_str() {
                "pass" => entry.pass += 1,
                "skipped" => entry.skipped += 1,
                "not_implemented" => entry.not_implemented += 1,
                _ => entry.fail += 1,
            }
        }

        if let Some(labels) = triage_value.get("labels").and_then(|v| v.as_array()) {
            for label in labels {
                if let Some(name) = label.as_str() {
                    *triage_counts.entry(name.to_string()).or_insert(0) += 1;
                }
            }
        }

        decision_acc_sum += parsed.metrics.decision_accuracy;
        deny_ratio_sum += parsed.metrics.deny_ratio;
        p95_sum += parsed.metrics.latency_ms_p95;
        metric_runs += 1.0;

        if let Some((_, base_p95)) = &first_p95 {
            if *base_p95 > 0.0 {
                let delta = ((parsed.metrics.latency_ms_p95 - *base_p95) / *base_p95) * 100.0;
                max_delta_vs_baseline = max_delta_vs_baseline.max(delta);
            }
        } else {
            first_p95 = Some((run_id.clone(), parsed.metrics.latency_ms_p95));
        }

        evidence_links.push(EvidenceLink {
            run_id: run_id.clone(),
            manifest: manifest_path.display().to_string(),
            preflight: run_dir.join("preflight.json").display().to_string(),
            traces: run_dir.join("traces.jsonl").display().to_string(),
            summary: summary_path.display().to_string(),
            triage: triage_path.display().to_string(),
            report_md: format!("evaluations/reports/run_{}.md", run_id),
            report_json: format!("evaluations/reports/run_{}.json", run_id),
        });
    }

    let overview = if metric_runs > 0.0 {
        MetricsOverview {
            avg_decision_accuracy: decision_acc_sum / metric_runs,
            avg_deny_ratio: deny_ratio_sum / metric_runs,
            avg_latency_ms_p95: p95_sum / metric_runs,
        }
    } else {
        MetricsOverview {
            avg_decision_accuracy: 0.0,
            avg_deny_ratio: 0.0,
            avg_latency_ms_p95: 0.0,
        }
    };

    let regression_summary = RegressionSummary {
        baseline_run_id: first_p95
            .map(|(id, _)| id)
            .unwrap_or_else(|| "none".to_string()),
        max_latency_p95_delta_pct_vs_baseline: max_delta_vs_baseline,
    };

    let compiled = CompiledSummary {
        runs_included: runs_included.clone(),
        at_totals,
        triage_label_counts: triage_counts,
        metrics_overview: overview,
        regression_summary,
        evidence_links,
    };

    let stamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let md_path = output.join(format!("compiled_{}.md", stamp));
    let json_path = output.join(format!("compiled_{}.json", stamp));

    let at_rows = compiled
        .at_totals
        .iter()
        .map(|(at_id, totals)| {
            format!(
                "| {} | {} | {} | {} | {} |",
                at_id, totals.pass, totals.fail, totals.skipped, totals.not_implemented
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let triage_rows = if compiled.triage_label_counts.is_empty() {
        "- none".to_string()
    } else {
        compiled
            .triage_label_counts
            .iter()
            .map(|(label, count)| format!("- {}: {}", label, count))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let evidence_rows = compiled
        .evidence_links
        .iter()
        .map(|e| format!("- {}: {}", e.run_id, e.report_md))
        .collect::<Vec<_>>()
        .join("\n");

    let md = format!(
        "# Compiled Acceptance Report\n\n## Runs Included\n- Count: {}\n{}\n\n## AT Pass/Fail Totals\n| AT ID | Pass | Fail | Skipped | Not Implemented |\n|---|---:|---:|---:|---:|\n{}\n\n## Metrics Overview\n- avg_decision_accuracy: {}\n- avg_deny_ratio: {}\n- avg_latency_ms_p95: {}\n\n## Regression Summary\n- baseline_run_id: {}\n- max_latency_p95_delta_pct_vs_baseline: {}\n\n## Triage Label Counts\n{}\n\n## Evidence Links\n{}\n",
        compiled.runs_included.len(),
        compiled
            .runs_included
            .iter()
            .map(|r| format!("- {}", r))
            .collect::<Vec<_>>()
            .join("\n"),
        at_rows,
        compiled.metrics_overview.avg_decision_accuracy,
        compiled.metrics_overview.avg_deny_ratio,
        compiled.metrics_overview.avg_latency_ms_p95,
        compiled.regression_summary.baseline_run_id,
        compiled
            .regression_summary
            .max_latency_p95_delta_pct_vs_baseline,
        triage_rows,
        evidence_rows,
    );
    fs::write(md_path, md).map_err(|e| format!("write compiled md: {e}"))?;

    write_json(json_path, &compiled)?;
    Ok(())
}

#[derive(Debug)]
struct CompileSummaryView {
    at_results: Vec<AtResult>,
    metrics: Metrics,
}

fn parse_summary_for_compile(value: &serde_json::Value) -> CompileSummaryView {
    let at_results = value
        .get("at_results")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|item| AtResult {
                    at_id: item
                        .get("at_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    status: item
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    evidence: item
                        .get("evidence")
                        .and_then(|v| v.as_str())
                        .unwrap_or("none")
                        .to_string(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let m = value.get("metrics");
    let metrics = Metrics {
        decision_accuracy: m
            .and_then(|mm| mm.get("decision_accuracy"))
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0),
        retry_after_accuracy: m
            .and_then(|mm| mm.get("retry_after_accuracy"))
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0),
        http_mapping_accuracy: m
            .and_then(|mm| mm.get("http_mapping_accuracy"))
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0),
        key_isolation_error_rate: m
            .and_then(|mm| mm.get("key_isolation_error_rate"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        backend_error_policy_conformance: m
            .and_then(|mm| mm.get("backend_error_policy_conformance"))
            .and_then(|v| v.as_f64()),
        short_circuit_conformance: m
            .and_then(|mm| mm.get("short_circuit_conformance"))
            .and_then(|v| v.as_f64()),
        mode_transition_conformance: m
            .and_then(|mm| mm.get("mode_transition_conformance"))
            .and_then(|v| v.as_f64()),
        throughput_rps_observed: m
            .and_then(|mm| mm.get("throughput_rps_observed"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        deny_ratio: m
            .and_then(|mm| mm.get("deny_ratio"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        latency_ms_p50: m
            .and_then(|mm| mm.get("latency_ms_p50"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        latency_ms_p95: m
            .and_then(|mm| mm.get("latency_ms_p95"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        latency_ms_p99: m
            .and_then(|mm| mm.get("latency_ms_p99"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        latency_regression_pct: m
            .and_then(|mm| mm.get("latency_regression_pct"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        per_key_allow_variance: m
            .and_then(|mm| mm.get("per_key_allow_variance"))
            .and_then(|v| v.as_f64()),
        per_key_deny_variance: m
            .and_then(|mm| mm.get("per_key_deny_variance"))
            .and_then(|v| v.as_f64()),
        global_target_drift_pct: m
            .and_then(|mm| mm.get("global_target_drift_pct"))
            .and_then(|v| v.as_f64()),
        artifact_completeness_rate: m
            .and_then(|mm| mm.get("artifact_completeness_rate"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        one_command_success_rate: m
            .and_then(|mm| mm.get("one_command_success_rate"))
            .and_then(|v| v.as_f64()),
        baseline_update_compliance_rate: m
            .and_then(|mm| mm.get("baseline_update_compliance_rate"))
            .and_then(|v| v.as_f64()),
    };

    CompileSummaryView {
        at_results,
        metrics,
    }
}

fn write_json<T: Serialize>(path: PathBuf, value: &T) -> Result<(), String> {
    let data = serde_json::to_string_pretty(value).map_err(|e| format!("serialize json: {e}"))?;
    fs::write(path, data).map_err(|e| format!("write json: {e}"))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text =
        fs::read_to_string(path).map_err(|e| format!("read json {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse json {}: {e}", path.display()))
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((p * (sorted.len() as f64 - 1.0)).round() as usize).min(sorted.len() - 1);
    sorted[idx]
}

fn sanitize(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

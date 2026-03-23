use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Manifest {
    pub(crate) run_id: String,
    pub(crate) pipeline_id: String,
    pub(crate) timestamp_utc: String,
    pub(crate) mode: String,
    pub(crate) repeat: u32,
    pub(crate) config_hash: String,
    pub(crate) selected_ats: Vec<String>,
    pub(crate) environment: EnvironmentInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct EnvironmentInfo {
    pub(crate) redis_url_present: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Preflight {
    pub(crate) passed: bool,
    pub(crate) checks: Vec<PreflightCheck>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PreflightCheck {
    pub(crate) name: String,
    pub(crate) required: bool,
    pub(crate) passed: bool,
    pub(crate) details: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Summary {
    pub(crate) status: String,
    pub(crate) selected_ats: Vec<String>,
    pub(crate) at_results: Vec<AtResult>,
    pub(crate) reproducibility: Reproducibility,
    pub(crate) distributed: DistributedEvidence,
    pub(crate) topology: TopologyEvidence,
    pub(crate) metrics: Metrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AtResult {
    pub(crate) at_id: String,
    pub(crate) status: String,
    pub(crate) evidence: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Reproducibility {
    pub(crate) repeat_runs: u32,
    pub(crate) repeat_run_decision_delta_pp: f64,
    pub(crate) repeat_run_latency_p95_delta_pct: f64,
    pub(crate) gate_passed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DistributedEvidence {
    pub(crate) backend_enabled: bool,
    pub(crate) at_016_status: String,
    pub(crate) at_017_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TopologyEvidence {
    pub(crate) rr_profile_selected: bool,
    pub(crate) sa_profile_selected: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Metrics {
    pub(crate) decision_accuracy: f64,
    pub(crate) retry_after_accuracy: f64,
    pub(crate) http_mapping_accuracy: f64,
    pub(crate) key_isolation_error_rate: f64,
    pub(crate) backend_error_policy_conformance: Option<f64>,
    pub(crate) short_circuit_conformance: Option<f64>,
    pub(crate) mode_transition_conformance: Option<f64>,
    pub(crate) throughput_rps_observed: f64,
    pub(crate) deny_ratio: f64,
    pub(crate) latency_ms_p50: f64,
    pub(crate) latency_ms_p95: f64,
    pub(crate) latency_ms_p99: f64,
    pub(crate) latency_regression_pct: f64,
    pub(crate) per_key_allow_variance: Option<f64>,
    pub(crate) per_key_deny_variance: Option<f64>,
    pub(crate) global_target_drift_pct: Option<f64>,
    pub(crate) artifact_completeness_rate: f64,
    pub(crate) one_command_success_rate: Option<f64>,
    pub(crate) baseline_update_compliance_rate: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TraceRecord {
    pub(crate) at_id: Option<String>,
    pub(crate) trace_id: String,
    pub(crate) scenario_id: String,
    pub(crate) request_started_at: i64,
    pub(crate) request_completed_at: i64,
    pub(crate) key: String,
    pub(crate) http_status: u16,
    pub(crate) decision: String,
    pub(crate) retry_after_ms: Option<u64>,
    pub(crate) latency_ms: u64,
    pub(crate) backend_outcome: String,
    pub(crate) failure_policy: Option<String>,
    pub(crate) error_code: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TimelineEvent {
    pub(crate) at_id: String,
    pub(crate) mode: String,
    pub(crate) event: String,
    pub(crate) ts_ms: i64,
    pub(crate) policy: String,
    pub(crate) conformant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RrSaComparison {
    pub(crate) rr_per_key_allow_variance: f64,
    pub(crate) sa_per_key_allow_variance: f64,
    pub(crate) rr_global_target_drift_pct: f64,
    pub(crate) sa_global_target_drift_pct: f64,
    pub(crate) fairness_preferred_profile: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct CompiledSummary {
    pub(crate) runs_included: Vec<String>,
    pub(crate) at_totals: BTreeMap<String, AtTotals>,
    pub(crate) triage_label_counts: BTreeMap<String, u64>,
    pub(crate) metrics_overview: MetricsOverview,
    pub(crate) regression_summary: RegressionSummary,
    pub(crate) evidence_links: Vec<EvidenceLink>,
}

#[derive(Debug, Serialize)]
pub(crate) struct AtTotals {
    pub(crate) pass: u64,
    pub(crate) fail: u64,
    pub(crate) skipped: u64,
    pub(crate) not_implemented: u64,
}

#[derive(Debug, Serialize)]
pub(crate) struct MetricsOverview {
    pub(crate) avg_decision_accuracy: f64,
    pub(crate) avg_deny_ratio: f64,
    pub(crate) avg_latency_ms_p95: f64,
}

#[derive(Debug, Serialize)]
pub(crate) struct RegressionSummary {
    pub(crate) baseline_run_id: String,
    pub(crate) max_latency_p95_delta_pct_vs_baseline: f64,
}

#[derive(Debug, Serialize)]
pub(crate) struct EvidenceLink {
    pub(crate) run_id: String,
    pub(crate) manifest: String,
    pub(crate) preflight: String,
    pub(crate) traces: String,
    pub(crate) summary: String,
    pub(crate) triage: String,
    pub(crate) report_md: String,
    pub(crate) report_json: String,
}

#[derive(Debug, Clone)]
pub(crate) struct BackendPreparation {
    pub(crate) ready: bool,
    pub(crate) details: String,
}

#[derive(Debug)]
pub(crate) struct ProcessResult {
    pub(crate) success: bool,
    pub(crate) details: String,
}

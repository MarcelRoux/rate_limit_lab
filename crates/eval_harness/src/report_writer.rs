use std::{fs, path::Path};

use crate::{
    model::{RrSaComparison, Summary},
    util::write_json,
};

pub(crate) fn write_run_reports(
    reports_dir: &Path,
    run_id: &str,
    run_dir: &Path,
    summary: &Summary,
    rr_sa_comparison: Option<RrSaComparison>,
) -> Result<(), String> {
    let report_md_path = reports_dir.join(format!("run_{}.md", run_id));
    let report_json_path = reports_dir.join(format!("run_{}.json", run_id));
    let observability_evidence_path = run_dir.join("observability_evidence.json");

    let at_rows = summary
        .at_results
        .iter()
        .map(|r| format!("| {} | {} | {} |", r.at_id, r.status, r.evidence))
        .collect::<Vec<_>>()
        .join("\n");
    let has_observability_ats = summary.at_results.iter().any(|r| {
        matches!(
            r.at_id.as_str(),
            "AT-052" | "AT-053" | "AT-054" | "AT-055" | "AT-056"
        )
    });
    let observability_section = if has_observability_ats {
        if observability_evidence_path.exists() {
            format!(
                "## Observability Evidence\n- Artifact: {}\n",
                observability_evidence_path.display()
            )
        } else {
            "## Observability Evidence\n- Artifact: not available for this run\n".to_string()
        }
    } else {
        String::new()
    };

    let report_md = format!(
        "# Acceptance Run {}\n\n## Status\n- Status: {}\n\n## Evidence Links\n- Manifest: {}\n- Preflight: {}\n- Traces: {}\n- Summary: {}\n- Triage: {}\n{}\n{}\n\n{}\n## Key Metrics\n- decision_accuracy: {}\n- http_mapping_accuracy: {}\n- deny_ratio: {}\n- latency_ms_p95: {}\n- throughput_rps_observed: {}\n- artifact_completeness_rate: {}\n- backend_error_policy_conformance: {}\n- per_key_allow_variance: {}\n- global_target_drift_pct: {}\n\n## Reproducibility\n- gate_passed: {}\n- repeat_run_decision_delta_pp: {}\n- repeat_run_latency_p95_delta_pct: {}\n\n## Fairness/Drift Comparison\n{}\n\n## AT Results\n| AT ID | Status | Evidence |\n|---|---|---|\n{}\n",
        run_id,
        summary.status,
        run_dir.join("manifest.json").display(),
        run_dir.join("preflight.json").display(),
        run_dir.join("traces.jsonl").display(),
        run_dir.join("summary.json").display(),
        run_dir.join("triage.json").display(),
        if run_dir.join("failure_timeline.json").exists() {
            format!(
                "- Failure Timeline: {}",
                run_dir.join("failure_timeline.json").display()
            )
        } else {
            String::new()
        },
        if run_dir.join("rr_sa_comparison.json").exists() {
            format!(
                "- RR/SA Comparison: {}",
                run_dir.join("rr_sa_comparison.json").display()
            )
        } else {
            String::new()
        },
        observability_section,
        summary.metrics.decision_accuracy,
        summary.metrics.http_mapping_accuracy,
        summary.metrics.deny_ratio,
        summary.metrics.latency_ms_p95,
        summary.metrics.throughput_rps_observed,
        summary.metrics.artifact_completeness_rate,
        summary
            .metrics
            .backend_error_policy_conformance
            .map_or_else(|| "null".to_string(), |v| v.to_string()),
        summary
            .metrics
            .per_key_allow_variance
            .map_or_else(|| "null".to_string(), |v| v.to_string()),
        summary
            .metrics
            .global_target_drift_pct
            .map_or_else(|| "null".to_string(), |v| v.to_string()),
        summary.reproducibility.gate_passed,
        summary.reproducibility.repeat_run_decision_delta_pp,
        summary.reproducibility.repeat_run_latency_p95_delta_pct,
        if let Some(cmp) = rr_sa_comparison.as_ref() {
            format!(
                "- rr_per_key_allow_variance: {}\n- sa_per_key_allow_variance: {}\n- rr_global_target_drift_pct: {}\n- sa_global_target_drift_pct: {}\n- fairness_preferred_profile: {}",
                cmp.rr_per_key_allow_variance,
                cmp.sa_per_key_allow_variance,
                cmp.rr_global_target_drift_pct,
                cmp.sa_global_target_drift_pct,
                cmp.fairness_preferred_profile
            )
        } else {
            "- not available for this run".to_string()
        },
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
            "triage": run_dir.join("triage.json"),
            "observability": if observability_evidence_path.exists() {
                serde_json::Value::String(observability_evidence_path.display().to_string())
            } else {
                serde_json::Value::Null
            }
        },
        "metrics": summary.metrics,
        "reproducibility": summary.reproducibility,
        "distributed": summary.distributed,
        "topology": summary.topology,
        "at_results": summary.at_results
    });
    write_json(report_json_path, &report_json)?;

    Ok(())
}

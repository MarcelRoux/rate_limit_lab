use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use chrono::Utc;

use crate::{
    model::{
        AtResult, AtTotals, CompiledSummary, EvidenceLink, Metrics, MetricsOverview,
        RegressionSummary,
    },
    util::{read_json, write_json},
};

pub(crate) fn compile_command(input: &Path, output: &Path) -> Result<(), String> {
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

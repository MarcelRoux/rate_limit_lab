use std::{env, fs, path::Path};

use chrono::Utc;

use crate::{
    at_engine::{
        at_registry_validate_selected, build_failure_timeline, build_rr_sa_comparison, execute_at,
        finalize_distributed_trace_checks, finalize_post_run_contract_checks, is_failure_at,
        score_backend_policy_conformance,
    },
    backend::{compute_config_hash, prepare_distributed_backend},
    metrics::{
        compute_artifact_completeness, score_metrics, score_reproducibility,
        validate_required_artifacts,
    },
    model::{AtResult, DistributedEvidence, EnvironmentInfo, Manifest, Summary, TopologyEvidence},
    preflight::build_preflight,
    profiles::select_ats,
    report_writer::write_run_reports,
    trace_io::{build_traces, read_traces, validate_trace_schema},
    util::{sanitize, write_json},
};

pub(crate) fn run_command(
    profile: Option<String>,
    at: Option<String>,
    repeat: u32,
) -> Result<(), String> {
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
    at_registry_validate_selected(&selected_ats)?;
    let redis_url_present = env::var("REDIS_URL").is_ok();
    let distributed_requested = selected_ats
        .iter()
        .any(|id| id == "AT-016" || id == "AT-017");
    let distributed_backend_ready = if distributed_requested {
        Some(prepare_distributed_backend(redis_url_present))
    } else {
        None
    };
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
        distributed_backend_ready.as_ref(),
    );
    write_json(run_dir.join("manifest.json"), &manifest)?;
    write_json(run_dir.join("preflight.json"), &preflight)?;
    if !preflight.passed {
        return Err("preflight failed; see evaluations/runs/<run_id>/preflight.json".to_string());
    }

    let mut at_results: Vec<AtResult> = Vec::new();
    for at_id in &selected_ats {
        let result = execute_at(at_id, redis_url_present, distributed_backend_ready.as_ref());
        at_results.push(result);
    }

    let trace_lines = build_traces(&selector, &at_results, repeat, now.timestamp_millis());
    fs::write(run_dir.join("traces.jsonl"), trace_lines.join("\n") + "\n")
        .map_err(|e| format!("write traces.jsonl: {e}"))?;

    let traces = read_traces(&run_dir.join("traces.jsonl"))?;
    finalize_distributed_trace_checks(&mut at_results, &traces);

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
    let topology = TopologyEvidence {
        rr_profile_selected: selected_ats
            .iter()
            .any(|id| id == "AT-030" || id == "AT-032"),
        sa_profile_selected: selected_ats
            .iter()
            .any(|id| id == "AT-031" || id == "AT-033"),
    };

    let reproducibility = score_reproducibility(repeat, &traces, at_results.len())?;
    let mut metrics = score_metrics(&traces)?;
    let failure_timeline = build_failure_timeline(&at_results, now.timestamp_millis());
    if !failure_timeline.is_empty() {
        write_json(run_dir.join("failure_timeline.json"), &failure_timeline)?;
        if let Some(value) = score_backend_policy_conformance(&failure_timeline) {
            metrics.backend_error_policy_conformance = Some(value);
        }
        for at in &mut at_results {
            if is_failure_at(&at.at_id) && at.status == "pass" {
                at.evidence = format!(
                    "{}; timeline: {}",
                    at.evidence,
                    run_dir.join("failure_timeline.json").display()
                );
            }
        }
    }

    let rr_sa_comparison = build_rr_sa_comparison(&at_results, &traces);
    if let Some(comparison) = rr_sa_comparison.as_ref() {
        write_json(run_dir.join("rr_sa_comparison.json"), comparison)?;
        metrics.per_key_allow_variance = Some(comparison.rr_per_key_allow_variance);
        metrics.global_target_drift_pct = Some(comparison.rr_global_target_drift_pct);
    }

    finalize_post_run_contract_checks(
        run_dir.as_path(),
        &mut at_results,
        &reproducibility,
        rr_sa_comparison.as_ref(),
    );

    let has_fail = at_results.iter().any(|r| r.status == "fail");
    let has_not_implemented = at_results.iter().any(|r| r.status == "not_implemented");
    let has_at_050_probe = at_results
        .iter()
        .any(|r| r.at_id == "AT-050" && r.status == "pass");
    let has_repro_failure = !reproducibility.gate_passed;
    let status = if has_fail || has_not_implemented || has_repro_failure {
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
        topology,
        metrics,
    };
    write_json(run_dir.join("summary.json"), &summary)?;

    let mut triage_labels: Vec<&str> = Vec::new();
    if has_not_implemented || has_at_050_probe {
        triage_labels.push("MISSING_REQUIRED_EVIDENCE");
    }
    if has_fail {
        triage_labels.push("EXECUTION_FAILURE");
    }
    if has_repro_failure {
        triage_labels.push("NON_REPRODUCIBLE_RUN");
    }
    let triage = serde_json::json!({ "labels": triage_labels });
    write_json(run_dir.join("triage.json"), &triage)?;

    summary.metrics.artifact_completeness_rate = compute_artifact_completeness(run_dir.as_path());
    write_json(run_dir.join("summary.json"), &summary)?;

    validate_required_artifacts(&run_dir)?;
    validate_trace_schema(&run_dir.join("traces.jsonl"))?;

    write_run_reports(reports_dir, &run_id, &run_dir, &summary, rr_sa_comparison)?;

    Ok(())
}

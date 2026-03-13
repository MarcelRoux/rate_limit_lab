use std::path::Path;

use crate::model::{BackendPreparation, Preflight, PreflightCheck};

pub(crate) fn build_preflight(
    selector: &str,
    run_id_unique: bool,
    run_dir: &Path,
    selected_ats: &[String],
    redis_url_present: bool,
    distributed_backend_ready: Option<&BackendPreparation>,
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
        required: distributed_requested,
        passed: !distributed_requested || redis_url_present,
        details: if distributed_requested {
            "AT-016/AT-017 selected; REDIS_URL required for full distributed execution".to_string()
        } else {
            "distributed ATs not selected".to_string()
        },
    });
    if distributed_requested {
        let (passed, details) = if let Some(prep) = distributed_backend_ready {
            (prep.ready, prep.details.clone())
        } else {
            (
                false,
                "distributed backend preflight missing; this is a harness bug".to_string(),
            )
        };
        checks.push(PreflightCheck {
            name: "distributed_backend_ready".to_string(),
            required: true,
            passed,
            details,
        });
    }

    let passed = checks.iter().filter(|c| c.required).all(|c| c.passed);
    Preflight { passed, checks }
}

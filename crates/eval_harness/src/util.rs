use std::{
    fs,
    path::{Path, PathBuf},
    process::Command as ProcessCommand,
};

use serde::{Deserialize, Serialize};

use crate::model::ProcessResult;

pub(crate) fn write_json<T: Serialize>(path: PathBuf, value: &T) -> Result<(), String> {
    let data = serde_json::to_string_pretty(value).map_err(|e| format!("serialize json: {e}"))?;
    fs::write(path, data).map_err(|e| format!("write json: {e}"))
}

pub(crate) fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text =
        fs::read_to_string(path).map_err(|e| format!("read json {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse json {}: {e}", path.display()))
}

pub(crate) fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((p * (sorted.len() as f64 - 1.0)).round() as usize).min(sorted.len() - 1);
    sorted[idx]
}

pub(crate) fn sanitize(input: &str) -> String {
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

pub(crate) fn run_process(program: &str, args: &[&str]) -> ProcessResult {
    match ProcessCommand::new(program).args(args).output() {
        Ok(output) => {
            if output.status.success() {
                ProcessResult {
                    success: true,
                    details: "ok".to_string(),
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                let msg = if !stderr.trim().is_empty() {
                    stderr.trim().to_string()
                } else if !stdout.trim().is_empty() {
                    stdout.trim().to_string()
                } else {
                    format!("exit status {}", output.status)
                };
                ProcessResult {
                    success: false,
                    details: msg,
                }
            }
        }
        Err(e) => ProcessResult {
            success: false,
            details: e.to_string(),
        },
    }
}

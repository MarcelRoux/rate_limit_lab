#!/usr/bin/env sh
set -eu

CASES_INPUT="${1:-${CASES:-}}"
OBS_RUNS_DIR="${OBS_RUNS_DIR:-evaluations/obs_runs}"
RUN_ID="${OBS_RUN_ID:-$(date +%Y%m%d_%H%M%S)_obs_batch}"
run_dir="$OBS_RUNS_DIR/$RUN_ID"

if [ -z "$CASES_INPUT" ]; then
  echo "Missing CASES list. Usage:"
  echo "  make obs-cases CASES=\"OBS-001 OBS-002 OBS-003\""
  echo "  make obs-cases CASES=\"configs/traffic_rest/observability/obs_002_single_key_low_rate_200x2_8s.toml\""
  exit 2
fi

mkdir -p "$run_dir"

passed=0
failed=0
results_list=""
started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

for case_input in $CASES_INPUT; do
  echo "Running case: $case_input"
  if OBS_RUN_ID="$RUN_ID" ./scripts/obs/case.sh "$case_input"; then
    passed=$((passed + 1))
  else
    failed=$((failed + 1))
  fi

  case_slug="$case_input"
  case "$case_slug" in
    /app/*)
      case_slug="${case_slug#/app/}"
      ;;
  esac
  case_slug="$(basename "$case_slug" .toml)"

  if [ -f "$run_dir/$case_input/result.json" ]; then
    result_path="$run_dir/$case_input/result.json"
  elif [ -f "$run_dir/$case_slug/result.json" ]; then
    result_path="$run_dir/$case_slug/result.json"
  else
    result_path=""
  fi

  if [ -n "$result_path" ]; then
    if [ -z "$results_list" ]; then
      results_list="\"$result_path\""
    else
      results_list="$results_list, \"$result_path\""
    fi
  fi
done

finished_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
total=$((passed + failed))
status="pass"
if [ "$failed" -gt 0 ]; then
  status="fail"
fi

cat >"$run_dir/summary.json" <<JSON
{
  "run_id": "$RUN_ID",
  "status": "$status",
  "started_at": "$started_at",
  "finished_at": "$finished_at",
  "total_cases": $total,
  "passed_cases": $passed,
  "failed_cases": $failed,
  "case_result_files": [$results_list]
}
JSON

echo "Batch artifact: $run_dir/summary.json"

if [ "$failed" -gt 0 ]; then
  exit 1
fi

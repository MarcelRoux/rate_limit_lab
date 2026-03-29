#!/usr/bin/env sh
set -eu

COMPOSE_FILE="${COMPOSE_FILE:-docker/compose/compose.dev.yml}"
OBS_RUNS_DIR="${OBS_RUNS_DIR:-evaluations/obs_runs}"
OBS_CASE_REGISTRY="${OBS_CASE_REGISTRY:-configs/traffic_rest/observability/case_registry.tsv}"
CASE_INPUT="${1:-${CASE:-}}"
RUN_ID="${OBS_RUN_ID:-$(date +%Y%m%d_%H%M%S)_obs_case}"

usage() {
  echo "Missing case input. Usage:"
  echo "  make obs-case CASE=OBS-001"
  echo "  make obs-case CASE=configs/traffic_rest/observability/obs_002_single_key_low_rate_200x2_8s.toml"
}

if [ -z "$CASE_INPUT" ]; then
  usage
  exit 2
fi

if [ ! -f "$OBS_CASE_REGISTRY" ]; then
  echo "Case registry not found: $OBS_CASE_REGISTRY"
  exit 2
fi

case_id="$CASE_INPUT"
config_rel=""

if [ -f "$CASE_INPUT" ]; then
  config_rel="$CASE_INPUT"
  case_id="$(basename "$CASE_INPUT" .toml)"
elif printf '%s' "$CASE_INPUT" | grep -q '^/app/'; then
  config_rel="${CASE_INPUT#/app/}"
  case_id="$(basename "$config_rel" .toml)"
else
  # shellcheck disable=SC2039
  line="$(awk -F '\t' -v id="$CASE_INPUT" 'BEGIN{found=0} $0 !~ /^#/ && NF >= 2 && $1==id {print $0; found=1; exit} END{if(found==0) print ""}' "$OBS_CASE_REGISTRY")"
  if [ -z "$line" ]; then
    echo "Unknown case id: $CASE_INPUT"
    echo "Known IDs:"
    awk -F '\t' '$0 !~ /^#/ && NF >= 2 {print "  - " $1 " -> " $2}' "$OBS_CASE_REGISTRY"
    exit 2
  fi
  config_rel="$(printf '%s' "$line" | awk -F '\t' '{print $2}')"
  case_id="$CASE_INPUT"
fi

if [ ! -f "$config_rel" ]; then
  echo "Resolved config path not found: $config_rel"
  exit 2
fi

case "$config_rel" in
  /app/*)
    config_in_container="$config_rel"
    ;;
  *)
    config_in_container="/app/$config_rel"
    ;;
esac

run_dir="$OBS_RUNS_DIR/$RUN_ID"
case_dir="$run_dir/$case_id"
mkdir -p "$case_dir"

started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

set +e
docker compose -f "$COMPOSE_FILE" --profile observability run --rm traffic_rest_observability \
  rest_traffic --config "$config_in_container" >"$case_dir/traffic.log" 2>&1
exit_code=$?
set -e

finished_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

status="pass"
if [ "$exit_code" -ne 0 ]; then
  status="fail"
fi

cat "$case_dir/traffic.log"

if docker compose -f "$COMPOSE_FILE" --profile observability exec -T rest_observability \
  curl -fsS http://localhost:3000/metrics >"$case_dir/metrics_snapshot.prom" 2>/dev/null; then
  :
else
  echo "# metrics snapshot unavailable" >"$case_dir/metrics_snapshot.prom"
fi

if curl -fsS "http://127.0.0.1:9090/api/v1/query?query=up" >"$case_dir/prometheus_up.json" 2>/dev/null; then
  :
else
  echo '{"status":"unavailable"}' >"$case_dir/prometheus_up.json"
fi

cat >"$case_dir/result.json" <<JSON
{
  "run_id": "$RUN_ID",
  "case_id": "$case_id",
  "case_input": "$CASE_INPUT",
  "config_path": "$config_rel",
  "config_in_container": "$config_in_container",
  "status": "$status",
  "exit_code": $exit_code,
  "started_at": "$started_at",
  "finished_at": "$finished_at",
  "artifacts": {
    "traffic_log": "$case_dir/traffic.log",
    "metrics_snapshot": "$case_dir/metrics_snapshot.prom",
    "prometheus_up": "$case_dir/prometheus_up.json"
  }
}
JSON

cat >"$case_dir/README.txt" <<TXT
case_id=$case_id
status=$status
config=$config_rel
run_dir=$run_dir
TXT

echo "Case artifact: $case_dir/result.json"

if [ "$exit_code" -ne 0 ]; then
  exit "$exit_code"
fi

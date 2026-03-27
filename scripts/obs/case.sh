#!/usr/bin/env sh
set -eu

COMPOSE_FILE="${COMPOSE_FILE:-docker/compose/compose.dev.yml}"
CASE_INPUT="${1:-${CASE:-}}"

if [ -z "$CASE_INPUT" ]; then
  echo "Missing case path. Usage:"
  echo "  make obs-case CASE=configs/traffic_rest/smoke/observability_demo__single_key__steady__1000x4__5s.toml"
  exit 2
fi

case "$CASE_INPUT" in
  /app/*)
    CASE_PATH="$CASE_INPUT"
    ;;
  *)
    CASE_PATH="/app/$CASE_INPUT"
    ;;
esac

docker compose -f "$COMPOSE_FILE" --profile observability run --rm traffic_rest_observability \
  rest_traffic --config "$CASE_PATH"

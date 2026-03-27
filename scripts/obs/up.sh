#!/usr/bin/env sh
set -eu

COMPOSE_FILE="${COMPOSE_FILE:-docker/compose/compose.dev.yml}"
AUTO_OPEN_DASHBOARD="${AUTO_OPEN_DASHBOARD:-0}"

docker compose -f "$COMPOSE_FILE" --profile observability up -d --build rest_observability prometheus grafana

echo "Observability stack is up."
echo "Grafana:    http://127.0.0.1:3001"
echo "Prometheus: http://127.0.0.1:9090/targets"

if [ "$AUTO_OPEN_DASHBOARD" = "1" ]; then
  if command -v open >/dev/null 2>&1; then
    open "http://127.0.0.1:3001"
  elif command -v xdg-open >/dev/null 2>&1; then
    xdg-open "http://127.0.0.1:3001" >/dev/null 2>&1 || true
  fi
fi

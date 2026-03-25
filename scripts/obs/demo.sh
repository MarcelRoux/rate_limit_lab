#!/usr/bin/env sh
set -eu

COMPOSE_FILE="${COMPOSE_FILE:-docker/compose/compose.dev.yml}"
AUTO_OPEN_DASHBOARD="${AUTO_OPEN_DASHBOARD:-0}"

echo "1/3 Starting containerized observability stack (REST + Prometheus + Grafana)..."
docker compose -f "$COMPOSE_FILE" --profile observability up -d --build rest_observability prometheus grafana

echo "2/3 Simulating traffic from container..."
docker compose -f "$COMPOSE_FILE" --profile observability run --rm traffic_rest_observability

echo "3/3 Dashboard ready."
echo "Grafana:    http://127.0.0.1:3001"
echo "Prometheus: http://127.0.0.1:9090/targets"
echo "To stop:    make obs-demo-down"

if [ "$AUTO_OPEN_DASHBOARD" = "1" ]; then
  if command -v open >/dev/null 2>&1; then
    open "http://127.0.0.1:3001"
  elif command -v xdg-open >/dev/null 2>&1; then
    xdg-open "http://127.0.0.1:3001" >/dev/null 2>&1 || true
  fi
fi

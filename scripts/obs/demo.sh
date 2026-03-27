#!/usr/bin/env sh
set -eu

DEFAULT_CASE="configs/traffic_rest/smoke/observability_demo__single_key__steady__1000x4__5s.toml"

echo "1/3 Bringing observability stack up..."
./scripts/obs/up.sh

echo "2/3 Running default case: $DEFAULT_CASE"
./scripts/obs/case.sh "$DEFAULT_CASE"

echo "3/3 Done. Stack remains up for dashboard inspection."
echo "To stop: make obs-demo-down"

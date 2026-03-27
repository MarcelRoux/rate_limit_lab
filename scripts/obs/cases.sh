#!/usr/bin/env sh
set -eu

CASES_INPUT="${1:-${CASES:-}}"

if [ -z "$CASES_INPUT" ]; then
  echo "Missing CASES list. Usage:"
  echo "  make obs-cases CASES=\"configs/traffic_rest/smoke/case_a.toml configs/traffic_rest/smoke/case_b.toml\""
  exit 2
fi

for case_path in $CASES_INPUT; do
  echo "Running case: $case_path"
  ./scripts/obs/case.sh "$case_path"
done

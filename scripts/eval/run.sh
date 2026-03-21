#!/usr/bin/env sh
set -eu

MODE="${1:-}"
AT_ID="${2:-}"

if [ -z "$MODE" ]; then
  echo "Usage: $0 <smoke|full|one> [AT-00X]"
  exit 2
fi

if cargo metadata --no-deps --format-version 1 | grep -q '"name":"eval_harness"'; then
  :
else
  echo "eval_harness crate is not implemented yet."
  echo "See docs/evaluation/shortcomings_and_remediation.md (Phase 1 / H-001..H-008)."
  exit 2
fi

case "$MODE" in
  smoke)
    cargo run -p eval_harness -- run --profile smoke_ready --repeat 2
    ;;
  full)
    cargo run -p eval_harness -- run --profile full_matrix --repeat 2
    ;;
  one)
    if [ -z "$AT_ID" ]; then
      echo "Missing AT id. Usage: make ac-one AT=AT-00X"
      exit 2
    fi
    cargo run -p eval_harness -- run --at "$AT_ID"
    ;;
  *)
    echo "Unknown mode '$MODE'. Use smoke, full, or one."
    exit 2
    ;;
esac

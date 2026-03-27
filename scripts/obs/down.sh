#!/usr/bin/env sh
set -eu

COMPOSE_FILE="${COMPOSE_FILE:-docker/compose/compose.dev.yml}"

docker compose -f "$COMPOSE_FILE" --profile observability down --remove-orphans

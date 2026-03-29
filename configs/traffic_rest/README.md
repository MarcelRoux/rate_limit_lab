# REST Traffic Config Loading

`rest_traffic` loads TOML config via `--config <path>`.

## Run with a config file

```bash
cargo run -p traffic_rest --bin rest_traffic -- \
  --config configs/traffic_rest/smoke/smoke__single_key__steady__1000x4__5s.toml
```

## Notes

- If `--config` is omitted, built-in defaults are used.
- Traffic config controls workload only (RPS, concurrency, key mode, duration, target URL).
- Pair this with a server config from `configs/rest_server/`.

## Observability case registry

- Registry file: `configs/traffic_rest/observability/case_registry.tsv`
- Run one case by stable ID:

```bash
make obs-case CASE=OBS-001
```

- Run a batch:

```bash
make obs-cases CASES="OBS-001 OBS-002 OBS-003"
```

## Minimum schema

```toml
target_url = "http://127.0.0.1:3000/"
duration_secs = 5
requests_per_second = 1000
concurrency = 4
key_header = "x-api-key"

[key_mode]
mode = "single_key" # keyless | single_key | round_robin
key = "user1"
```

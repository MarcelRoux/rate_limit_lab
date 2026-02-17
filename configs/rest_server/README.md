# REST Server Config Loading

`rest_server` loads TOML config via `--config <path>`.

## In-memory limiter

```bash
cargo run -p rest --bin rest_server -- \
  --config configs/rest_server/in_memory.toml
```

## Distributed limiter

Requires `REDIS_URL`.

```bash
REDIS_URL=redis://127.0.0.1:6379 \
cargo run -p rest --bin rest_server --no-default-features --features distributed_limiter -- \
  --config configs/rest_server/distributed.toml
```

## Hybrid limiter

Requires `REDIS_URL`.

```bash
REDIS_URL=redis://127.0.0.1:6379 \
cargo run -p rest --bin rest_server --no-default-features --features hybrid_limiter -- \
  --config configs/rest_server/hybrid.toml
```

## Notes

- Exactly one limiter feature must be enabled.
- If `--config` is omitted, built-in defaults are used.

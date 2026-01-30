# Dev Services

This repo uses Docker Compose to run supporting services locally (e.g., Redis for distributed rate limiting tests).

Compose file:
- `docker/compose/compose.dev.yml`

## Development Environment Variables

This project uses environment variables to configure local services.  
Local dev uses `docker/env/dev.env` (created from sample). CI sets env vars directly in the pipeline config.

### Setup

This creates docker/env/dev.env from the committed template.

```sh
make env-init
```

## Quick start (Redis)

Start Redis (exposes `6379` on localhost):

```bash
make redis-up
```

Tail logs:

```bash
make redis-logs
```


Stop Redis:

```bash
make redis-down
```

Remove containers + volumes (reset state):

```bash
make reset
```

## Running Redis integration tests

This runs the feature-gated integration tests in crates/state_backend/tests/*::

```bash
make test-redis-backend
```

Equivalent command:

```bash
REDIS_URL=redis://127.0.0.1:6379 cargo test -p state_backend --features redis-tests
```

## Valkey (planned)

Start Valkey (exposes 6380 on localhost, mapped to container 6379):

```bash
make valkey-up
```

Tail logs:

```bash
make valkey-logs
```

Stop Valkey:

```bash
make valkey-down
```

## Notes

•	Services are enabled using Compose profiles; nothing runs unless explicitly started.  
•	Default Redis URL used by tests:  
•	redis://127.0.0.1:6379  
•	Override via environment variable:  
•	REDIS_URL=redis://...  
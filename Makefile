COMPOSE_FILE := docker/compose/compose.dev.yml
ENV_FILE := docker/env/dev.env

# Load env vars from ENV_FILE into Make (exported to commands).
# - ignores comments/blank lines
# - works on macOS make
ifneq ("$(wildcard $(ENV_FILE))","")
include $(ENV_FILE)
export $(shell sed -n 's/^\([A-Za-z_][A-Za-z0-9_]*\)=.*/\1/p' $(ENV_FILE))
endif

.PHONY: help
help:
	@echo "Dev services:"
	@echo "  make redis-up       Start Redis (6379)"
	@echo "  make redis-down     Stop Redis"
	@echo "  make redis-logs     Tail Redis logs"
	@echo "  make valkey-up      Start Valkey (6380->6379)"
	@echo "  make valkey-down    Stop Valkey"
	@echo "  make valkey-logs    Tail Valkey logs"
	@echo "  make obs-up         Start Prometheus + Grafana (9090, 3001)"
	@echo "  make obs-down       Stop observability services"
	@echo "  make obs-logs       Tail observability service logs"
	@echo "  make down           Stop all services started from this compose file"
	@echo "  make reset          Stop all + remove volumes"
	@echo ""
	@echo "Testing:"
	@echo "  make test-all-targets     Run all integration tests - no features"
	@echo "  make test-redis-backend   Run state_backend Redis integration tests"
	@echo "  make ac                   Run acceptance smoke profile (Rust harness)"
	@echo "  make ac-full              Run acceptance full implemented profile (Rust harness)"
	@echo "  make ac-one AT=AT-00X     Run a single acceptance test id (Rust harness)"
	@echo ""
	@echo "Environment:"
	@echo "  make env                Show environment variables"
	@echo "  make env-init           Initialize dev.env from sample"

.PHONY: redis-up
redis-up:
	docker compose -f $(COMPOSE_FILE) --profile redis up -d redis

.PHONY: redis-down
redis-down:
	docker compose -f $(COMPOSE_FILE) --profile redis down

.PHONY: redis-logs
redis-logs:
	docker compose -f $(COMPOSE_FILE) logs -f redis

.PHONY: valkey-up
valkey-up:
	docker compose -f $(COMPOSE_FILE) --profile valkey up -d valkey

.PHONY: valkey-down
valkey-down:
	docker compose -f $(COMPOSE_FILE) --profile valkey down

.PHONY: valkey-logs
valkey-logs:
	docker compose -f $(COMPOSE_FILE) logs -f valkey

.PHONY: down
down:
	docker compose -f $(COMPOSE_FILE) down

.PHONY: obs-up
obs-up:
	docker compose -f $(COMPOSE_FILE) --profile observability up -d prometheus grafana

.PHONY: obs-down
obs-down:
	docker compose -f $(COMPOSE_FILE) --profile observability down

.PHONY: obs-logs
obs-logs:
	docker compose -f $(COMPOSE_FILE) logs -f prometheus grafana

.PHONY: reset
reset:
	docker compose -f $(COMPOSE_FILE) down -v

.PHONY: test-all-targets
test-all-targets:
	cargo test --all-targets

.PHONY: test-redis-backend
test-redis-backend:
	cargo test -p state_backend --features redis-tests

.PHONY: env
env:
	@echo "ENV_FILE=$(ENV_FILE)"
	@echo "REDIS_URL=$(REDIS_URL)"

.PHONY: env-init
env-init:
	@if [ ! -f docker/env/dev.env ]; then \
		cp docker/env/dev.env.sample docker/env/dev.env; \
		echo "Created docker/env/dev.env from sample"; \
	else \
		echo "docker/env/dev.env already exists"; \
	fi

.PHONY: ac
ac:
	./scripts/eval/run.sh smoke

.PHONY: ac-full
ac-full:
	./scripts/eval/run.sh full

.PHONY: ac-one
ac-one:
	./scripts/eval/run.sh one "$(AT)"

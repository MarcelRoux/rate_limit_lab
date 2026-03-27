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
	@echo "  make obs-up         Start containerized observability stack (REST + Prometheus + Grafana)"
	@echo "  make obs-case       Run one traffic case against running observability stack (CASE=...)"
	@echo "  make obs-cases      Run multiple traffic cases against running stack (CASES=\"...\")"
	@echo "  make obs-down       Stop containerized observability stack"
	@echo "  make obs-logs       Tail observability service logs"
	@echo "  make obs-demo       Start containerized REST + Prometheus + Grafana, then run traffic"
	@echo "  make obs-demo-down  Stop containerized observability demo services"
	@echo "  make down           Stop all services started from this compose file"
	@echo "  make reset          Stop all + remove volumes"
	@echo ""
	@echo "Testing:"
	@echo "  make test-all-targets     Run all integration tests - no features"
	@echo "  make test-redis-backend   Run state_backend Redis integration tests"
	@echo "  make ac                   Run acceptance smoke profile (Rust harness)"
	@echo "  make ac-full              Run acceptance full profile (Rust harness)"
	@echo "  make ac-obs               Run observability MVP profile with live checks (Rust harness)"
	@echo "  make ac-one AT=AT-00X     Run a single acceptance test id (Rust harness)"
	@echo "  make report               Compile all run artifacts into report outputs"
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
	./scripts/obs/up.sh

.PHONY: obs-case
obs-case:
	./scripts/obs/case.sh "$(CASE)"

.PHONY: obs-cases
obs-cases:
	./scripts/obs/cases.sh "$(CASES)"

.PHONY: obs-down
obs-down:
	./scripts/obs/down.sh

.PHONY: obs-logs
obs-logs:
	docker compose -f $(COMPOSE_FILE) logs -f rest_observability prometheus grafana

.PHONY: obs-demo
obs-demo:
	./scripts/obs/demo.sh

.PHONY: obs-demo-down
obs-demo-down:
	./scripts/obs/demo_down.sh

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

.PHONY: ac-obs
ac-obs:
	./scripts/eval/run.sh obs

.PHONY: ac-one
ac-one:
	./scripts/eval/run.sh one "$(AT)"

.PHONY: report
report:
	cargo run -p eval_harness -- compile --input evaluations/runs --output evaluations/reports

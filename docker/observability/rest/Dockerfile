FROM rust:1.87-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY configs ./configs

RUN cargo build -p rest --features in_memory_limiter,observability_ui --bin rest_server \
    && cargo build -p traffic_rest --bin rest_traffic

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/debug/rest_server /usr/local/bin/rest_server
COPY --from=builder /app/target/debug/rest_traffic /usr/local/bin/rest_traffic
COPY --from=builder /app/configs /app/configs

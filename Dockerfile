FROM rust:1.75-bookworm AS builder

WORKDIR /build

RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -u 1000 appuser

WORKDIR /app

COPY --from=builder /build/target/release/cortex /usr/local/bin/

USER appuser

ENV CORTEX_SURREALDB_URL=ws://surrealdb:8000
ENV CORTEX_SURREALDB_USERNAME=root
ENV CORTEX_SURREALDB_PASSWORD=root
ENV CORTEX_SURREALDB_NS=agentrag
ENV CORTEX_SURREALDB_DB=system3

EXPOSE 8003

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD sh -c 'exec 3<>/dev/tcp/127.0.0.1/8003' || exit 1

ENTRYPOINT ["cortex"]

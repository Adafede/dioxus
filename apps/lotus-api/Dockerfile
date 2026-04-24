# ── Stage 1: build ────────────────────────────────────────────────────────────
FROM rust:1-slim-bookworm AS builder

WORKDIR /build

# Copy workspace manifests first so dependency layers are cached
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY apps/lotus-api/ apps/lotus-api/
COPY apps/lotus-explorer/src/models.rs  apps/lotus-explorer/src/models.rs
COPY apps/lotus-explorer/src/queries.rs apps/lotus-explorer/src/queries.rs
COPY apps/lotus-explorer/src/sparql.rs  apps/lotus-explorer/src/sparql.rs
# hello-world and lotus-explorer need to be present for the workspace to resolve
COPY apps/hello-world/  apps/hello-world/
COPY apps/lotus-explorer/ apps/lotus-explorer/

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN cargo build --release -p lotus-api

# ── Stage 2: runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/lotus-api /usr/local/bin/lotus-api

# Bind to all interfaces by default when running in a container
ENV HOST=0.0.0.0
ENV PORT=8787

EXPOSE 8787

CMD ["lotus-api"]


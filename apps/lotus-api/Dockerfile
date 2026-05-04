# ── Stage 1: build ────────────────────────────────────────────────────────────
# Pin to the same channel as rust-toolchain.toml so the build is reproducible.
FROM rust:1.95.0-slim-bookworm AS builder

WORKDIR /build

# Copy workspace manifests first so dependency layers are cached
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY apps/lotus-api/ apps/lotus-api/
COPY apps/lotus-explorer/src/models.rs  apps/lotus-explorer/src/models.rs
COPY apps/lotus-explorer/src/queries.rs apps/lotus-explorer/src/queries.rs
COPY apps/lotus-explorer/src/sparql.rs  apps/lotus-explorer/src/sparql.rs
# hello-world and lotus-explorer must be present for the workspace to resolve
COPY apps/hello-world/  apps/hello-world/
COPY apps/lotus-explorer/ apps/lotus-explorer/

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# --locked ensures Cargo.lock is respected exactly (no silent upgrades)
RUN cargo build --release --locked -p lotus-api

# ── Stage 2: runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim

# ── OCI image labels ─────────────────────────────────────────────────────────
LABEL org.opencontainers.image.title="lotus-api" \
      org.opencontainers.image.description="OpenAPI service for LOTUS Wikidata natural-product search and export" \
      org.opencontainers.image.url="https://github.com/adafede/dioxus" \
      org.opencontainers.image.source="https://github.com/adafede/dioxus" \
      org.opencontainers.image.licenses="AGPL-3.0" \
      org.opencontainers.image.vendor="adafede"

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Run as a non-root user — principle of least privilege
RUN adduser --system --no-create-home --uid 1001 appuser

COPY --from=builder /build/target/release/lotus-api /usr/local/bin/lotus-api

# Bind to all interfaces by default when running in a container
ENV HOST=0.0.0.0
ENV PORT=8787

EXPOSE 8787

USER appuser

# Liveness probe — requires the /health endpoint to respond
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider "http://localhost:${PORT}/health" || exit 1

CMD ["lotus-api"]


# Root task runner for the dioxus workspace.
# Run `just --list` to see available recipes.
#
# Web apps (use `just serve`/`just build` with one of these):
#   cxsmiles-yoga  index  json-count-rs  lipid-selecto-rs
#   mgf-precursor-erro-rs  lotus-explore-rs  smellfish-rs

# ── Workspace gate (mirrors .github/workflows/ci.yml) ─────────────────────────

fmt:
	cargo fmt --all -- --check

check:
	cargo check --workspace --all-targets --locked

clippy:
	cargo clippy --workspace --all-targets --locked -- -D warnings

test:
	cargo test --workspace --all-targets --locked --quiet

doc:
	cargo doc --workspace --no-deps --locked

# WASM apps only — never `--workspace` (lotus-api/axum is non-wasm).
wasm:
	cargo check -p cxsmiles-yoga --target wasm32-unknown-unknown --locked
	cargo check -p index --target wasm32-unknown-unknown --locked
	cargo check -p json-count-rs --target wasm32-unknown-unknown --locked
	cargo check -p mgf-precursor-erro-rs --target wasm32-unknown-unknown --locked
	cargo check -p lipid-selecto-rs --target wasm32-unknown-unknown --locked
	cargo check -p lotus-explore-rs --target wasm32-unknown-unknown --locked
	cargo check -p smellfish-rs --target wasm32-unknown-unknown --locked

# ── Per-app dev servers / production builds ───────────────────────────────────

serve app:
	dx serve --package {{app}}

build app:
	dx build --release --package {{app}}

# ── Native services ───────────────────────────────────────────────────────────

lotus-api:
	cargo run --locked -p lotus-api

# ── Supply-chain hygiene ──────────────────────────────────────────────────────

machete:
	cargo machete

deny:
	cargo deny check advisories bans licenses sources

audit:
	cargo audit

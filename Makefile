# Makefile — unified quality gate for the dioxus-apps workspace
#
# Run everything with:  make all
# Run a single stage with:  make check, make test, make clippy, etc.
#
# This supersedes the manual "run these 10 commands" workflow;
# `prek run cargo-qa` delegates here for the Rust-specific checks.

.PHONY: all check clippy test fmt fmt-check doc test-cov audit deny outdated machete outdated-deps udeps readme-generate clean

# ── Default: run the full quality gate ────────────────────────────────────────

all: check clippy test fmt-check doc deny audit

# ── Compilation ───────────────────────────────────────────────────────────────

check:
	cargo check --workspace --all-targets --locked

# ── Linting ───────────────────────────────────────────────────────────────────

clippy:
	cargo clippy --workspace --all-targets --locked -- -D warnings

fmt-check:
	cargo fmt --all -- --check

fmt:
	cargo fmt --all

# ── Tests ─────────────────────────────────────────────────────────────────────

test:
	cargo test --workspace --all-targets --locked

# ── Coverage (native targets only — WASM not supported by tarpaulin) ──────────

test-cov:
	cargo tarpaulin -p lotus -p lotus-api -p ui --locked --out lcov --out html

test-cov-lotus:
	cargo tarpaulin -p lotus --locked --out html

test-cov-upload:
	cargo tarpaulin -p upload --locked --out html

# ── Documentation ─────────────────────────────────────────────────────────────

doc:
	cargo doc --workspace --no-deps --locked

# ── Security / Licenses ───────────────────────────────────────────────────────

audit:
	cargo audit

deny:
	cargo deny check licenses

# ── Dependency hygiene ────────────────────────────────────────────────────────

outdated:
	cargo outdated

machete:
	cargo machete --workspace

udeps:
	cargo udeps --workspace --all-targets

# ── README generation (cargo-readme) ──────────────────────────────────────────

readme-generate:
	(cd crates/lotus && cargo readme -t README.tpl -o README.md)
	(cd crates/ui && cargo readme -t README.tpl -o README.md)
	(cd crates/upload && cargo readme -t README.tpl -o README.md)

# ── Combined dependency management ────────────────────────────────────────────

deps-upgrade:
	cargo upgrade --workspace --locked
	cargo update --workspace
	cargo machete --workspace --fix
	cargo deny check licenses

# ── Cleanup ───────────────────────────────────────────────────────────────────

clean:
	cargo clean

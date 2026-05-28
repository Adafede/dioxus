APP ?= lotus-explorer

.PHONY: serve build check test test-verbose clippy fmt fmt-check qa deny audit supply-chain clean doc help

## Display this help message
help:
	@grep -E '^## ' Makefile | sed 's/## //'

## Run an app in dev mode (hot-reload)
serve:
	dx serve --package $(APP)

## Production WASM build
build:
	dx build --release --package $(APP)

## Type-check the whole workspace without building WASM
check:
	cargo check --workspace --all-targets --locked

## Run workspace tests
test:
	cargo test --workspace --all-targets --locked

## Run tests with output (useful for debugging)
test-verbose:
	cargo test --workspace --all-targets --locked -- --nocapture

## Lint all targets with warnings denied
clippy:
	cargo clippy --workspace --all-targets --locked -- -D warnings

## Format all code
fmt:
	cargo fmt --all

## Validate formatting only
fmt-check:
	cargo fmt --all -- --check

## Build documentation with dependencies visible
doc:
	cargo doc --workspace --all-features --no-deps --locked

## CI-equivalent quality gate: format, check, test, lint
qa: fmt-check check test clippy
	cargo check -p lotus-explorer --target wasm32-unknown-unknown --locked
	cargo doc --workspace --no-deps --locked

## Dependency advisories, bans, licenses, sources (requires cargo-deny)
deny:
	cargo deny check advisories bans licenses sources

## RustSec vulnerability audit (requires cargo-audit)
audit:
	cargo audit

## Full supply-chain gate (requires cargo-deny and cargo-audit)
supply-chain: deny audit

## Remove all build artefacts
clean:
	cargo clean
	find apps -name dist -type d -exec rm -rf {} +

## List available apps
list:
	@ls apps/

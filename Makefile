APP ?= lotus-explorer

.PHONY: serve build check fmt clean

## Run an app in dev mode (hot-reload)
serve:
	dx serve --package $(APP)

## Production WASM build
build:
	dx build --release --package $(APP)

## Type-check the whole workspace without building WASM
check:
	cargo check --target wasm32-unknown-unknown

## Format all code
fmt:
	cargo fmt --all

## Remove all build artefacts
clean:
	cargo clean
	find apps -name dist -type d -exec rm -rf {} +

## List available apps
list:
	@ls apps/

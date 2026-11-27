# lipid-selecto-rs

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![CI](https://github.com/Adafede/dioxus/actions/workflows/ci.yml/badge.svg)](https://github.com/Adafede/dioxus/actions/workflows/ci.yml)

`lipid-selecto-rs` --- browser-based lipid classification and filtering.

Thin Dioxus WebAssembly UI over `lipid-core` (MGF/SMILES extraction, compiled
LIPID MAPS SMARTS classification, gallery projection, and filtered exports).

## Usage

### Locally

```bash
dx serve --package lipid-selecto-rs
```

### Build for deployment

```bash
dx build --profile wasm-release --platform web --package lipid-selecto-rs
```

### Tests

```bash
cargo nextest run -p lipid-core -p lipid-selecto-rs --locked
```

## License

`AGPL-3.0-only` --- see
[`LICENSE`](https://github.com/Adafede/dioxus/blob/main/LICENSE) for details.

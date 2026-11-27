# mgf-precursor-erro-rs

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![CI](https://github.com/Adafede/dioxus/actions/workflows/ci.yml/badge.svg)](https://github.com/Adafede/dioxus/actions/workflows/ci.yml)

`mgf-precursor-erro-rs` --- MGF precursor mass-error analysis.

Thin Dioxus WebAssembly UI and SVG plotting adapter over [`mgf_analysis_core`],
streaming MGF spectra, recalibrating precursor and fragment *m/z* values, and
rendering interactive mass-error diagnostics.

## Run locally

```bash
dx serve --package mgf-precursor-erro-rs
```

## Build for the website

```bash
dx build --profile wasm-release --platform web --package mgf-precursor-erro-rs
```

## License

`AGPL-3.0-only` --- see
[`LICENSE`](https://github.com/Adafede/dioxus/blob/main/LICENSE) for details.

# mgf-precursor-erro-rs

[![AGPL-3.0 license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![Tests](https://img.shields.io/badge/tests-34-brightgreen)](https://github.com/adafede/dioxus/actions)

`mgf-precursor-erro-rs` — MGF precursor mass-error analysis.

Uploads an MGF file, recalibrates precursor *m/z* values, and visualises
the resulting mass-error distribution as an interactive histogram.

### Run locally

```bash
dx serve --package mgf-precursor-erro-rs
```

### Build for the website

```bash
dx build --release --platform web --package mgf-precursor-erro-rs
```

## License

`AGPL-3.0-only` — see [`LICENSE`](https://www.gnu.org/licenses/agpl-3.0.html) for details.

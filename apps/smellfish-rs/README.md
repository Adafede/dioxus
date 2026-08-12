# smellfish-rs

[![AGPL-3.0 license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![Tests](https://img.shields.io/badge/tests-20-brightgreen)](https://github.com/adafede/dioxus/actions)

`smellfish-rs` — literature-backed NP-likeness scoring.

Scores natural-product-likeness of SMILES structures using machine-learned
features, `Query` enrichment, and RDKit.js chemistry descriptors.

### Run locally

```bash
dx serve --package smellfish-rs
```

### Build for deployment

```bash
dx build --release --platform web --package smellfish-rs
```

## License

`AGPL-3.0-only` — see [`LICENSE`](https://www.gnu.org/licenses/agpl-3.0.html) for details.

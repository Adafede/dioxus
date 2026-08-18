# lipid-selecto-rs

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![Tests](https://img.shields.io/badge/tests-33-brightgreen)](https://github.com/adafede/dioxus/actions)

`lipid-selecto-rs` --- interactive lipid classification and filtering.

**A modern, interactive web application for filtering and visualizing lipid mass
spectrometry data.**

Analyzes mass spectrometry data (MGF or SMILES), classifies molecules using
LIPID MAPS-aligned SMARTS patterns, and lets you download the lipid-only subset
in the same format. Built with pure Rust using WebAssembly for blazing-fast
performance in the browser.

## Features

- 30+ built-in SMARTS rules covering all 8 LIPID MAPS families (FA, GL, GP, SP,
  ST, PR, SL, PK)
- Supports evolved SMARTS rules from
  [`smarts-evoliposuction`](./lipidmaps/smarts-evoliposuction/) --- load
  high-precision GA-evolved patterns from `smarts_results.csv` via
  `LipidRuleLibrary::add_evolved_rules_from_csv()`
- Auto-detects input format: MGF or SMILES list; preserves output format
- Real-time class selection with live gallery/count updates
- 2D molecular structure rendering (no external rendering service)
- Download filtered output (MGF or SMILES) with all metadata preserved
- 100 curated example lipids from LIPID MAPS

## Usage

### Online

Visit [lipid-selecto-rs.princelab.org](https://lipid-selecto-rs.princelab.org)
to use the web app.

### Locally

```bash
dx serve --package lipid-selecto-rs
```

### Build for deployment

```bash
dx build --release --platform web --package lipid-selecto-rs
```

### Tests

```bash
cargo test --lib -p lipid-selecto-rs
```

## Classification strategy

Rules are **fully configurable** in YAML. See
[`RULES_GUIDE.md`](./RULES_GUIDE.md) for adding custom lipid classes, SMARTS
pattern syntax, and rule priority configuration.

### Evolved SMARTS rules

In addition to the built-in rules, you can load [GA-evolved SMARTS
patterns](./lipidmaps/README.md) produced by the `smarts-evoliposuction` binary:

```rust
use lipid_selecto_rs::rules::LipidRuleLibrary;

let mut library = LipidRuleLibrary::defaults();
library.add_evolved_rules_from_csv("lipidmaps/smarts_results.csv")?;
```

Evolved rules are pre-compiled at load time and include MCC/coverage metrics in
their descriptions. See [`lipidmaps/README.md`](./lipidmaps/README.md) for the
full end-to-end workflow: download LIPID MAPS data → split into
positive/negative SMILES pairs → evolve SMARTS via GA → load into this app.

## Dependencies

- [`chematic`](https://crates.io/crates/chematic) --- Pure-Rust SMILES/SMARTS
- [`dioxus`](https://dioxuslabs.com) --- UI framework (compiles to WebAssembly)
- [`web-sys`](https://crates.io/crates/web-sys) --- Browser API bindings

No external services or native binaries required. All processing happens in the
browser --- your data never leaves your computer.

## License

`AGPL-3.0-only` --- see [`LICENSE`](https://www.gnu.org/licenses/agpl-3.0.html)
for details.

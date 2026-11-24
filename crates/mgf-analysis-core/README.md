# mgf-analysis-core

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![CI](https://github.com/Adafede/dioxus/actions/workflows/ci.yml/badge.svg)](https://github.com/Adafede/dioxus/actions/workflows/ci.yml)

Pure-Rust MGF precursor mass-error analysis, adduct grammar, MS2 recalibration,
and summary diagnostics.

Zero Dioxus, browser, or plotter dependencies: feeds the thin WASM shell and
native/host CLI or example runners with deterministic math and streaming block
ingestion backed by [`mgf`].

## Example

```rust
use mgf_analysis_core::{
    CalibrationModel, compute_recalibration_diagnostics, generate_recalibrated_mgf,
    ingest_lines,
};

let mgf_text = "\
BEGIN IONS
TITLE=caffeine_test
PEPMASS=195.0880
FORMULA=C8H10N4O2
ADDUCT=[M+H]+
195.0980 5000.0
138.0660 2500.0
END IONS
";
let stats = ingest_lines(mgf_text.lines(), |_| {});
assert_eq!(stats.spectra, 1);

let model = CalibrationModel::tof(1.0);
let diag = compute_recalibration_diagnostics(&stats, model).expect("active model");
assert_eq!(diag.total_count, 1);
assert!(generate_recalibrated_mgf(mgf_text, model).contains("138.055"));
```

## License

`AGPL-3.0-only` --- see
[`LICENSE`](https://github.com/Adafede/dioxus/blob/main/LICENSE) for details.

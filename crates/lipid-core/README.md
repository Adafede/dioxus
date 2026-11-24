# lipid-core

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![CI](https://github.com/Adafede/dioxus/actions/workflows/ci.yml/badge.svg)](https://github.com/Adafede/dioxus/actions/workflows/ci.yml)

Pure-Rust LIPID MAPS lipid classification, MGF/SMILES extraction, and export
filtering.

Moves the core chemistry, SMARTS matching, MGF block state machine, metadata
tagging, and export builders out of the Dioxus browser shell.

## Example

```rust
use lipid_core::{analyze, lmsd_classes};

let input = "CCCCCCCCCCCCCCCC(=O)O\tpalmitic_acid\nCCO\tethanol\n";
let (blocks, summary) = analyze(input);
assert_eq!(summary.total_items, 2);
assert_eq!(summary.lipid_items, 1);
assert!(blocks[0].is_lipid());
assert_eq!(lmsd_classes().len(), 86);
```

## License

`AGPL-3.0-only` --- see
[`LICENSE`](https://github.com/Adafede/dioxus/blob/main/LICENSE) for details.

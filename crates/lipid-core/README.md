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
use lipid_core::{
    BlockExtractor, build_analysis_from_classified, classify_blocks, lmsd_classes,
};

let input = "CCCCCCCCCCCCCCCC(=O)O\tpalmitic_acid\nCCO\tethanol\n";
let mut extractor = BlockExtractor::default();
for line in input.lines() {
    extractor.push_line(line);
}
let mut blocks = extractor.finish();
let classes = lmsd_classes();
classify_blocks(&mut blocks, classes);
let analysis = build_analysis_from_classified(blocks, 16, classes);
assert_eq!(analysis.summary.total_items, 2);
assert_eq!(analysis.summary.lipid_items, 1);
assert!(analysis.blocks[0].is_lipid());
assert_eq!(classes.len(), 86);
```

## License

`AGPL-3.0-only` --- see
[`LICENSE`](https://github.com/Adafede/dioxus/blob/main/LICENSE) for details.

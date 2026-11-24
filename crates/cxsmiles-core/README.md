# cxsmiles-core

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![CI](https://github.com/Adafede/dioxus/actions/workflows/ci.yml/badge.svg)](https://github.com/Adafede/dioxus/actions/workflows/ci.yml)

Pure-Rust CX-SMILES generation engine (`m:` positional blocks and `Sg:n:`
repeats).

Pipeline (`generate` / `generate_with_options`): 1. Parse non-empty SMILES lines
into molecules. 2. Cluster candidates by ECFP4/Tanimoto similarity. 3. Detect
repeating units (`Sg:n:`) or fall back to MCS positional diffs (`m:`). 4. Expand
and canonicalise the resulting CX-SMILES to compute round-trip recall.

## Example

```rust
use cxsmiles_core::{ConstructKind, generate};

let inputs = vec![
    "Clc1ccccc1-c2ccccc2".to_string(),
    "Clc1cccc(-c2ccccc2)c1".to_string(),
    "Clc1ccc(-c2ccccc2)cc1".to_string(),
];
let res = generate(&inputs).expect("valid biphenyl series");
assert_eq!(res.construct, ConstructKind::Positional);
assert!(res.confidence.clean);
assert!(res.cx_smiles.contains("m:"));
```

## License

`AGPL-3.0-only` --- see
[`LICENSE`](https://github.com/Adafede/dioxus/blob/main/LICENSE) for details.

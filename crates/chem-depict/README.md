# chem-depict

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![CI](https://github.com/Adafede/dioxus/actions/workflows/ci.yml/badge.svg)](https://github.com/Adafede/dioxus/actions/workflows/ci.yml)

Panic-guarded `chematic` SMILES parsing and local 2D SVG depiction.

Centralizes `std::panic::catch_unwind` recovery around `chematic::smiles` and
`chematic::depict` so WASM apps fall back cleanly instead of aborting.

## Example

```rust
use chem_depict::{parse_smiles_safe, render_smiles_svg};

assert!(parse_smiles_safe("CCO").is_ok());
assert!(parse_smiles_safe("   ").is_err());
let svg = render_smiles_svg("CCO");
assert!(svg.trim_start().starts_with("<svg"));
```

## License

`AGPL-3.0-only` --- see
[`LICENSE`](https://github.com/Adafede/dioxus/blob/main/LICENSE) for details.

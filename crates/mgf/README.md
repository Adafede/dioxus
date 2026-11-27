# mgf

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![CI](https://github.com/Adafede/dioxus/actions/workflows/ci.yml/badge.svg)](https://github.com/Adafede/dioxus/actions/workflows/ci.yml)

Streaming MGF (`Mascot Generic Format`) block parser and spectrum model.

Provides `BEGIN IONS ... END IONS` line-by-line parsing with first-line UTF-8
BOM stripping, case-insensitive header keys, positive-m/z peak validation,
returning completed blocks at `END IONS` or a nested `BEGIN IONS`.

## Example

```rust
use mgf::MgfLineParser;

let mut parser = MgfLineParser::new();
let mut blocks = Vec::new();
for line in ["BEGIN IONS", "PEPMASS=400.25 1000.0", "150.1 50.0", "END IONS"] {
    if let Some(block) = parser.push_line(line) {
        blocks.push(block);
    }
}
assert_eq!(blocks.len(), 1);
assert_eq!(blocks[0].header.effective_precursor_mz(), Some(400.25));
assert_eq!(blocks[0].peaks.len(), 1);
```

## License

`AGPL-3.0-only` --- see
[`LICENSE`](https://github.com/Adafede/dioxus/blob/main/LICENSE) for details.

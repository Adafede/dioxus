# mgf

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![CI](https://github.com/Adafede/dioxus/actions/workflows/ci.yml/badge.svg)](https://github.com/Adafede/dioxus/actions/workflows/ci.yml)

Streaming MGF (`Mascot Generic Format`) block parser and spectrum model.

Unifies `BEGIN IONS ... END IONS` line-by-line parsing across the workspace with
explicit options for UTF-8 BOM stripping, header-key case sensitivity, inline
`BEGIN IONS KEY=VALUE` headers, peak line validation, and unterminated-EOF block
handling.

## Example

```rust
use mgf::{MgfLineParser, MgfOptions};

let mut parser = MgfLineParser::new(MgfOptions::standard());
for line in ["BEGIN IONS", "PEPMASS=400.25 1000.0", "150.1 50.0", "END IONS"] {
    if let Some(block) = parser.push_line(line) {
        assert_eq!(block.header.effective_precursor_mz(), Some(400.25));
        assert_eq!(block.peaks.len(), 1);
    }
}
assert!(parser.finish().is_none());
```

## License

`AGPL-3.0-only` --- see
[`LICENSE`](https://github.com/Adafede/dioxus/blob/main/LICENSE) for details.

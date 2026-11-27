# json-count-core

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![CI](https://github.com/Adafede/dioxus/actions/workflows/ci.yml/badge.svg)](https://github.com/Adafede/dioxus/actions/workflows/ci.yml)

Portable streaming RFC 8259 JSON non-null field counter and result sorter.

Counts non-null values per top-level key over any [`stream_io::ChunkSource`]
with bounded memory and reusable scratch buffers across array rows.

## Example

```rust
use futures_executor::block_on;
use json_count_core::{ColumnResult, scan_source_with_progress};
use stream_io::SliceSource;

let doc = br#"{"a":1,"b":[null,"ok"]}"#;
let cols = block_on(scan_source_with_progress(
    SliceSource::new(doc),
    |_, _| {},
));
assert_eq!(
    cols,
    Ok(vec![
        ColumnResult { key: "a".to_string(), count: 1 },
        ColumnResult { key: "b".to_string(), count: 1 },
    ])
);
```

## License

`AGPL-3.0-only` --- see
[`LICENSE`](https://github.com/Adafede/dioxus/blob/main/LICENSE) for details.

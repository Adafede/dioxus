# stream-io

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![CI](https://github.com/Adafede/dioxus/actions/workflows/ci.yml/badge.svg)](https://github.com/Adafede/dioxus/actions/workflows/ci.yml)

Portable async chunked byte and line streaming primitives.

Provides bounded-memory streaming readers over [`ChunkSource`] without coupling
to browser or UI frameworks:

- [`BlobCursor`] --- byte-level chunked lookahead, slice access, and cursor
  advancement.
- [`BlobLines`] --- line-oriented `\n`/`\r\n` streaming reader with line length
  caps.
- [`SliceSource`] --- in-memory [`ChunkSource`] for host execution and tests.

## Example

```rust
use futures_executor::block_on;
use stream_io::{BlobLines, SliceSource};

let lines = block_on(async {
    let mut reader = BlobLines::new(SliceSource::new(b"alpha\nbeta\n"), |_, _| {});
    let mut out = Vec::new();
    while let Some(line) = reader.next_line().await? {
        out.push(line);
    }
    Ok::<_, stream_io::StreamError>(out)
});
assert_eq!(lines, Ok(vec!["alpha".to_string(), "beta".to_string()]));
```

## License

`AGPL-3.0-only` --- see
[`LICENSE`](https://github.com/Adafede/dioxus/blob/main/LICENSE) for details.

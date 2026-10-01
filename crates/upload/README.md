# upload

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![Tests](https://img.shields.io/badge/tests-14-brightgreen)](https://github.com/adafede/dioxus/actions)

> **Note:** Most of the upload crate's API is WASM-only (`BlobCursor`,
> `BlobLines`, browser `download_*` helpers). Native-test coverage is limited to
> `ProgressThrottler`, `sanitize_filename`, `UploadError`, and the non-WASM
> download stubs.

## upload --- streaming file I/O, progress, and download for WASM apps

The single crate that every upload-based WASM app in this workspace consumes. It
provides:

- `BlobCursor` --- byte-level chunked streaming over a browser `Blob`, keeping
  exactly one 16 MiB chunk in memory regardless of file size.
- `BlobLines` --- line-oriented chunked streaming for text formats (MGF, SMILES,
  CSV).
- Throttled progress callbacks on both readers --- byte- and time-gated so a
  multi-gigabyte upload does not flood the UI thread. The throttling type is
  crate-private; apps pass a closure to `BlobCursor::new` or `BlobLines::new`.
- **`extract_blob_from_file_data`** --- unified file-input / drag-drop
  extraction over `&[FileData]`.
- **`download_text`** --- browser download of text content.

### Design non-goals

- Native file I/O (WASM-only by design)
- HTTP upload to servers
- SPARQL querying or LOTUS domain modeling → see the `lotus-search` crate

## License

`AGPL-3.0-only` --- see [`LICENSE`](https://www.gnu.org/licenses/agpl-3.0.html)
for details.

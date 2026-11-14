# upload

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![CI](https://github.com/Adafede/dioxus/actions/workflows/ci.yml/badge.svg)](https://github.com/Adafede/dioxus/actions/workflows/ci.yml)

> **Note:** `BlobCursor`, `BlobLines`, `ChunkSource`, `SliceSource`,
> `ProgressThrottler`, `UploadError`, and `sanitize_filename` compile and run on
> native targets as well as `wasm32`. Only `BlobSource`, DOM file-event
> extraction (`extract_blob_from_file_data` on `wasm32`), and browser
> `download_*` triggers target `wasm32` (with non-WASM no-op stubs for native
> builds).

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
- **`download_text` / `download_svg` / `download_with_mime`** --- browser
  download helpers.

### Design non-goals

- Native file I/O (WASM-only by design)
- HTTP upload to servers
- SPARQL querying or LOTUS domain modeling → see the `lotus-search` crate

## License

`AGPL-3.0-only` --- see [`LICENSE`](https://www.gnu.org/licenses/agpl-3.0.html)
for details.

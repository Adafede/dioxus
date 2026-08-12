# upload

[![AGPL-3.0 license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![Coverage](https://img.shields.io/badge/coverage-native--tested-blue)]()

> **Note:** Most of the upload crate's API is WASM-only (`BlobCursor`,
> `BlobLines`, browser `download_*` helpers).  Native-test coverage is limited
> to `ProgressThrottler`, `sanitize_filename`, `UploadError`, and the
> non-WASM download stubs.

## upload — streaming file I/O, progress, and download for WASM apps

The single crate that every upload-based WASM app in this workspace
consumes.  It provides:

- `BlobCursor` — byte-level chunked streaming over a browser `Blob`,
  keeping exactly one 16 MiB chunk in memory regardless of file size.
- `BlobLines` — line-oriented chunked streaming for text formats (MGF,
  SMILES, CSV).
- **`ProgressThrottler`** — byte+time throttled progress callbacks, shared by
  all upload apps.
- **`extract_blob_from_file_data`** — unified file-input / drag-drop
  extraction over `&[FileData]`, eliminating the identical inline
  boilerplate every app previously copied.

- **Download helpers** — `download_text`, `download_text_as_blob`,
  `download_url`, and `submit_download_form`, consolidating the per-app
  download code from `json-count-rs`, `mgf-precursor-erro-rs`,
  `lipid-selecto-rs`, and `lotus-explore-rs`.

### Design non-goals

- Native file I/O (WASM-only by design)
- HTTP upload to servers
- SPARQL querying or LOTUS domain modeling → see the `lotus` crate

## License

`AGPL-3.0-only` — see [`LICENSE`](https://www.gnu.org/licenses/agpl-3.0.html) for details.

# {{crate}}

[![AGPL-3.0 license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![CI](https://github.com/Adafede/dioxus/actions/workflows/ci.yml/badge.svg)](https://github.com/Adafede/dioxus/actions/workflows/ci.yml)

> **Note:** `BlobCursor`, `BlobLines`, `ChunkSource`, `SliceSource`,
> `ProgressThrottler`, `UploadError`, and `sanitize_filename` compile and run on
> native targets as well as `wasm32`. Only `BlobSource`, DOM file-event
> extraction (`extract_blob_from_file_data` on `wasm32`), and browser
> `download_*` triggers target `wasm32` (with non-WASM no-op stubs for native
> builds).

{{readme}}

## License

`AGPL-3.0-only` — see [`LICENSE`](https://www.gnu.org/licenses/agpl-3.0.html) for details.

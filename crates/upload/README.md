# upload

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![CI](https://github.com/Adafede/dioxus/actions/workflows/ci.yml/badge.svg)](https://github.com/Adafede/dioxus/actions/workflows/ci.yml)

> **Note:** Portable streaming primitives (`BlobCursor`, `BlobLines`,
> `ChunkSource`, `SliceSource`, `StreamError`) live in `stream-io` and are
> re-exported here alongside browser `BlobSource`, Dioxus `FileData` extraction,
> and DOM download helpers.

## upload --- browser file I/O, extraction, and download adapters

Browser-facing adapter over [`stream_io`] providing:

- `BlobSource` (`wasm32`) --- [`ChunkSource`] backed by a browser `Blob` with a
  monotonic `performance.now()` clock.
- [`extract_blob_from_file_data`] --- unified file-input / drag-drop extraction
  over `&[FileData]`.
- [`download_text`] / [`download_svg`] / [`download_with_mime`] --- browser
  download helpers returning [`UploadError`].
- Re-exports of [`BlobCursor`], [`BlobLines`], [`ChunkSource`], [`SliceSource`],
  and [`StreamError`] from [`stream_io`].

## License

`AGPL-3.0-only` --- see
[`LICENSE`](https://github.com/Adafede/dioxus/blob/main/LICENSE) for details.

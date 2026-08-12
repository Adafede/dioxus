# Changelog

## Unreleased

- **Document head in Rust**: All static `index.html` files replaced with
  `ui::document::DocumentHead` (meta tags, scripts, styles, JSON-LD) and
  `ui::document::DocumentLinks` (link tags, preconnect, favicons). The `index`,
  `lotus-explore-rs`, and `smellfish-rs` apps now manage their `<head>` from
  Rust via `dioxus::document`, eliminating 6 static `index.html` files.
  Smellfish CSS moved from body `<style>` to document head.
- **Crate consolidation**: Split `crates/shared` into `crates/lotus` (SPARQL,
  models, queries) and `crates/upload` (streaming file I/O, progress, download).
  Removed the old `crates/file-upload` and `crates/shared` crates entirely.
- **One canonical export format**: `lotus::export::ExportFormat` replaces the
  duplicated `ExportArchiveFormat` (lotus-api), `DownloadFormat`
  (lotus-explore-rs), and ad-hoc export URL builders. All three apps now share
  `lotus::export` for `qlever_export_url`, `api_export_file_url`,
  `build_upstream_export_url`, `sanitize_download_filename`, and
  `prepared_query`.
- **Unified file-upload handlers**: Every WASM app (json-count-rs,
  mgf-precursor-erro-rs, lipid-selecto-rs, lotus-explore-rs) now uses
  `upload::extract_blob_from_file_data` instead of inline FormData/DragData
  extraction code.
- **Streaming via `BlobLines`**: All text-based apps (MGF, SMILES/CSV) now
  stream through `upload::BlobLines` with 16 MiB chunks instead of calling
  `blob.text()` which loaded the entire file into memory. No per-byte `await`.
- **Consolidated download helpers**: `upload::download_text`,
  `download_text_as_blob`, `download_url`, and `submit_download_form` replace
  the per-app download implementations. Removed all `unwrap`/`expect`/`panic!`
  from download paths.
- **Progress throttling**: `ProgressThrottler` with
  `PROGRESS_BYTE_INTERVAL = 4 MiB` and `PROGRESS_TIME_INTERVAL_MS = 120.0`
  shared by all streaming apps.
- Added AI agent onboarding docs and machine-readable project metadata.
- Hardened `lotus-api` with secure response headers and a runtime metrics
  endpoint.
- Simplified the `lotus-explorer` skillbook into plain skill modules with a
  separate suggestions file.
- **README verification in prek**: Combined `cargo-readme-check` and
  `panache lint` into a single `cargo-readme-panache` pre-push hook that
  generates all 10 `README.md` files from `//!` doc comments, lints them with
  `panache`, and verifies they are in sync with the committed versions. Fixed a
  `swallowed-list-marker` lint in the `index` crate's doc comments.

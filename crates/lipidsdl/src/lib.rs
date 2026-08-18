//! `lipidsdl` — generic download, SDF parsing, and lipid-data conversion utilities.
//!
//! This crate is intentionally **not** specific to any one application or dataset.
//! It provides:
//!
//! - [`download`] — async file download from a URL (native only).
//! - [`sdf`] — a streaming SDF (Structure-Data File) parser.
//! - [`sdf::lipidmaps`] — column definitions and TSV conversion helpers for
//!   the `LipidMaps` LMSD dataset.
//!
//! Both the WASM apps and native binaries in this workspace can depend on
//! `lipidsdl` for the pure-Rust parsing logic; only the `download` submodule
//! requires native (`tokio` + `reqwest`).

pub mod download;
pub mod sdf;

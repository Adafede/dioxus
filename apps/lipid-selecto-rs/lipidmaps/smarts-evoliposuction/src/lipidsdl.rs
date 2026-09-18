//! Inlined from `crates/lipidsdl` — generic download, SDF parsing, and lipid-data conversion utilities.

#[cfg(not(target_arch = "wasm32"))]
pub mod download;
pub mod sdf;

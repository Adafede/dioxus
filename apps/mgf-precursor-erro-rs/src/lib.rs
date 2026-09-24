// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! `mgf-precursor-erro-rs` — MGF precursor mass-error analysis.
//!
//! Uploads an MGF file, recalibrates precursor *m/z* values, and visualises
//! the resulting mass-error distribution as an interactive histogram.
//!
//! # Run locally
//!
//! ```bash
//! dx serve --package mgf-precursor-erro-rs
//! ```
//!
//! # Build for the website
//!
//! ```bash
//! dx build --release --platform web --package mgf-precursor-erro-rs
//! ```

pub(crate) mod app;
/// Externally consumed by `examples/recalibration_demo.rs`.
pub mod diagnostics;
pub(crate) mod errors;
pub mod metrics;
#[cfg(target_arch = "wasm32")]
pub(crate) mod parser;
/// Externally consumed by `examples/recalibration_demo.rs`.
pub mod plotting;
/// Externally consumed by `examples/recalibration_demo.rs`.
pub mod recalibration;

pub use app::app;
/// Error type surfaced in public signatures (e.g. plotting renders).
pub use errors::MgfError;

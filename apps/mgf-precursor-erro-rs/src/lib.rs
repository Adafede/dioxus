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

pub mod app;
pub mod diagnostics;
pub mod errors;
pub mod metrics;
pub mod parser;
pub mod plotting;
pub mod recalibration;

pub use app::app;
pub use errors::MgfError;

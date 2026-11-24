// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! `cxsmiles-yoga` — generate CX-SMILES from lists of related structures.
//!
//! Core CX-SMILES clustering, MCS extraction, diffing, and round-trip scoring
//! live in `cxsmiles-core`. This crate provides the Dioxus WebAssembly interface
//! (`app`) and curated fixture sets (`examples`).
//!
//! ## Usage
//!
//! ### Locally
//!
//! ```bash
//! dx serve --package cxsmiles-yoga
//! ```
//!
//! ### Build for deployment
//!
//! ```bash
//! dx build --profile wasm-release --platform web --package cxsmiles-yoga
//! ```
//!
//! ### Tests
//!
//! ```bash
//! cargo nextest run -p cxsmiles-core -p cxsmiles-yoga
//! ```

pub mod app;
pub mod examples;

/// Root application component. The only externally consumed item.
pub use app::app;

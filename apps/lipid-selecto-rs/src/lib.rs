// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! `lipid-selecto-rs` — browser-based lipid classification and filtering.
//!
//! Thin Dioxus WebAssembly UI over `lipid-core` (MGF/SMILES extraction,
//! compiled LIPID MAPS SMARTS classification, gallery projection, and filtered
//! exports).
//!
//! # Usage
//!
//! ## Locally
//!
//! ```bash
//! dx serve --package lipid-selecto-rs
//! ```
//!
//! ## Build for deployment
//!
//! ```bash
//! dx build --profile wasm-release --platform web --package lipid-selecto-rs
//! ```
//!
//! ## Tests
//!
//! ```bash
//! cargo nextest run -p lipid-core -p lipid-selecto-rs --locked
//! ```

pub(crate) mod app;

/// Root application component. The only externally consumed item.
pub use app::app;

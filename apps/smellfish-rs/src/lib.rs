// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! `smellfish-rs` — literature-backed NP-likeness scoring.
//!
//! Scores natural-product-likeness of SMILES structures using machine-learned
//! features, `Query` enrichment, and RDKit.js chemistry descriptors.
//!
//! # Run locally
//!
//! ```bash
//! dx serve --package smellfish-rs
//! ```
//!
//! # Build for deployment
//!
//! ```bash
//! dx build --release --platform web --package smellfish-rs
//! ```
//!
//! # Tests
//!
//! ```bash
//! cargo test --lib -p smellfish-rs
//! ```

#![cfg_attr(target_arch = "wasm32", allow(clippy::future_not_send))]

pub(crate) mod app;
#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) mod csv;
pub(crate) mod document_head;
#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) mod evidence;
pub(crate) mod literature;
pub(crate) mod model;
#[cfg(target_arch = "wasm32")]
pub(crate) mod pipeline;
#[cfg(target_arch = "wasm32")]
pub(crate) mod qlever;
#[cfg(target_arch = "wasm32")]
pub(crate) mod rdkit;
pub(crate) mod rdkit_bridge;
pub(crate) mod styles;

/// Root application component. The only externally consumed item.
pub use app::app;

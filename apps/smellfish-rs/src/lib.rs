// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! `smellfish-rs` — literature-backed NP-likeness scoring.
//!
//! Scores natural-product-likeness of SMILES structures using Ertl
//! fragment-contribution NP-likeness scores, QLever/LOTUS SPARQL enrichment,
//! and RDKit.js chemistry descriptors.
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
//! dx build --profile wasm-release --platform web --package smellfish-rs
//! ```
//!
//! # Tests
//!
//! ```bash
//! cargo nextest run -p smellfish-core -p smellfish-rs
//! ```

pub(crate) mod app;
pub(crate) mod document_head;
pub(crate) mod literature;
#[cfg(target_arch = "wasm32")]
pub(crate) mod pipeline;
#[cfg(target_arch = "wasm32")]
pub(crate) mod rdkit;
pub(crate) mod rdkit_bridge;
pub(crate) mod styles;

/// Root application component. The only externally consumed item.
pub use app::app;

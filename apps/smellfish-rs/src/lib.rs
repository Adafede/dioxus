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
//!
//! # Lint policy
//!
//! WASM UI code legitimately triggers some pedantic lints (float casts in
//! descriptor math, `format!` interpolation, etc.).  Rather than a blanket
//! `#![allow(...)]`, each suppression is a targeted, justified `#[allow]` at
//! the affected call site.

#![cfg_attr(target_arch = "wasm32", allow(clippy::future_not_send))]

pub mod app;
#[cfg(any(test, target_arch = "wasm32"))]
pub mod csv;
pub mod document_head;
#[cfg(any(test, target_arch = "wasm32"))]
pub mod evidence;
pub mod literature;
pub mod model;
#[cfg(target_arch = "wasm32")]
pub mod pipeline;
#[cfg(target_arch = "wasm32")]
pub mod qlever;
#[cfg(target_arch = "wasm32")]
pub mod rdkit;
pub mod styles;

pub use app::app;

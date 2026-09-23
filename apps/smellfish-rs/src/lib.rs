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
// Note: Many items in submodules are `pub` but only accessible within this crate
// (the modules themselves are not re-exported publicly). `unreachable_pub` would
// flag these as unreachable from external crates, but they form the internal API
// that other modules in this crate depend on.
#![allow(unreachable_pub)]

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
pub mod rdkit_bridge;
pub mod styles;

pub use app::app;

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

fn main() {
    dioxus::launch(smellfish_rs::app);
}

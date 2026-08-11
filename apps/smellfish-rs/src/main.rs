//! `smellfish-rs` — literature-backed NP-likeness scoring.
//!
//! Scores natural-product-likeness of SMILES structures using machine-learned
//! features, `Query` enrichment, and RDKit.js chemistry descriptors.

mod app;
#[cfg(any(test, target_arch = "wasm32"))]
mod csv;
#[cfg(any(test, target_arch = "wasm32"))]
mod evidence;
mod literature;
mod model;
#[cfg(target_arch = "wasm32")]
mod pipeline;
#[cfg(target_arch = "wasm32")]
mod qlever;
#[cfg(target_arch = "wasm32")]
mod rdkit;
mod styles;

use dioxus::prelude::launch;

fn main() {
    launch(app::app);
}

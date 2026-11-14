// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! `lipid-selecto-rs` — browser-based lipid classification and filtering.
//!
//! Parses mass-spectrometry files (MGF or SMILES), classifies molecules using
//! compiled-in LIPID MAPS-aligned SMARTS patterns, and exports the selected
//! lipid subset in the original input format. All parsing and matching run
//! client-side in WebAssembly.
//!
//! # Features
//!
//! - Built-in SMARTS patterns covering all 8 LIPID MAPS families (FA, GL, GP, SP,
//!   ST, PR, SL, PK) and all 86 LMSD main classes
//! - Auto-detects input format (MGF or SMILES list) and preserves it on export
//! - Real-time class selection with live gallery and count updates
//! - Local 2D molecular structure rendering (no external rendering service)
//! - Export filtered results (MGF or SMILES) with original metadata preserved
//! - 204 curated example lipids across all 8 LIPID MAPS families
//!
//! # Usage
//!
//! ## Online
//!
//! Published with the workspace suite on GitHub Pages under `/lipid-selecto-rs`.
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
//! cargo nextest run -p lipid-selecto-rs --locked
//! ```
//!
//! # Classification strategy
//!
//! Rules are compiled into `src/chemical_class/lmsd.rs`. See
//! [`LIPID_ARCHITECTURE.md`](./LIPID_ARCHITECTURE.md) and
//! [`RULES_GUIDE.md`](./RULES_GUIDE.md) for the rule families, SMARTS syntax,
//! and regeneration workflow via `lipidmaps/smarts-evoliposuction`.
//!
//! # Dependencies
//!
//! - [`chematic`](https://crates.io/crates/chematic) — Pure-Rust SMILES/SMARTS
//! - [`dioxus`](https://dioxuslabs.com) — UI framework (compiles to WebAssembly)
//! - [`web-sys`](https://crates.io/crates/web-sys) — Browser API bindings
//!
//! Processing runs entirely inside the browser without uploading files to a
//! remote server.

pub(crate) mod app;
pub(crate) mod chemical_class;
pub(crate) mod depict_simple;
pub(crate) mod examples;
pub(crate) mod format;
pub(crate) mod lipids;
pub(crate) mod parser;

/// Root application component. The only externally consumed item.
pub use app::app;

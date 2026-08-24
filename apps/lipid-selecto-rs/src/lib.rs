//! `lipid-selecto-rs` — interactive lipid classification and filtering.
//!
//! **A modern, interactive web application for filtering and visualizing lipid
//! mass spectrometry data.**
//!
//! Analyzes mass spectrometry data (MGF or SMILES), classifies molecules using
//! LIPID MAPS-aligned SMARTS patterns, and lets you download the lipid-only
//! subset in the same format. Built with pure Rust using WebAssembly for
//! blazing-fast performance in the browser.
//!
//! # Features
//!
//! - 30+ built-in SMARTS rules covering all 8 LIPID MAPS families (FA, GL, GP, SP, ST, PR, SL, PK)
//! - Supports evolved SMARTS rules from smarts-evoliposuction
//! - Auto-detects input format: MGF or SMILES list; preserves output format
//! - Real-time class selection with live gallery/count updates
//! - 2D molecular structure rendering (no external rendering service)
//! - Download filtered output (MGF or SMILES) with all metadata preserved
//! - 100 curated example lipids from LIPID MAPS
//!
//! # Usage
//!
//! ## Online
//!
//! Visit [lipid-selecto-rs.princelab.org](https://lipid-selecto-rs.princelab.org)
//! to use the web app.
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
//! dx build --release --platform web --package lipid-selecto-rs
//! ```
//!
//! ## Tests
//!
//! ```bash
//! cargo test --lib -p lipid-selecto-rs
//! ```
//!
//! # Classification strategy
//!
//! Rules are **fully configurable** in YAML.  See
//! [`RULES_GUIDE.md`](./RULES_GUIDE.md) for adding custom lipid classes,
//! SMARTS pattern syntax, and rule priority configuration.
//!
//! # Dependencies
//!
//! - [`chematic`](https://crates.io/crates/chematic) — Pure-Rust SMILES/SMARTS
//! - [`dioxus`](https://dioxuslabs.com) — UI framework (compiles to WebAssembly)
//! - [`web-sys`](https://crates.io/crates/web-sys) — Browser API bindings
//!
//! No external services or native binaries required.  All processing happens
//! in the browser — your data never leaves your computer.
pub mod app;
pub mod chain_analysis;
pub mod chemical_class;
pub mod depict_simple;
pub mod examples;
pub mod format;
pub mod lipids;
pub mod parser;
pub mod rules;

pub use app::app;

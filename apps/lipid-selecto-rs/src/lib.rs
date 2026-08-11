//! `lipid-selecto-rs` — select lipid spectra by chemical class with interactive filtering.
//!
//! Supports MGF and SMILES input formats, auto-detects and preserves format for output.
//! Uses extensible SMARTS rules aligned with LIPID MAPS classification.
pub mod app;
pub mod chain_analysis;
pub mod chemical_class;
pub mod depict_simple;
pub mod examples;
pub mod format;
pub mod lipid_smarts;
pub mod lipids;
pub mod parser;
pub mod rules;

pub use app::app;

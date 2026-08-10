//! `lipid-selecto-rs` — select lipid spectra from an MGF file by SMILES.
pub mod app;
pub mod chain_analysis;
pub mod chemical_class;
pub mod depict_simple;
pub mod lipids;
pub mod lipid_smarts;
pub mod parser;

pub use app::app;

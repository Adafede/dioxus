// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! `lipid-selecto-rs` — browser-based lipid classification and filtering.
//!
//! Thin Dioxus WebAssembly UI over `lipid-core` (MGF/SMILES extraction,
//! compiled LIPID MAPS SMARTS classification, gallery projection, and filtered
//! exports).
//!
//! # Usage
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
//! cargo nextest run -p lipid-core -p lipid-selecto-rs --locked
//! ```

pub(crate) mod app;

/// Root application component. The only externally consumed item.
pub use app::app;

#[cfg(test)]
mod tests {
    use lipid_core::{
        BlockExtractor, build_analysis_from_classified, classify_blocks, example_smiles,
        lmsd_classes,
    };

    #[test]
    fn bundled_example_asset_matches_curated_dataset() {
        let asset_text = include_str!("../assets/example_lipids.smi");
        let mut extractor = BlockExtractor::default();
        for line in asset_text.lines() {
            extractor.push_line(line);
        }
        let mut blocks = extractor.finish();
        let classes = lmsd_classes();
        classify_blocks(&mut blocks, classes);
        let analysis = build_analysis_from_classified(blocks, 16, classes);
        assert_eq!(analysis.summary.total_items, example_smiles().len());
        assert_eq!(analysis.summary.total_items, 204);
        assert_eq!(analysis.summary.lipid_items, 195);
        assert_eq!(analysis.summary.unclassified, 9);
    }
}

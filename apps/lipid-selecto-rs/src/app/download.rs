// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Browser file downloads (filtered MGF and filtered SMILES).
//!
//! WASM-only: uses the `upload` crate's `download_text` helper to trigger
//! a browser download with the given filename.

use super::class_tags::insert_class_comment;
use super::gallery::{block_class_tags, build_smiles_from_blocks, passes_item_filters};
use crate::chemical_class::ChemicalClass;
use crate::parser::{GalleryItem, SpectrumBlock};

#[allow(clippy::too_many_arguments)]
fn filter_matching_blocks<'a>(
    blocks: &'a [SpectrumBlock],
    mz_min_val: f64,
    mz_max_val: f64,
    prec_min_val: f64,
    prec_max_val: f64,
    adduct_val: &str,
    selected: &[String],
) -> Vec<&'a SpectrumBlock> {
    blocks
        .iter()
        .filter(|block| {
            passes_item_filters(
                block.gallery_item_matches.as_ref(),
                block.exact_mass,
                block.precursor_mz,
                block.adduct.as_deref(),
                selected,
                mz_min_val,
                mz_max_val,
                prec_min_val,
                prec_max_val,
                adduct_val,
            )
        })
        .collect()
}

/// Build filtered MGF content with class tags across all parsed blocks.
#[allow(clippy::too_many_arguments)]
pub(super) fn build_filtered_mgf_content(
    blocks: &[SpectrumBlock],
    gallery: &[GalleryItem],
    all_classes: &[ChemicalClass],
    mz_min_val: f64,
    mz_max_val: f64,
    prec_min_val: f64,
    prec_max_val: f64,
    adduct_val: &str,
    selected: &[String],
) -> String {
    let mut mgf_content = String::new();
    for block in filter_matching_blocks(
        blocks,
        mz_min_val,
        mz_max_val,
        prec_min_val,
        prec_max_val,
        adduct_val,
        selected,
    ) {
        let class_tags = block_class_tags(block, gallery, all_classes);
        let tagged = insert_class_comment(&block.raw, &class_tags);
        mgf_content.push_str(&tagged);
        mgf_content.push('\n');
    }
    mgf_content
}

/// Build filtered SMILES content with class tags across all parsed blocks.
#[allow(clippy::too_many_arguments)]
pub(super) fn build_filtered_smiles_content(
    blocks: &[SpectrumBlock],
    gallery: &[GalleryItem],
    all_classes: &[ChemicalClass],
    mz_min_val: f64,
    mz_max_val: f64,
    prec_min_val: f64,
    prec_max_val: f64,
    adduct_val: &str,
    selected: &[String],
) -> String {
    let filtered = filter_matching_blocks(
        blocks,
        mz_min_val,
        mz_max_val,
        prec_min_val,
        prec_max_val,
        adduct_val,
        selected,
    );
    build_smiles_from_blocks(&filtered, gallery, all_classes)
}

/// Download filtered MGF content with class tags, applying all filter selections.
#[allow(clippy::too_many_arguments)] // Dioxus Signal callbacks inherently require many params
pub(super) fn download_filtered_mgf(
    blocks: &[SpectrumBlock],
    gallery: &[GalleryItem],
    all_classes: &[ChemicalClass],
    mz_min_val: f64,
    mz_max_val: f64,
    prec_min_val: f64,
    prec_max_val: f64,
    adduct_val: &str,
    selected: &[String],
) {
    let mgf_content = build_filtered_mgf_content(
        blocks,
        gallery,
        all_classes,
        mz_min_val,
        mz_max_val,
        prec_min_val,
        prec_max_val,
        adduct_val,
        selected,
    );
    #[cfg(target_arch = "wasm32")]
    drop(upload::download_text(&mgf_content, "lipids_filtered.mgf"));
    #[cfg(not(target_arch = "wasm32"))]
    drop(mgf_content);
}

/// Download filtered SMILES content with class tags, applying all filter selections.
#[allow(clippy::too_many_arguments)] // Dioxus Signal callbacks inherently require many params
pub(super) fn download_filtered_smiles(
    blocks: &[SpectrumBlock],
    gallery: &[GalleryItem],
    all_classes: &[ChemicalClass],
    mz_min_val: f64,
    mz_max_val: f64,
    prec_min_val: f64,
    prec_max_val: f64,
    adduct_val: &str,
    selected: &[String],
) {
    let smiles_content = build_filtered_smiles_content(
        blocks,
        gallery,
        all_classes,
        mz_min_val,
        mz_max_val,
        prec_min_val,
        prec_max_val,
        adduct_val,
        selected,
    );
    #[cfg(target_arch = "wasm32")]
    drop(upload::download_text(&smiles_content, "lipids.smi"));
    #[cfg(not(target_arch = "wasm32"))]
    drop(smiles_content);
}

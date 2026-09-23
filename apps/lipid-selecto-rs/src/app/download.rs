// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Browser file downloads (filtered MGF and filtered SMILES).
//!
//! WASM-only: uses the `upload` crate's `download_text` helper to trigger
//! a browser download with the given filename.

#[cfg(target_arch = "wasm32")]
use super::class_tags::insert_class_comment;
#[cfg(target_arch = "wasm32")]
use super::gallery::build_smiles_from_gallery;
#[cfg(target_arch = "wasm32")]
use super::types::GallerySmilesEntry;
#[cfg(target_arch = "wasm32")]
use crate::parser::SpectrumBlock;

/// Download filtered MGF content with class tags, applying all filter selections.
#[cfg(target_arch = "wasm32")]
#[allow(clippy::too_many_arguments)]
pub fn download_filtered_mgf(
    blocks: &[SpectrumBlock],
    block_class_tags: &[Vec<(String, String)>],
    mz_min_val: f64,
    mz_max_val: f64,
    prec_min_val: f64,
    prec_max_val: f64,
    adduct_val: &str,
    selected: &[String],
) {
    let mut mgf_content = String::new();
    for (idx, block) in blocks.iter().enumerate() {
        if !selected.is_empty()
            && !passes_class_filter(block.gallery_item_matches.as_ref(), selected)
        {
            continue;
        }
        if block.exact_mass < mz_min_val || block.exact_mass > mz_max_val {
            continue;
        }
        if let Some(pmz) = block.precursor_mz
            && (pmz < prec_min_val || pmz > prec_max_val)
        {
            continue;
        }
        if !adduct_val.is_empty() && block.adduct.as_deref() != Some(adduct_val) {
            continue;
        }
        let class_tags = &block_class_tags[idx];
        let tagged = insert_class_comment(&block.raw, class_tags);
        mgf_content.push_str(&tagged);
        mgf_content.push('\n');
    }
    let _ = upload::download_text(&mgf_content, "lipids_filtered.mgf");
}

/// Download filtered SMILES content with class tags, applying all filter selections.
#[cfg(target_arch = "wasm32")]
pub fn download_filtered_smiles(
    gallery_smiles: &[GallerySmilesEntry],
    mz_min_val: f64,
    mz_max_val: f64,
    prec_min_val: f64,
    prec_max_val: f64,
    adduct_val: &str,
    selected: &[String],
) {
    let filtered: Vec<_> = gallery_smiles
        .iter()
        .filter(
            |(_, _, _, _, _, exact_mass, precursor_mz, adduct, class_matches)| {
                if !selected.is_empty()
                    && !selected
                        .iter()
                        .any(|class_name| class_matches.get(class_name).copied().unwrap_or(false))
                {
                    return false;
                }
                if let Some(mass) = exact_mass
                    && (*mass < mz_min_val || *mass > mz_max_val)
                {
                    return false;
                }
                if let (Some(pmz), val) = (precursor_mz, prec_min_val)
                    && (*pmz < val || *pmz > prec_max_val)
                {
                    return false;
                }
                if !adduct_val.is_empty() && adduct.as_deref() != Some(adduct_val) {
                    return false;
                }
                true
            },
        )
        .collect();
    let smiles_content = build_smiles_from_gallery(&filtered);
    let _ = upload::download_text(&smiles_content, "lipids.smi");
}

/// Check if any of the selected class names match the gallery item matches.
#[cfg(target_arch = "wasm32")]
fn passes_class_filter(
    gallery_item_matches: Option<&std::collections::HashMap<String, bool>>,
    selected: &[String],
) -> bool {
    selected.iter().any(|class_name| {
        gallery_item_matches
            .and_then(|m| m.get(class_name))
            .copied()
            .unwrap_or(false)
    })
}

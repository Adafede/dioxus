// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! LIPID MAPS metadata tag derivation and filtered MGF / SMILES export builders.

use std::fmt::Write as _;

use crate::chemical_class::ChemicalClass;
use crate::family::LipidFamily;
use crate::filter::{FilterCriteria, passes_item_filters};
use crate::parser::analysis::GalleryItem;
use crate::parser::block::SpectrumBlock;

/// Structured LIPID MAPS `COMMENT=LIPID_MAPS_*` tags for a spectrum.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ClassTags {
    /// `CATEGORY` field (e.g. `"Fatty Acyls [FA]"`).
    pub(crate) category: String,
    /// `MAIN_CLASS` field (e.g. `"Fatty Acids and Conjugates [FA01]"`).
    pub(crate) main_class: String,
    /// `SUB_CLASS` field (currently always `"-"`).
    pub(crate) sub_class: &'static str,
}

impl ClassTags {
    /// Render `(key, value)` pairs for MGF comment headers.
    #[must_use]
    pub(crate) fn as_pairs(&self) -> [(&'static str, &str); 3] {
        [
            ("CATEGORY", &self.category),
            ("MAIN_CLASS", &self.main_class),
            ("SUB_CLASS", self.sub_class),
        ]
    }
}

/// Compute `CATEGORY` / `MAIN_CLASS` / `SUB_CLASS` tags from a [`GalleryItem`].
#[must_use]
pub(crate) fn class_tags_from_gallery(item: &GalleryItem, classes: &[ChemicalClass]) -> ClassTags {
    let matched: Vec<&ChemicalClass> = classes
        .iter()
        .filter(|c| item.class_matches.contains(c.index))
        .collect();
    if matched.is_empty() {
        let category = item.classification.as_ref().map_or_else(
            || "Other Lipids [-]".to_string(),
            |c| c.family.category().to_string(),
        );
        let main_class = item
            .primary_class_name
            .clone()
            .unwrap_or_else(|| "-".to_string());
        return ClassTags {
            category,
            main_class,
            sub_class: "-",
        };
    }
    tags_from_matched(&matched, item.classification.as_ref().map(|c| c.family))
}

fn tags_from_matched(matched: &[&ChemicalClass], broad: Option<LipidFamily>) -> ClassTags {
    let selected: Vec<&ChemicalClass> = broad.map_or_else(
        || matched.to_vec(),
        |fam| {
            let same: Vec<&ChemicalClass> = matched
                .iter()
                .filter(|c| c.family == fam)
                .copied()
                .collect();
            if same.is_empty() {
                matched.to_vec()
            } else {
                same
            }
        },
    );

    let mut categories: Vec<&'static str> = Vec::new();
    for c in &selected {
        let label = c.family.category();
        if !categories.contains(&label) {
            categories.push(label);
        }
    }
    let main_class = selected
        .iter()
        .map(|c| c.name.as_str())
        .collect::<Vec<_>>()
        .join(" | ");
    ClassTags {
        category: categories.join(" | "),
        main_class,
        sub_class: "-",
    }
}

/// Compute fallback tags directly from a [`SpectrumBlock`].
#[must_use]
pub(crate) fn class_tags_from_block(block: &SpectrumBlock, classes: &[ChemicalClass]) -> ClassTags {
    let category = block.classification.as_ref().map_or_else(
        || "Other Lipids [-]".to_string(),
        |c| c.family.category().to_string(),
    );
    let main_class = block
        .gallery_item_matches
        .and_then(|mask| {
            classes
                .iter()
                .find(|c| mask.contains(c.index))
                .map(|c| c.name.clone())
        })
        .unwrap_or_else(|| "-".to_string());
    ClassTags {
        category,
        main_class,
        sub_class: "-",
    }
}

/// Look up matching gallery item tags if in gallery cap, otherwise derive from block.
#[must_use]
pub(crate) fn block_class_tags(
    block: &SpectrumBlock,
    gallery: &[GalleryItem],
    classes: &[ChemicalClass],
) -> ClassTags {
    let gallery_item = gallery
        .binary_search_by_key(&block.index, |item| item.block_index)
        .ok()
        .and_then(|idx| gallery.get(idx));
    gallery_item.map_or_else(
        || class_tags_from_block(block, classes),
        |item| class_tags_from_gallery(item, classes),
    )
}

/// Insert `COMMENT=LIPID_MAPS_*` lines right after `BEGIN IONS`.
#[must_use]
pub(crate) fn insert_class_comment(block_raw: &str, tags: &ClassTags) -> String {
    let mut out = String::with_capacity(block_raw.len() + 128);
    let split_pos = block_raw
        .find(|c: char| !c.is_whitespace())
        .filter(|&begin_pos| {
            block_raw
                .get(begin_pos..)
                .and_then(|rest| rest.lines().next())
                .is_some_and(|line| mgf::is_ions_marker(line, "BEGIN IONS"))
        })
        .and_then(|begin_pos| {
            block_raw
                .get(begin_pos..)
                .and_then(|rest| rest.find('\n'))
                .map(|newline_pos| begin_pos + newline_pos + 1)
        })
        .unwrap_or(0);
    let (before, after) = block_raw.split_at(split_pos);
    out.push_str(before);
    for (k, v) in tags.as_pairs() {
        let _ = writeln!(out, "COMMENT=LIPID_MAPS_{k}={v}");
    }
    out.push_str(after);
    out
}

fn filter_matching_blocks<'a>(
    blocks: &'a [SpectrumBlock],
    criteria: &FilterCriteria<'_>,
) -> Vec<&'a SpectrumBlock> {
    blocks
        .iter()
        .filter(|block| {
            passes_item_filters(
                block.class_matches(),
                block.exact_mass,
                block.precursor_mz,
                block.adduct.as_deref(),
                criteria,
            )
        })
        .collect()
}

/// Build filtered MGF content with `COMMENT=LIPID_MAPS_*` tags.
#[must_use]
pub fn build_filtered_mgf_content(
    blocks: &[SpectrumBlock],
    gallery: &[GalleryItem],
    classes: &[ChemicalClass],
    criteria: &FilterCriteria<'_>,
) -> String {
    let mut out = String::new();
    for block in filter_matching_blocks(blocks, criteria) {
        let tags = block_class_tags(block, gallery, classes);
        out.push_str(&insert_class_comment(&block.raw, &tags));
        out.push('\n');
    }
    out
}

/// Build filtered SMILES tab-delimited export content (`SMILES\tTitle\tCATEGORY\tMAIN_CLASS\t-`).
#[must_use]
pub fn build_filtered_smiles_content(
    blocks: &[SpectrumBlock],
    gallery: &[GalleryItem],
    classes: &[ChemicalClass],
    criteria: &FilterCriteria<'_>,
) -> String {
    let mut out = String::new();
    for block in filter_matching_blocks(blocks, criteria) {
        let Some(smiles) = block.psm_smiles.as_deref() else {
            continue;
        };
        let tags = block_class_tags(block, gallery, classes);
        let cat = if tags.category.is_empty() {
            "-"
        } else {
            &tags.category
        };
        let mc = if tags.main_class.is_empty() {
            "-"
        } else {
            &tags.main_class
        };
        let title = block
            .title
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or("-");
        let _ = writeln!(out, "{smiles}\t{title}\t{cat}\t{mc}\t{}", tags.sub_class);
    }
    out
}

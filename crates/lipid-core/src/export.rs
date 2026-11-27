// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! LIPID MAPS metadata tag derivation and filtered MGF / SMILES export builders.

use std::borrow::Cow;
use std::fmt::Write as _;

use crate::chemical_class::ChemicalClass;
use crate::family::LipidFamily;
use crate::filter::{FilterCriteria, passes_item_filters};
use crate::lipids::types::LipidClassification;
use crate::parser::analysis::{GalleryItem, find_gallery_item};
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

fn tags_for(
    mask: crate::bitset::ClassMatchSet,
    classification: Option<LipidClassification>,
    primary_class_name: Option<&str>,
    classes: &[ChemicalClass],
) -> ClassTags {
    let mut matched = classes.iter().filter(|c| mask.contains(c.index)).peekable();
    if matched.peek().is_none() {
        let category = classification.map_or_else(
            || "Other Lipids [-]".to_string(),
            |c| c.family.category().to_string(),
        );
        let main_class = primary_class_name.unwrap_or("-").to_string();
        return ClassTags {
            category,
            main_class,
            sub_class: "-",
        };
    }
    tags_from_matched(matched, classification.map(|c| c.family), classes, mask)
}

/// Compute `CATEGORY` / `MAIN_CLASS` / `SUB_CLASS` tags from a [`GalleryItem`].
#[must_use]
pub(crate) fn class_tags_from_gallery(item: &GalleryItem, classes: &[ChemicalClass]) -> ClassTags {
    tags_for(
        item.class_matches,
        item.classification,
        item.primary_class_name.as_deref(),
        classes,
    )
}

fn tags_from_matched<'a>(
    matched: impl Iterator<Item = &'a ChemicalClass>,
    broad: Option<LipidFamily>,
    classes: &[ChemicalClass],
    mask: crate::bitset::ClassMatchSet,
) -> ClassTags {
    let filter_broad = broad.filter(|&fam| {
        classes
            .iter()
            .any(|c| mask.contains(c.index) && c.family == fam)
    });

    let mut seen_families = 0u8;
    let mut category = String::new();
    let mut main_class = String::new();
    for c in matched.filter(|c| filter_broad.is_none_or(|fam| c.family == fam)) {
        let bit = 1u8 << c.family.rank();
        if seen_families & bit == 0 {
            seen_families |= bit;
            if !category.is_empty() {
                category.push_str(" | ");
            }
            category.push_str(c.family.category());
        }
        if !main_class.is_empty() {
            main_class.push_str(" | ");
        }
        main_class.push_str(&c.name);
    }
    ClassTags {
        category,
        main_class,
        sub_class: "-",
    }
}

/// Compute fallback tags directly from a [`SpectrumBlock`].
#[must_use]
pub(crate) fn class_tags_from_block(block: &SpectrumBlock, classes: &[ChemicalClass]) -> ClassTags {
    tags_for(block.class_matches(), block.classification, None, classes)
}

/// Look up matching gallery item tags if in gallery cap, otherwise derive from block.
#[must_use]
pub(crate) fn block_class_tags(
    block: &SpectrumBlock,
    gallery: &[GalleryItem],
    classes: &[ChemicalClass],
) -> ClassTags {
    find_gallery_item(gallery, block.index).map_or_else(
        || class_tags_from_block(block, classes),
        |item| class_tags_from_gallery(item, classes),
    )
}

fn write_class_comments(out: &mut String, tags: &ClassTags) {
    for (k, v) in tags.as_pairs() {
        let _ = writeln!(out, "COMMENT=LIPID_MAPS_{k}={v}");
    }
}

/// Insert `COMMENT=LIPID_MAPS_*` lines right after `BEGIN IONS`.
#[must_use]
pub(crate) fn insert_class_comment(block_raw: &str, tags: &ClassTags) -> String {
    let mut out = String::with_capacity(block_raw.len() + 128);
    let begin_pos = block_raw.find(|c: char| !c.is_whitespace()).filter(|&pos| {
        block_raw
            .get(pos..)
            .and_then(|rest| rest.lines().next())
            .is_some_and(|line| mgf::is_ions_marker(line, "BEGIN IONS"))
    });
    if let Some(pos) = begin_pos {
        let rest = block_raw.get(pos..).unwrap_or_default();
        if let Some(newline_pos) = rest.find('\n') {
            let split_pos = pos + newline_pos + 1;
            let (before, after) = block_raw.split_at(split_pos);
            out.push_str(before);
            write_class_comments(&mut out, tags);
            out.push_str(after);
        } else {
            out.push_str(block_raw);
            out.push('\n');
            write_class_comments(&mut out, tags);
        }
    } else {
        write_class_comments(&mut out, tags);
        out.push_str(block_raw);
    }
    out
}

fn filter_matching_blocks<'a>(
    blocks: &'a [SpectrumBlock],
    criteria: &'a FilterCriteria<'a>,
) -> impl Iterator<Item = &'a SpectrumBlock> + 'a {
    blocks.iter().filter(move |block| {
        passes_item_filters(
            block.class_matches(),
            block.exact_mass,
            block.precursor_mz,
            block.adduct.as_deref(),
            criteria,
        )
    })
}

fn sanitize_tsv(value: &str) -> Cow<'_, str> {
    if value.bytes().any(|b| matches!(b, b'\t' | b'\r' | b'\n')) {
        Cow::Owned(value.replace(['\t', '\r', '\n'], " "))
    } else {
        Cow::Borrowed(value)
    }
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
        let title = sanitize_tsv(
            block
                .title
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or("-"),
        );
        let safe_smiles = sanitize_tsv(smiles);
        let _ = writeln!(
            out,
            "{safe_smiles}\t{title}\t{cat}\t{mc}\t{}",
            tags.sub_class
        );
    }
    out
}

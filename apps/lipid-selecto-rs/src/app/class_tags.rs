// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Lipid MAP class-tag extraction and MGF COMMENT insertion.
//!
//! Each `SpectrumBlock` or `GalleryItem` gets `CATEGORY` / `MAIN_CLASS` /
//! `SUB_CLASS` tags derived from its matched `ChemicalClass` entries.
//! These tags are either displayed in the gallery or injected as
//! `COMMENT=LIPID_MAPS_*` lines in exported MGF blocks.

use super::family::{family_code, family_from_category};
use crate::chemical_class::ChemicalClass;
use crate::parser::{GalleryItem, SpectrumBlock};

/// Returns LIPID MAPS class tags from a `GalleryItem`, ensuring the
/// export uses the same class assignment shown in the UI (via
/// `primary_class_name`).
///
/// `CATEGORY` comes from the broad classification, `MAIN_CLASS` from the
/// first matching LMSD class, `SUB_CLASS` is always "-".
pub(super) fn class_tags_from_gallery(
    item: &GalleryItem,
    classes: &[ChemicalClass],
) -> Vec<(String, String)> {
    let matched: Vec<&ChemicalClass> = classes
        .iter()
        .filter(|c| item.class_matches.get(&c.name).copied().unwrap_or(false))
        .collect();

    if matched.is_empty() {
        let category = item.classification.as_ref().map_or_else(
            || "Other Lipids [-]".to_string(),
            |c| c.class.lipidmaps_category().to_string(),
        );
        let main_class = item
            .primary_class_name
            .clone()
            .unwrap_or_else(|| "-".to_string());
        vec![
            ("CATEGORY".to_string(), category),
            ("MAIN_CLASS".to_string(), main_class),
            ("SUB_CLASS".to_string(), "-".to_string()),
        ]
    } else {
        class_tags_from_matched(&matched, item)
    }
}

/// Build `CATEGORY` / `MAIN_CLASS` tags from a list of matched classes,
/// preferring same-family matches when the broad classification provides one.
fn class_tags_from_matched(
    matched: &[&ChemicalClass],
    item: &GalleryItem,
) -> Vec<(String, String)> {
    let broad_family = item
        .classification
        .as_ref()
        .map_or("", |c| family_from_category(c.class.lipidmaps_category()));

    let selected: Vec<&ChemicalClass> = if broad_family.is_empty() {
        matched.to_vec()
    } else {
        let same_family: Vec<&ChemicalClass> = matched
            .iter()
            .filter(|c| c.family == broad_family)
            .copied()
            .collect();
        if same_family.is_empty() {
            matched.to_vec()
        } else {
            same_family
        }
    };

    let mut categories: Vec<String> = Vec::new();
    for c in &selected {
        let label = format!("{} [{}]", c.family, family_code(&c.family));
        if !categories.contains(&label) {
            categories.push(label);
        }
    }
    let category = categories.join(" | ");
    let main_class = selected
        .iter()
        .map(|c| c.name.as_str())
        .collect::<Vec<_>>()
        .join(" | ");

    vec![
        ("CATEGORY".to_string(), category),
        ("MAIN_CLASS".to_string(), main_class),
        ("SUB_CLASS".to_string(), "-".to_string()),
    ]
}

/// Returns LIPID MAPS class tags for a spectrum block as (key, value) pairs.
/// Used as a fallback when no matching `GalleryItem` is found.
pub(super) fn class_tags_from_block(
    block: &SpectrumBlock,
    classes: &[ChemicalClass],
) -> Vec<(String, String)> {
    let category = block.classification.as_ref().map_or_else(
        || "Other Lipids [-]".to_string(),
        |c| c.class.lipidmaps_category().to_string(),
    );

    let main_class = block
        .gallery_item_matches
        .as_ref()
        .and_then(|matches| {
            classes
                .iter()
                .find(|c| matches.get(&c.name).copied().unwrap_or(false))
                .map(|c| c.name.clone())
                .or_else(|| {
                    matches
                        .iter()
                        .find(|(_, matched)| **matched)
                        .map(|(name, _)| name.clone())
                })
        })
        .unwrap_or_else(|| "-".to_string());

    vec![
        ("CATEGORY".to_string(), category),
        ("MAIN_CLASS".to_string(), main_class),
        ("SUB_CLASS".to_string(), "-".to_string()),
    ]
}

/// Insert `COMMENT=LIPID_MAPS_*` lines with class tags in the MGF header
/// block (after `BEGIN IONS`, before the first peak line).
pub(super) fn insert_class_comment(block_raw: &str, class_tags: &[(String, String)]) -> String {
    use std::fmt::Write as _;

    let mut out = String::with_capacity(block_raw.len() + class_tags.len() * 32);
    let split_pos = block_raw
        .find(|c: char| !c.is_whitespace())
        .filter(|&begin_pos| block_raw[begin_pos..].starts_with("BEGIN IONS"))
        .and_then(|begin_pos| {
            block_raw[begin_pos..]
                .find('\n')
                .map(|newline_pos| begin_pos + newline_pos + 1)
        })
        .unwrap_or(0);
    let (before, after) = block_raw.split_at(split_pos);
    out.push_str(before);
    for (k, v) in class_tags {
        let _ = writeln!(out, "COMMENT=LIPID_MAPS_{k}={v}");
    }
    out.push_str(after);
    out
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests;

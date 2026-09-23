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
pub fn class_tags_from_gallery(
    item: &GalleryItem,
    classes: &[ChemicalClass],
) -> Vec<(String, String)> {
    let matched: Vec<&ChemicalClass> = item
        .class_matches
        .iter()
        .filter(|(_, matched)| **matched)
        .filter_map(|(name, _)| classes.iter().find(|c| &c.name == name))
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

    let category = selected
        .iter()
        .map(|c| format!("{} [{}]", c.family, family_code(&c.family)))
        .collect::<Vec<_>>()
        .join(" | ");
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
pub fn class_tags_from_block(
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
            let matched_name = matches
                .iter()
                .find(|(_, matched)| **matched)
                .map(|(name, _)| name.clone());
            matched_name.and_then(|name| {
                classes
                    .iter()
                    .find(|c| c.name == name)
                    .map(|c| c.name.clone())
                    .or(Some(name))
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
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn insert_class_comment(block_raw: &str, class_tags: &[(String, String)]) -> String {
    let comment_block: String = class_tags
        .iter()
        .map(|(k, v)| format!("COMMENT=LIPID_MAPS_{k}={v}\n"))
        .collect::<Vec<_>>()
        .concat();
    if let Some(begin_pos) = block_raw.find(|c: char| !c.is_whitespace())
        && block_raw[begin_pos..].starts_with("BEGIN IONS")
        && let Some(newline_pos) = block_raw[begin_pos..].find('\n')
    {
        let (before, after) = block_raw[begin_pos..].split_at(newline_pos + 1);
        return format!("{before}{comment_block}{after}");
    }
    format!("{comment_block}{block_raw}")
}

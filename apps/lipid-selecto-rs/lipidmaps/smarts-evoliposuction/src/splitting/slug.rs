// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

//! Slug generation, LIPID MAPS category ranking, and sampling count helpers.

use std::collections::HashMap;

use super::SplitConfig;

const LIPID_MAPS_CATEGORIES: [(&str, &str, usize); 8] = [
    ("FA", "Fatty Acyls", 0),
    ("GL", "Glycerolipids", 1),
    ("GP", "Glycerophospholipids", 2),
    ("SP", "Sphingolipids", 3),
    ("ST", "Sterol Lipids", 4),
    ("PR", "Prenol Lipids", 5),
    ("SL", "Saccharolipids", 6),
    ("PK", "Polyketides", 7),
];

pub(super) fn category_rank(category: &str) -> usize {
    let trimmed = category.split(" [").next().unwrap_or(category).trim();
    LIPID_MAPS_CATEGORIES
        .iter()
        .find(|(_, name, _)| *name == trimmed)
        .map_or(99, |(_, _, rank)| *rank)
}

pub(super) fn category_of_main(main_class: &str) -> &'static str {
    let code_source = main_class
        .rsplit_once('[')
        .map_or(main_class, |(_, rest)| rest);
    let abbr = code_source
        .get(..code_source.floor_char_boundary(2))
        .unwrap_or(code_source);
    LIPID_MAPS_CATEGORIES
        .iter()
        .find(|(code, _, _)| *code == abbr)
        .map_or("Other", |(_, name, _)| *name)
}

pub(super) fn target_negative_count(positive_count: usize, config: &SplitConfig) -> usize {
    let pos = f64::from(u32::try_from(positive_count).unwrap_or(u32::MAX));
    #[expect(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        reason = "clamped >= 0.0 and rounded into non-negative usize before capping"
    )]
    let scaled = (pos * config.neg_ratio).round().max(0.0) as usize;
    scaled.min(config.max_negatives)
}

pub(super) fn slugify(label: &str) -> String {
    slugify_with_len(label, 60)
}

pub(super) fn slugify_with_len(label: &str, maxlen: usize) -> String {
    let mut slug = String::with_capacity(label.len());
    let mut last_was_sep = false;
    for ch in label.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_sep = false;
        } else if !last_was_sep {
            slug.push('_');
            last_was_sep = true;
        }
    }
    let trimmed = slug.trim_matches('_');
    let slug = if trimmed.is_empty() {
        "unlabeled".to_string()
    } else {
        trimmed.to_string()
    };

    if slug.len() > maxlen {
        let hash = short_hash(label);
        let keep = maxlen.saturating_sub(hash.len() + 1);
        let prefix = slug.get(..slug.floor_char_boundary(keep)).unwrap_or(&slug);
        format!("{prefix}_{hash}")
    } else {
        slug
    }
}

fn short_hash(input: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{:08x}", (hash & 0xffff_ffff) as u32)
}

pub(super) fn dedupe_slug(
    slug: String,
    original_label: &str,
    owner: &mut HashMap<String, String>,
) -> String {
    if let Some(existing_owner) = owner.get(&slug) {
        if existing_owner == original_label {
            return slug;
        }
        let mut i = 2;
        loop {
            let candidate = format!("{slug}_{i}");
            if !owner.contains_key(&candidate) {
                owner.insert(candidate.clone(), original_label.to_string());
                return candidate;
            }
            i += 1;
        }
    }
    owner.insert(slug.clone(), original_label.to_string());
    slug
}

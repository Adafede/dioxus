// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Static LMSD subclass table assembly in LIPID MAPS hierarchy order.

use std::sync::LazyLock;

use super::ChemicalClass;
use super::lmsd_part1::ENTRIES_PART_1;
use super::lmsd_part2::ENTRIES_PART_2;
use crate::family::LipidFamily;

pub(super) struct LmsdEntry {
    pub(super) family: LipidFamily,
    pub(super) code: &'static str,
    pub(super) description: &'static str,
    pub(super) smarts: &'static str,
    pub(super) count: usize,
}

impl LmsdEntry {
    pub(super) const fn new(
        family: LipidFamily,
        code: &'static str,
        description: &'static str,
        smarts: &'static str,
        count: usize,
    ) -> Self {
        Self {
            family,
            code,
            description,
            smarts,
            count,
        }
    }
}

/// Static slice of all 86 precompiled LMSD main-class SMARTS entries, ordered
/// by [`LipidFamily::ALL`] and descending LMSD compound count within each family.
#[must_use]
pub fn lmsd_classes() -> &'static [ChemicalClass] {
    static CLASSES: LazyLock<Vec<ChemicalClass>> = LazyLock::new(|| {
        let total = ENTRIES_PART_1.len() + ENTRIES_PART_2.len();
        let mut result = Vec::with_capacity(total);
        for family in LipidFamily::ALL {
            let palette = family.palette();
            let mut matching: Vec<&LmsdEntry> = ENTRIES_PART_1
                .iter()
                .chain(ENTRIES_PART_2.iter())
                .filter(|e| e.family == family)
                .collect();
            matching.sort_by_key(|e| std::cmp::Reverse(e.count));
            for (rank, entry) in matching.iter().enumerate() {
                let shade = match rank {
                    0 => palette[0],
                    1 => palette[1],
                    2 => palette[2],
                    3 => palette[3],
                    _ => palette[4],
                };
                let full_name = format!("{} [{}]", entry.description, entry.code);
                let index = result.len();
                result.push(ChemicalClass::new(
                    index,
                    entry.code,
                    full_name,
                    entry.smarts,
                    shade,
                    family,
                ));
            }
        }
        result
    });
    &CLASSES
}

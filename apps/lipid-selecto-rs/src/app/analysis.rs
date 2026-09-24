// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Analysis helpers for rendering: collecting adduct options and grouping
//! chemical classes by LIPID MAPS family.

use super::family::family_rank;
use crate::chemical_class::ChemicalClass;
use crate::parser::GalleryItem;

/// Collect unique adduct values from gallery items, sorted with `+` first.
pub(super) fn collect_adduct_options(gallery: &[GalleryItem]) -> Vec<String> {
    let adduct_values: std::collections::BTreeSet<String> = gallery
        .iter()
        .filter_map(|item| item.adduct.as_ref())
        .filter(|a| !a.is_empty())
        .cloned()
        .collect();
    let mut adduct_options: Vec<String> = adduct_values.into_iter().collect();
    adduct_options.sort_by(|a, b| {
        let a_plus = a.contains('+');
        let b_plus = b.contains('+');
        match (a_plus, b_plus) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.cmp(b),
        }
    });
    adduct_options
}

/// Group chemical classes by family, sorted by LIPID MAPS rank order.
pub(super) fn group_classes_by_family(
    classes: &[ChemicalClass],
) -> Vec<(String, Vec<ChemicalClass>)> {
    let mut families: Vec<(String, Vec<ChemicalClass>)> = Vec::new();
    for class in classes {
        if let Some(entry) = families.iter_mut().find(|(f, _)| f == &class.family) {
            entry.1.push(class.clone());
        } else {
            families.push((class.family.clone(), vec![class.clone()]));
        }
    }
    families.sort_by_key(|(f, _)| family_rank(f));
    families
}

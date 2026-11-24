// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Item filtering, adduct option collection, and canonical family grouping.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use crate::bitset::ClassMatchSet;
use crate::chemical_class::ChemicalClass;
use crate::family::LipidFamily;
use crate::parser::block::SpectrumBlock;

/// Filter parameters shared by gallery rendering and filtered exports.
#[derive(Clone, Copy, Debug)]
pub struct FilterCriteria<'a> {
    /// Bitset of selected LMSD class indices.
    pub selected_classes: ClassMatchSet,
    /// Minimum calculated neutral exact mass (Da).
    pub mz_min: f64,
    /// Maximum calculated neutral exact mass (Da).
    pub mz_max: f64,
    /// Minimum observed precursor m/z (when present).
    pub precursor_min: f64,
    /// Maximum observed precursor m/z (when present).
    pub precursor_max: f64,
    /// Required `ADDUCT=` string (empty means allow any).
    pub adduct: &'a str,
}

/// Shared filter predicate used across gallery rendering and MGF/SMILES exports.
#[must_use]
pub fn passes_item_filters(
    class_matches: ClassMatchSet,
    exact_mass: f64,
    precursor_mz: Option<f64>,
    adduct: Option<&str>,
    criteria: &FilterCriteria<'_>,
) -> bool {
    if !class_matches.intersects(criteria.selected_classes) {
        return false;
    }
    if exact_mass < criteria.mz_min || exact_mass > criteria.mz_max {
        return false;
    }
    if let Some(pmz) = precursor_mz
        && (pmz < criteria.precursor_min || pmz > criteria.precursor_max)
    {
        return false;
    }
    criteria.adduct.is_empty() || adduct == Some(criteria.adduct)
}

/// Collect unique adduct values across lipid blocks, sorted with `+` adducts first.
#[must_use]
pub fn collect_adduct_options(blocks: &[SpectrumBlock]) -> Vec<String> {
    let adduct_values: BTreeSet<String> = blocks
        .iter()
        .filter(|block| block.is_lipid())
        .filter_map(|block| block.adduct.as_ref())
        .filter(|a| !a.is_empty())
        .cloned()
        .collect();
    let mut options: Vec<String> = adduct_values.into_iter().collect();
    options.sort_by(|a, b| b.contains('+').cmp(&a.contains('+')).then_with(|| a.cmp(b)));
    options
}

/// Group chemical classes by [`LipidFamily`] in canonical LIPID MAPS rank order without cloning.
#[must_use]
pub(crate) fn group_classes_by_family(
    classes: &[ChemicalClass],
) -> Vec<(LipidFamily, Vec<&ChemicalClass>)> {
    let mut families: Vec<(LipidFamily, Vec<&ChemicalClass>)> = Vec::new();
    for class in classes {
        if let Some(entry) = families.iter_mut().find(|(fam, _)| *fam == class.family) {
            entry.1.push(class);
        } else {
            families.push((class.family, vec![class]));
        }
    }
    families.sort_by_key(|(fam, _)| fam.rank());
    families
}

/// Static grouping of all compiled LMSD classes by [`LipidFamily`] in rank order.
#[must_use]
pub fn lmsd_classes_by_family() -> &'static [(LipidFamily, Vec<&'static ChemicalClass>)] {
    static GROUPED: LazyLock<Vec<(LipidFamily, Vec<&'static ChemicalClass>)>> =
        LazyLock::new(|| group_classes_by_family(crate::chemical_class::lmsd::lmsd_classes()));
    GROUPED.as_slice()
}

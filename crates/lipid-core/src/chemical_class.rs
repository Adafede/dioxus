// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Precompiled LMSD chemical class SMARTS definitions.

#[cfg(test)]
mod defaults;
pub(crate) mod lmsd;
mod lmsd_part1;
mod lmsd_part2;

use chematic::core::Molecule;
use chematic::smarts;

use crate::family::LipidFamily;

/// A chemical class defined by an indexed LMSD name, precompiled SMARTS pattern,
/// display color, and parent [`LipidFamily`].
#[derive(Clone, Debug)]
pub struct ChemicalClass {
    /// Deterministic bit index in [`lmsd::lmsd_classes`] (`0..86`).
    pub index: usize,
    /// Display name (e.g. `"Fatty Acids and Conjugates [FA01]"`).
    pub name: String,
    /// Raw SMARTS pattern string.
    pub smarts: &'static str,
    /// Microshades hex color code for UI badges and cards.
    pub color: &'static str,
    /// Canonical LIPID MAPS parent family.
    pub family: LipidFamily,
    compiled: Option<smarts::QueryMolecule>,
}

impl ChemicalClass {
    /// Construct and precompile a [`ChemicalClass`] entry.
    #[must_use]
    pub(crate) fn new(
        index: usize,
        name: impl Into<String>,
        smarts: &'static str,
        color: &'static str,
        family: LipidFamily,
    ) -> Self {
        let compiled = smarts::parse_smarts(smarts).ok();
        Self {
            index,
            name: name.into(),
            smarts,
            color,
            family,
            compiled,
        }
    }

    /// Returns `true` when the SMARTS string compiled into a valid query.
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn is_compiled(&self) -> bool {
        self.compiled.is_some()
    }

    /// Returns `true` if `molecule` matches this class's precompiled SMARTS query.
    #[must_use]
    pub(crate) fn matches(&self, molecule: &Molecule) -> bool {
        let Some(query) = &self.compiled else {
            return false;
        };
        smarts::has_match_with_config(query, molecule, &smarts::MatchConfig::default())
    }

    #[cfg(test)]
    pub(crate) fn defaults() -> Vec<Self> {
        defaults::all()
    }
}

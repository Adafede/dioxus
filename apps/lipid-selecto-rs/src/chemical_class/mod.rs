// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! User-defined chemical classes with SMARTS pattern matching.
//!
//! This module provides two complementary classification systems:
//!
//! - **Broad classes** ([`ChemicalClass::defaults`]): coarse-grained family-level
//!   SMARTS patterns (e.g. "FA", "PC(AA)") used for color attribution and
//!   simple lipid detection.
//! - **LMSD subclasses** ([`lmsd_all`]): the 58 LIPID MAPS Structure Database
//!   subclasses (FA01–FA13, GL01–GL07, …) with specific SMARTS patterns, used
//!   for precise class assignment in exports and gallery matching.
//!
//! Both systems share the [`ChemicalClass`] type and the same pre-compiled
//! SMARTS matching engine.

#[cfg(test)]
mod defaults;
mod lmsd;

#[cfg(test)]
use std::collections::HashMap;

use chematic::smarts;

#[cfg(test)]
use defaults::{
    fatty_acyls, glycerolipids, glycerophospholipids, polyketides, prenol_lipids, saccharolipids,
    sphingolipids, sterol_lipids,
};
pub(crate) use lmsd::lmsd_all;

// === Palette constants ===

// Microshades palettes — shade 0-3 for first 4 classes per family,
// shade 4 for the rest.
pub(super) const FA_PALETTE: [&str; 5] = ["#4E7705", "#6D9F06", "#97CE2F", "#BDEC6F", "#DDFFA0"];
pub(super) const GL_PALETTE: [&str; 5] = ["#098BD9", "#56B4E9", "#7DCCFF", "#BCE1FF", "#E7F4FF"];
pub(super) const GP_PALETTE: [&str; 5] = ["#7D3560", "#A1527F", "#CC79A7", "#E794C1", "#EFB6D6"];
pub(super) const SP_PALETTE: [&str; 5] = ["#9D654C", "#C17754", "#F09163", "#FCB076", "#FFD5AF"];
pub(super) const ST_PALETTE: [&str; 5] = ["#238b45", "#41ab5d", "#74c476", "#a1d99b", "#c7e9c0"];
pub(super) const PR_PALETTE: [&str; 5] = ["#4292c6", "#6baed6", "#9ecae1", "#c6dbef", "#eff3ff"];
pub(super) const SL_PALETTE: [&str; 5] = ["#6a51a3", "#807dba", "#9e9ac8", "#bcbddc", "#dadaeb"];
pub(super) const PK_PALETTE: [&str; 5] = ["#ff7f00", "#fe9929", "#fdae6b", "#fec44f", "#feeda0"];

/// A chemical class defined by name, SMARTS pattern, display color, and family.
///
/// SMARTS patterns are pre-compiled once at construction time to avoid
/// re-parsing the pattern string on every `matches` call — critical for large
/// datasets where thousands of molecules are matched against dozens of classes.
#[derive(Clone, Debug)]
pub(crate) struct ChemicalClass {
    /// Display name of the lipid class (e.g. "FA", "Cer(AS)").
    pub name: String,
    /// SMARTS pattern string (as defined in the constructor).
    #[cfg(test)]
    pub smarts: String,
    /// Hex color code for UI rendering.
    pub color: String,
    /// LIPID MAPS broad family name (e.g. "Fatty Acyls", "Sphingolipids").
    pub family: String,
    /// Pre-compiled SMARTS query (parsed once in `new`).
    ///
    /// Ungated, and written on every target, so that `new` below is one shape
    /// rather than four. It used to be `cfg`-gated, and `new` with it, into a
    /// combination no build had ever compiled at once: the host-test shape
    /// moved `smarts_str` into a `drop` and then read it again, and did not
    /// build.
    ///
    /// Read only where a molecule can be matched, which is the analysis pipeline:
    /// the wasm app, and tests that are themselves `all(test, wasm32)`. A host
    /// build — with or without a test harness — writes this field and never looks
    /// at it, so `not(wasm32)` is the exact set of builds in which the lint
    /// fires.
    ///
    /// Kept as an `allow` rather than a `cfg` on the field, because the
    /// constructor above is one shape on every target and gating the field is
    /// what made it four.
    #[allow(dead_code)] // see above
    compiled: Option<smarts::QueryMolecule>,
}

impl ChemicalClass {
    /// Create a new chemical class, pre-compiling the SMARTS pattern.
    pub(crate) fn new(
        name: impl Into<String>,
        smarts_str: impl Into<String>,
        color: impl Into<String>,
        family: impl Into<String>,
    ) -> Self {
        // One shape, not four. The pattern is compiled on every target, so
        // there is no branch here that a build has not compiled, and no
        // `drop(smarts_str)` to keep an unused variable quiet — the value is
        // always read.
        let smarts = smarts_str.into();
        Self {
            name: name.into(),
            #[cfg(test)]
            smarts: smarts.clone(),
            color: color.into(),
            family: family.into(),
            compiled: smarts::parse_smarts(&smarts).ok(),
        }
    }

    /// Check if a molecule matches this class's pre-compiled SMARTS pattern.
    ///
    /// Returns `true` if the molecule contains at least one match, `false` otherwise
    /// or if the SMARTS pattern cannot be parsed.
    #[cfg(any(test, target_arch = "wasm32"))]
    #[must_use]
    pub(crate) fn matches(&self, molecule: &chematic::core::Molecule) -> bool {
        let Some(query) = &self.compiled else {
            return false;
        };
        !smarts::find_matches(query, molecule).is_empty()
    }

    /// Return the default lipid classes.
    ///
    /// These match the LIPID MAPS classification system with proper family and
    /// architecture designations.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn defaults() -> Vec<Self> {
        [
            fatty_acyls(),
            glycerolipids(),
            glycerophospholipids(),
            sphingolipids(),
            sterol_lipids(),
            prenol_lipids(),
            saccharolipids(),
            polyketides(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    /// Convert defaults into a map for quick lookup by name.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn defaults_map() -> HashMap<String, Self> {
        Self::defaults()
            .into_iter()
            .map(|c| (c.name.clone(), c))
            .collect()
    }
}
#[cfg(test)]
#[expect(clippy::unwrap_used)] // tests unwrap fixture lookups to fail-fast if the default set changes
#[expect(clippy::expect_used)] // tests expect known default classes / valid SMILES fixtures
mod tests;

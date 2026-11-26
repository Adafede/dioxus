// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Spectrum/SMILES record representation and single-pass SMARTS + formula classification.

use std::fmt;

use chematic::chem;
use chematic::core::{Element, Molecule};
use mgf::parse_leading_float;

use crate::bitset::ClassMatchSet;
use crate::chemical_class::ChemicalClass;
use crate::family::LipidFamily;
use crate::lipids::classify::{classify_acyclic_molecule, classify_formula, is_acyclic};
use crate::lipids::types::LipidClassification;

/// One `BEGIN IONS ... END IONS` MGF spectrum or SMILES row.
#[derive(Clone)]
pub struct SpectrumBlock {
    /// 1-based position within the uploaded file.
    pub index: usize,
    /// Spectrum or compound title (`TITLE=` / `NAME=` / `SCANS=` or SMILES label).
    pub title: Option<String>,
    /// Annotated SMILES string.
    pub psm_smiles: Option<String>,
    /// Annotated molecular formula (`FORMULA=`).
    pub formula: Option<String>,
    /// Annotated charge (`CHARGE=`).
    pub charge: Option<String>,
    /// Annotated adduct (`ADDUCT=`, e.g. `"[M+H]+"`).
    pub adduct: Option<String>,
    /// Normalized lowercase ion mode (`IONMODE=`).
    pub ion_mode: Option<String>,
    /// Observed precursor m/z from `PEPMASS=` or `PRECURSOR_MZ=`.
    pub precursor_mz: Option<f64>,
    /// Broad lipid classification assignment.
    pub classification: Option<LipidClassification>,
    /// Compact 128-bit bitset of matched LMSD class indices.
    pub gallery_item_matches: Option<ClassMatchSet>,
    /// Calculated monoisotopic mass (`0.0` when unavailable).
    pub exact_mass: f64,
    /// Verbatim text of the block (used for filtered MGF export).
    pub raw: String,
    pub(crate) parsed_molecule: Option<Molecule>,
}

impl fmt::Debug for SpectrumBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpectrumBlock")
            .field("index", &self.index)
            .field("title", &self.title)
            .field("psm_smiles", &self.psm_smiles)
            .field("formula", &self.formula)
            .field("adduct", &self.adduct)
            .field("precursor_mz", &self.precursor_mz)
            .field("classification", &self.classification)
            .field("gallery_item_matches", &self.gallery_item_matches)
            .field("exact_mass", &self.exact_mass)
            .finish_non_exhaustive()
    }
}

impl SpectrumBlock {
    /// Construct an empty block with 1-based `index`.
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self {
            index,
            title: None,
            psm_smiles: None,
            formula: None,
            charge: None,
            adduct: None,
            ion_mode: None,
            precursor_mz: None,
            classification: None,
            gallery_item_matches: None,
            exact_mass: 0.0,
            raw: String::new(),
            parsed_molecule: None,
        }
    }

    /// Returns `true` when this spectrum matched at least one LMSD class.
    #[must_use]
    pub fn is_lipid(&self) -> bool {
        self.gallery_item_matches
            .is_some_and(|matches| !matches.is_empty())
    }

    /// Returns matched LMSD classes as a compact [`ClassMatchSet`].
    #[must_use]
    pub fn class_matches(&self) -> ClassMatchSet {
        self.gallery_item_matches.unwrap_or_default()
    }

    pub(super) fn consume_metadata(&mut self, line: &str) {
        let Some((key, value)) = line.trim().split_once('=') else {
            return;
        };
        let value = value.trim();
        match key {
            "SMILES" => {
                self.psm_smiles = Some(value.to_string());
                self.parsed_molecule = None;
            }
            "FORMULA" => self.formula = Some(value.to_string()),
            "CHARGE" => self.charge = Some(value.to_string()),
            "ADDUCT" => self.adduct = Some(value.to_string()),
            "IONMODE" => self.ion_mode = Some(value.to_ascii_lowercase()),
            "PEPMASS" | "PRECURSOR_MZ" => self.precursor_mz = parse_leading_float(value),
            "TITLE" => self.title = Some(value.to_string()),
            "NAME" | "SCANS" if self.title.is_none() => self.title = Some(value.to_string()),
            _ => {}
        }
    }

    pub(super) fn consume_inline_header(&mut self, header: &str) {
        for token in header.split_whitespace() {
            if token.eq_ignore_ascii_case("BEGIN") || token.eq_ignore_ascii_case("IONS") {
                continue;
            }
            if token.contains('=') {
                self.consume_metadata(token);
            }
        }
    }

    /// Classify this block once and populate [`SpectrumBlock::gallery_item_matches`],
    /// reusing any already-parsed [`Molecule`] to avoid redundant SMILES parsing.
    pub fn classify_and_match(&mut self, classes: &[ChemicalClass]) {
        let mut smiles_classification = None;
        let mut matches = ClassMatchSet::empty();

        let mol_opt = self.parsed_molecule.take().or_else(|| {
            let smiles_str = self.psm_smiles.as_deref().map(str::trim)?;
            (!smiles_str.is_empty())
                .then(|| chem_depict::parse_smiles_safe(smiles_str).ok())
                .flatten()
        });

        if let Some(molecule) = mol_opt {
            self.exact_mass = chem::exact_mass(&molecule);
            if is_acyclic(&molecule) {
                smiles_classification =
                    classify_acyclic_molecule(&molecule).map(|family| LipidClassification {
                        family,
                        derived_from_smiles: true,
                    });
            }
            for class in classes {
                if class.matches(&molecule) {
                    if smiles_classification.is_none() {
                        smiles_classification = Some(LipidClassification {
                            family: class.family,
                            derived_from_smiles: true,
                        });
                    }
                    matches.insert(class.index);
                }
            }
            if !matches.is_empty() {
                self.parsed_molecule = Some(molecule);
            }
            self.classification = smiles_classification;
        } else {
            if self.exact_mass <= 0.0
                && let Some(formula_str) = self.formula.as_deref()
            {
                self.exact_mass = formula_exact_mass(formula_str);
            }

            self.classification = self.formula.as_deref().and_then(|formula_str| {
                let family = classify_formula(formula_str)?;
                if let Some(idx) = formula_fallback_class_index(classes, family) {
                    matches.insert(idx);
                }
                Some(LipidClassification {
                    family,
                    derived_from_smiles: false,
                })
            });
        }
        self.gallery_item_matches = Some(matches);
    }
}

fn formula_exact_mass(formula_str: &str) -> f64 {
    let Ok(counts) = chem::parse_formula(formula_str) else {
        return 0.0;
    };
    counts
        .into_iter()
        .filter_map(|(symbol, count)| {
            let element = Element::from_symbol(&symbol)?;
            Some(element.atomic_mass() * f64::from(count))
        })
        .sum()
}

fn formula_fallback_class_index(classes: &[ChemicalClass], family: LipidFamily) -> Option<usize> {
    let code = family.fallback_code();
    classes
        .iter()
        .find(|class| class.name.contains(code))
        .or_else(|| classes.iter().find(|class| class.family == family))
        .map(|class| class.index)
}

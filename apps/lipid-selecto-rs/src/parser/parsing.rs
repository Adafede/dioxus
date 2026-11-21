// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! MGF `BEGIN IONS ... END IONS` block parsing and the `SpectrumBlock` domain
//! type.
//!
// A `SpectrumBlock` owns both its parsed metadata and the chemical-class
// classification derived from it, so `classify_with_compiled` and
// `compute_class_matches_with_compiled` live alongside the block structure
// (rather than in the aggregating `analysis` module). This keeps block-level
// parsing and per-block classification in one cohesive unit; cross-block
// aggregation lives in `super::analysis`.

#[cfg(any(test, target_arch = "wasm32"))]
use crate::chemical_class::ChemicalClass;
#[cfg(any(test, target_arch = "wasm32"))]
use crate::lipids::LipidClass;
use crate::lipids::LipidClassification;
#[cfg(any(test, target_arch = "wasm32"))]
use crate::lipids::{classify_acyclic_molecule, classify_formula, is_acyclic};
#[cfg(any(test, target_arch = "wasm32"))]
use chematic::chem;
#[cfg(any(test, target_arch = "wasm32"))]
use chematic::smiles;
use std::collections::HashMap;

/// One `BEGIN IONS ... END IONS` record from the source MGF, together with the
/// metadata fields relevant to lipid selection.
#[derive(Clone, Debug)]
pub(crate) struct SpectrumBlock {
    /// 1-based position of the block within the original file.
    ///
    /// Used as a stable key for re-associating gallery items with blocks
    /// during filtered-MGF export.
    pub index: usize,
    /// Spectrum title (`TITLE=` / `NAME=`), if present.
    pub title: Option<String>,
    /// `SMILES=` value, if present.
    pub psm_smiles: Option<String>,
    /// `FORMULA=` value, if present.
    #[cfg(any(test, target_arch = "wasm32"))]
    pub formula: Option<String>,
    /// `CHARGE=` value, if present.
    #[cfg(any(test, target_arch = "wasm32"))]
    pub charge: Option<String>,
    /// `ADDUCT=` value, if present (e.g., "[M+H]+", "[M-H]-").
    pub adduct: Option<String>,
    /// `IONMODE=` value (normalized to lower case), if present.
    #[cfg(any(test, target_arch = "wasm32"))]
    pub ion_mode: Option<String>,
    /// Observed precursor m/z from `PEPMASS=` / `PRECURSOR_MZ=`.
    pub precursor_mz: Option<f64>,
    /// Result of lipid classification (populated by `SpectrumBlock::classify_with_compiled`).
    pub classification: Option<LipidClassification>,
    /// Maps chemical class name -> bool (does this spectrum match?)
    pub gallery_item_matches: Option<HashMap<String, bool>>,
    /// Pre-computed exact mass from the parsed SMILES (avoids re-parsing
    /// the SMILES in gallery rendering — the dominant cost for large files).
    pub exact_mass: f64,
    /// Verbatim text of the block, from `BEGIN IONS` through `END IONS`.
    pub raw: String,
}

impl SpectrumBlock {
    /// Create a new empty block with the given 1-based index.
    #[must_use]
    #[cfg(any(test, target_arch = "wasm32"))]
    pub(crate) const fn new(index: usize) -> Self {
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
        }
    }

    /// Parse a single metadata line (`KEY=VALUE`) into the appropriate field.
    #[cfg(any(test, target_arch = "wasm32"))]
    fn consume_metadata(&mut self, line: &str) {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("SMILES=") {
            self.psm_smiles = Some(value.trim().to_string());
            return;
        }
        if let Some(value) = trimmed.strip_prefix("FORMULA=") {
            self.formula = Some(value.trim().to_string());
            return;
        }
        if let Some(value) = trimmed.strip_prefix("CHARGE=") {
            self.charge = Some(value.trim().to_string());
            return;
        }
        if let Some(value) = trimmed.strip_prefix("ADDUCT=") {
            self.adduct = Some(value.trim().to_string());
            return;
        }
        if let Some(value) = trimmed.strip_prefix("IONMODE=") {
            self.ion_mode = Some(value.trim().to_ascii_lowercase());
            return;
        }
        if let Some(value) = trimmed.strip_prefix("PEPMASS=") {
            self.precursor_mz = parse_first_float(value);
            return;
        }
        if let Some(value) = trimmed.strip_prefix("PRECURSOR_MZ=") {
            self.precursor_mz = value.trim().parse().ok();
            return;
        }
        if let Some(value) = trimmed.strip_prefix("TITLE=") {
            self.title = Some(value.trim().to_string());
            return;
        }
        if let Some(value) = trimmed.strip_prefix("NAME=") {
            if self.title.is_none() {
                self.title = Some(value.trim().to_string());
            }
            return;
        }
        if let Some(value) = trimmed.strip_prefix("SCANS=")
            && self.title.is_none()
        {
            self.title = Some(value.trim().to_string());
        }
    }

    /// Parse the inline header tokens emitted on the `BEGIN IONS` line, e.g.
    /// `BEGIN IONS SMILES=... PEPMASS=100.0 CHARGE=1+`.
    #[cfg(any(test, target_arch = "wasm32"))]
    fn consume_inline_header(&mut self, header: &str) {
        for token in header.split_whitespace() {
            if token.eq_ignore_ascii_case("BEGIN") || token.eq_ignore_ascii_case("IONS") {
                continue;
            }
            if token.contains('=') {
                self.consume_metadata(token);
            }
        }
    }

    /// `true` when this spectrum has at least one matching chemical class.
    #[must_use]
    pub(crate) fn is_lipid(&self) -> bool {
        self.gallery_item_matches
            .as_ref()
            .is_some_and(|matches| matches.values().any(|&m| m))
    }

    /// Classify this block and compute its chemical-class matches in a single
    /// pass.
    ///
    /// Parses the SMILES once, caches `exact_mass`, checks acyclicity once, and
    /// evaluates both the broad lipid classifier (with `FORMULA=` fallback) and
    /// all LMSD chemical classes against the same parsed [`chematic::core::Molecule`].
    #[cfg(any(test, target_arch = "wasm32"))]
    pub(crate) fn classify_and_match(&mut self, classes: &[ChemicalClass]) {
        let mut smiles_classification = None;
        let mut matches = HashMap::new();

        if let Some(smiles_str) = self.psm_smiles.as_deref().map(str::trim)
            && !smiles_str.is_empty()
            && let Some(Ok(molecule)) =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| smiles::parse(smiles_str)))
                    .ok()
        {
            self.exact_mass = chem::exact_mass(&molecule);
            if is_acyclic(&molecule) {
                smiles_classification =
                    classify_acyclic_molecule(&molecule).map(|class| LipidClassification {
                        class,
                        derived_from_smiles: true,
                    });
            }
            for class in classes {
                if class.matches(&molecule) {
                    if smiles_classification.is_none()
                        && let Some(broad) = LipidClass::from_family(&class.family)
                    {
                        smiles_classification = Some(LipidClassification {
                            class: broad,
                            derived_from_smiles: true,
                        });
                    }
                    matches.insert(class.name.clone(), true);
                }
            }
        }

        if self.exact_mass <= 0.0
            && let Some(formula_str) = self.formula.as_deref()
        {
            self.exact_mass = formula_exact_mass(formula_str);
        }

        self.classification = smiles_classification.or_else(|| {
            let formula_str = self.formula.as_deref()?;
            let class = classify_formula(formula_str)?;
            if let Some(name) = formula_fallback_class_name(classes, class) {
                matches.insert(name.to_string(), true);
            }
            Some(LipidClassification {
                class,
                derived_from_smiles: false,
            })
        });
        self.gallery_item_matches = Some(matches);
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
fn parse_first_float(text: &str) -> Option<f64> {
    text.split_whitespace()
        .next()
        .and_then(|value| value.parse::<f64>().ok())
}

/// Detect an MGF section marker (`BEGIN IONS` / `END IONS`, case-insensitive).
#[cfg(any(test, target_arch = "wasm32"))]
fn is_ions_marker(line: &str, marker: &str) -> bool {
    let trimmed = line.trim();
    trimmed
        .get(..marker.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(marker))
        && trimmed
            .get(marker.len()..)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with(char::is_whitespace))
}

/// Incremental streaming builder that splits MGF or SMILES lines into
/// [`SpectrumBlock`] records without buffering the entire file's lines first.
#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Default)]
pub(crate) struct BlockExtractor {
    blocks: Vec<SpectrumBlock>,
    index: usize,
    current: Option<SpectrumBlock>,
    format: Option<crate::format::LipidFormat>,
    saw_mgf: bool,
}

#[cfg(any(test, target_arch = "wasm32"))]
impl BlockExtractor {
    /// Construct an extractor constrained to an optional known input format.
    #[must_use]
    pub(crate) fn with_format(format: Option<crate::format::LipidFormat>) -> Self {
        Self {
            format,
            ..Self::default()
        }
    }

    /// Return the resolved input format after inspecting pushed lines.
    #[must_use]
    pub(crate) fn detected_format(&self) -> Option<crate::format::LipidFormat> {
        self.format
            .or_else(|| (!self.blocks.is_empty()).then_some(crate::format::LipidFormat::Smiles))
    }

    /// Feed a single line into the block state machine.
    pub(crate) fn push_line(&mut self, line: &str) {
        if self.format == Some(crate::format::LipidFormat::Smiles) {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return;
            }
            self.index += 1;
            if let Some(block) = parse_smiles_line(self.index, trimmed) {
                self.blocks.push(block);
            }
            return;
        }

        if is_ions_marker(line, "BEGIN IONS") {
            if !self.saw_mgf {
                self.saw_mgf = true;
                if self.format.is_none() {
                    self.blocks.clear();
                    self.index = 0;
                    self.format = Some(crate::format::LipidFormat::Mgf);
                }
            }
            if let Some(block) = self.current.take() {
                self.blocks.push(block);
            }
            self.index += 1;
            let mut block = SpectrumBlock::new(self.index);
            block.consume_inline_header(line);
            block.raw.push_str(line);
            block.raw.push('\n');
            self.current = Some(block);
            return;
        }

        if let Some(block) = self.current.as_mut() {
            if is_ions_marker(line, "END IONS") {
                block.raw.push_str(line);
                block.raw.push('\n');
                if let Some(finished) = self.current.take() {
                    self.blocks.push(finished);
                }
                return;
            }
            block.consume_metadata(line);
            block.raw.push_str(line);
            block.raw.push('\n');
        } else if self.format.is_none() {
            // Unknown extension before any `BEGIN IONS`: tentatively parse as SMILES lines.
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || is_ions_marker(line, "END IONS") {
                return;
            }
            self.index += 1;
            if let Some(block) = parse_smiles_line(self.index, trimmed) {
                self.blocks.push(block);
            }
        }
    }

    /// Finish extraction and return all parsed blocks.
    #[must_use]
    pub(crate) fn finish(mut self) -> Vec<SpectrumBlock> {
        if let Some(block) = self.current.take() {
            self.blocks.push(block);
        }
        self.blocks
    }
}

/// Split raw MGF or SMILES text into [`SpectrumBlock`] records.
///
/// Accepts any iterator of line-like items so callers can stream from a
/// `upload::BlobLines` reader without buffering the entire file as a single
/// string.
///
/// For MGF: Each block's [`SpectrumBlock::raw`] preserves the original lines.
/// For SMILES: Each line becomes a `SpectrumBlock` with SMILES in the `psm_smiles` field.
#[must_use]
#[cfg(test)]
pub(crate) fn extract_blocks_from_lines<'a, I: Iterator<Item = impl AsRef<str> + 'a>>(
    lines: I,
) -> Vec<SpectrumBlock> {
    let mut extractor = BlockExtractor::default();
    for line in lines {
        extractor.push_line(line.as_ref());
    }
    extractor.finish()
}

/// Parse a single SMILES-format line into an optional [`SpectrumBlock`].
#[cfg(any(test, target_arch = "wasm32"))]
fn formula_exact_mass(formula_str: &str) -> f64 {
    let Ok(counts) = chem::parse_formula(formula_str) else {
        return 0.0;
    };
    counts
        .into_iter()
        .filter_map(|(symbol, count)| {
            let element = chematic::core::Element::from_symbol(&symbol)?;
            Some(element.atomic_mass() * f64::from(count))
        })
        .sum()
}

#[cfg(any(test, target_arch = "wasm32"))]
fn formula_fallback_class_name(classes: &[ChemicalClass], broad: LipidClass) -> Option<&str> {
    let code = match broad {
        LipidClass::FattyAcyl => "[FA00]",
        LipidClass::Glycerolipid => "[GL00]",
        LipidClass::Glycerophospholipid => "[GP00]",
        LipidClass::Sphingolipid => "[SP00]",
        LipidClass::SterolLipid => "[ST00]",
        LipidClass::PrenolLipid => "[PR00]",
        LipidClass::Saccharolipid => "[SL00]",
        LipidClass::Polyketide => "[PK00]",
    };
    classes
        .iter()
        .find(|class| class.name.contains(code))
        .or_else(|| {
            classes
                .iter()
                .find(|class| LipidClass::from_family(&class.family) == Some(broad))
        })
        .map(|class| class.name.as_str())
}

#[cfg(any(test, target_arch = "wasm32"))]
fn parse_smiles_line(index: usize, trimmed: &str) -> Option<SpectrumBlock> {
    let (first, second) = if let Some((left, right)) = trimmed.split_once('\t') {
        let first = left.trim();
        let right_col = right
            .split('\t')
            .next()
            .map(str::trim)
            .filter(|s| !s.is_empty());
        (first, right_col)
    } else if let Some((left, right)) = trimmed.split_once(char::is_whitespace) {
        let first = left.trim();
        let second = Some(right.trim()).filter(|s| !s.is_empty());
        (first, second)
    } else {
        (trimmed.trim(), None)
    };
    if first.is_empty() {
        return None;
    }

    let is_valid_smiles = |s: &str| {
        !s.contains(char::is_whitespace)
            && std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| smiles::parse(s).is_ok()))
                .ok()
                .unwrap_or(false)
    };
    let looks_like_id = |s: &str| {
        s.chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    };

    // A tab-separated line is either `smiles<TAB>id` or `id<TAB>smiles`; the
    // field that parses as a SMILES (and is not a bare alphanumeric identifier
    // when both parse) goes in `psm_smiles`.
    let (smiles_str, id_str) = second.map_or((first, ""), |sec| {
        let first_ok = is_valid_smiles(first);
        let sec_ok = is_valid_smiles(sec);
        match (first_ok, sec_ok) {
            (true, false) => (first, sec),
            (false, true) => (sec, first),
            (true, true) => {
                if looks_like_id(first) && !looks_like_id(sec) {
                    (sec, first)
                } else {
                    (first, sec)
                }
            }
            (false, false) => {
                if looks_like_id(sec) {
                    (first, sec)
                } else {
                    (sec, first)
                }
            }
        }
    });

    let mut block = SpectrumBlock::new(index);
    block.psm_smiles = Some(smiles_str.to_string());
    if !id_str.is_empty() {
        block.title = Some(id_str.to_string());
    }
    block.raw = trimmed.to_string();
    block.raw.push('\n');
    Some(block)
}

/// Split raw MGF or SMILES text into [`SpectrumBlock`] records.
///
/// Convenience wrapper around [`extract_blocks_from_lines`] for callers that
/// already have the full text in memory (tests, native builds).
#[cfg(test)]
#[must_use]
pub(crate) fn extract_blocks(content: &str) -> Vec<SpectrumBlock> {
    extract_blocks_from_lines(content.lines())
}

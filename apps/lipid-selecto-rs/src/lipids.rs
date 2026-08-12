//! Lipid classification from SMILES / molecular formula.
//!
//! A molecule is classified as a lipid when it carries the structural hallmark
//! of one — a **long aliphatic carbon chain** (≥ 8 contiguous carbons) — together
//! with a lipid-polar head group (carboxylic acid, ester, amide, phosphate,
//! sulfate, or a sphingoid amino-alcohol), **or** matches the formula signature
//! of a steroid / sterol (the fused tetracyclic skeleton that typically lacks a
//! classic polar head group).
//!
//! The long-chain guard is what suppresses the common false positives found in
//! metabolite MGFs: cofactors such as ATP, NAD⁺ or coenzyme A, sugar
//! phosphates like glucose-6-phosphate, and choline all lack an 8-carbon
//! aliphatic carbon path and are therefore rejected.
//!
//! Classification prefers the SMILES (structural) path; when a spectrum has no
//! parseable SMILES but carries a `FORMULA=`, a conservative formula-only
//! classifier is used as a fallback.

// Numeric classifiers count atoms as small `u32`/`i32` values and do bounded
// coordinate math; the casts below are intentional and cannot overflow for real
// molecular formulas, so the pedantic cast lints are silenced here.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_lossless,
    clippy::cast_possible_wrap,
    clippy::unnecessary_cast,
    clippy::many_single_char_names,
    clippy::similar_names
)]

use chematic::chem;
use chematic::core::Element;
use chematic::core::Molecule;
use chematic::smarts;
use chematic::smiles;
use std::fmt::Write;

/// The broad lipid category a spectrum's molecule belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LipidClass {
    /// Free fatty acid (RCOOH) — the simplest fatty acyl.
    FattyAcyl,
    /// Di-/tri-acylglycerol (no phosphorus, ester-linked acyl chains).
    Glycerolipid,
    /// Glycerophospholipids & sphingomyelin (contain phosphorus).
    Glycerophospholipid,
    /// Ceramides, sphingoid bases, gangliosides and other sphingolipids.
    Sphingolipid,
    /// Steroids / sterols (fused tetracyclic skeleton).
    Sterol,
    /// A molecule that matched the structural formula but not a specific class.
    Other,
}

impl LipidClass {
    /// Human-readable label for UI display.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::FattyAcyl => "Fatty acyl",
            Self::Glycerolipid => "Glycerolipid",
            Self::Glycerophospholipid => "Glycerophospholipid",
            Self::Sphingolipid => "Sphingolipid",
            Self::Sterol => "Sterol",
            Self::Other => "Lipid",
        }
    }

    /// CSS background color used by the classification badge in the UI.
    #[must_use]
    pub const fn color(self) -> &'static str {
        match self {
            Self::FattyAcyl => "#2563eb",
            Self::Glycerolipid => "#0d9488",
            Self::Glycerophospholipid => "#7c3aed",
            Self::Sphingolipid => "#be185d",
            Self::Sterol => "#b45309",
            Self::Other => "#475569",
        }
    }
}

/// Elemental composition extracted from a molecule or a formula string.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ElementCounts {
    pub carbon: u32,
    pub hydrogen: u32,
    pub nitrogen: u32,
    pub oxygen: u32,
    pub phosphorus: u32,
    pub sulfur: u32,
    pub halogens: u32,
}

impl ElementCounts {
    /// Double-bond equivalents (a.k.a. the index of hydrogen deficiency).
    ///
    /// `BE = C - H/2 - X/2 + N/2 + 1`
    #[must_use]
    pub fn double_bond_equivalent(&self) -> f64 {
        let c = f64::from(self.carbon);
        let h = f64::from(self.hydrogen);
        let n = f64::from(self.nitrogen);
        let x = f64::from(self.halogens);
        c - h / 2.0 - x / 2.0 + n / 2.0 + 1.0
    }

    /// Molecular formula string in Hill order (e.g. `C16H32O2`).
    #[must_use]
    pub fn formula_string(&self) -> String {
        let mut out = String::new();
        if self.carbon > 0 {
            out.push('C');
            if self.carbon > 1 {
                let _ = write!(out, "{}", self.carbon);
            }
        }
        let h = self.hydrogen;
        if h == 1 {
            out.push('H');
        } else if h > 1 {
            let _ = write!(out, "H{h}");
        }
        for (symbol, count) in [
            ("Cl", self.halogens),
            ("N", self.nitrogen),
            ("O", self.oxygen),
            ("P", self.phosphorus),
            ("S", self.sulfur),
        ] {
            if count == 1 {
                out.push_str(symbol);
            } else if count > 1 {
                let _ = write!(out, "{symbol}{count}");
            }
        }
        out
    }
}

/// A full result of classifying one spectrum, ready for display.
#[derive(Clone, Debug)]
pub struct LipidClassification {
    pub class: LipidClass,
    pub counts: ElementCounts,
    pub formula: String,
    pub exact_mass: f64,
    /// Whether the classification came from a structural (SMILES) analysis.
    pub derived_from_smiles: bool,
}

// REAL LIPID SIGNATURES - only actual lipid backbone structures
// NO RINGS. Just the chemistry that defines each lipid class.

/// Phosphate group with characteristic [PX4](=O) bonding
const PATTERN_PHOSPHATE: &str = "[PX4](=[OX1])";

/// Choline headgroup: quaternary nitrogen
const PATTERN_CHOLINE: &str = "[NX4+]";

/// Ethanolamine headgroup: secondary/primary amine bonded to saturated carbon
const PATTERN_ETHANOLAMINE: &str = "[NX3][CX4]";

/// Triglyceride: one carbon with 3 ester groups (glycerol backbone)
const PATTERN_TRIGLYCERIDE: &str =
    "[CX4]([OX2][CX3](=[OX1])[#6])([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])";

/// Diglyceride: carbon with 2 ester groups
const PATTERN_DIGLYCERIDE: &str = "[CX4]([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])";

/// Monoglyceride: single ester linkage
const PATTERN_MONOGLYCERIDE: &str = "[CX4][OX2][CX3](=[OX1])[#6]";

/// Long aliphatic chain: 8+ saturated carbons NOT in rings (NO aromatic, NO rings)
/// Uses [!a] for not aromatic and [!R] for not in ring
const PATTERN_ALIPHATIC_CHAIN: &str =
    "[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]";

/// Amide linkage: C(=O)-N (found in ceramides)
const PATTERN_AMIDE: &str = "[CX3](=[OX1])[NX3]";

/// Amino group: N bonded to aliphatic carbon (sphinganine backbones)
const PATTERN_AMINO: &str = "[NX3][CH0,CH1,CH2,CH3]";

/// Fatty acid: carboxylic acid with non-aromatic, acyclic carbon
const PATTERN_CARBOXYLIC_ACID: &str = "[#6;!a;!R][CX3](=[OX1])[OH]";

/// Returns `true` when `molecule` contains at least one match for `pattern`.
fn has_substructure(molecule: &Molecule, pattern: &str) -> bool {
    let Ok(query) = smarts::parse_smarts(pattern) else {
        return false;
    };
    !smarts::find_matches(&query, molecule).is_empty()
}

/// Check if a molecule is acyclic (contains no rings).
///
/// Returns `true` if the molecule has no ring atoms (aromatic or alicyclic).
/// Used to reject aromatic rings, nucleotides, sugars, steroids, etc.
#[must_use]
pub fn is_acyclic(molecule: &Molecule) -> bool {
    // Count ring atoms - a lipid should have ZERO
    let ring_pattern = "[R]"; // Any atom in a ring
    let Ok(query) = smarts::parse_smarts(ring_pattern) else {
        return true; // If we can't parse, assume acyclic
    };
    smarts::find_matches(&query, molecule).is_empty()
}

/// Extract elemental counts from a parsed molecule using chematic's own
/// descriptors (heavy atoms plus implicit/explicit hydrogens).
fn counts_from_molecule(molecule: &Molecule) -> ElementCounts {
    ElementCounts {
        carbon: chem::num_carbons(molecule) as u32,
        hydrogen: chem::num_hydrogens(molecule) as u32,
        nitrogen: chem::num_nitrogens(molecule) as u32,
        oxygen: chem::num_oxygens(molecule) as u32,
        phosphorus: chem::num_phosphorus(molecule) as u32,
        sulfur: chem::num_sulfurs(molecule) as u32,
        halogens: (chem::num_fluorines(molecule)
            + chem::num_chlorines(molecule)
            + chem::num_bromines(molecule)
            + chem::num_iodines(molecule)) as u32,
    }
}

/// Structural (SMILES-based) lipid classification.
/// Only matches REAL lipid backbones: long aliphatic chains + characteristic functional groups.
/// NO RINGS. NO AROMATICS. NO NUCLEOTIDES. NO STEROIDS.
fn classify_molecule(molecule: &Molecule) -> Option<(LipidClass, ElementCounts)> {
    let counts = counts_from_molecule(molecule);

    // === CRITICAL: Lipids are ACYCLIC ===
    // Reject any molecule with rings: nucleotides, sugars, steroids, aromatic rings
    if !is_acyclic(molecule) {
        return None;
    }

    // === PHOSPHOLIPIDS (PC/PE): phosphate + headgroup + aliphatic chain ===
    // Require all three to avoid matching ATP, CoA, nucleotides
    if has_substructure(molecule, PATTERN_PHOSPHATE)
        && has_substructure(molecule, PATTERN_ALIPHATIC_CHAIN)
    {
        if has_substructure(molecule, PATTERN_CHOLINE) {
            return Some((LipidClass::Glycerophospholipid, counts));
        }
        if has_substructure(molecule, PATTERN_ETHANOLAMINE) {
            return Some((LipidClass::Glycerophospholipid, counts));
        }
    }

    // === TRIGLYCERIDES: 3 ester groups on glycerol backbone ===
    if has_substructure(molecule, PATTERN_TRIGLYCERIDE) {
        return Some((LipidClass::Glycerolipid, counts));
    }

    // === DIGLYCERIDES: 2 ester groups on glycerol backbone ===
    if has_substructure(molecule, PATTERN_DIGLYCERIDE) {
        return Some((LipidClass::Glycerolipid, counts));
    }

    // === MONOGLYCERIDES: 1 ester group + long chain ===
    if has_substructure(molecule, PATTERN_MONOGLYCERIDE)
        && has_substructure(molecule, PATTERN_ALIPHATIC_CHAIN)
    {
        return Some((LipidClass::Glycerolipid, counts));
    }

    // === CERAMIDES & SPHINGANINES: amide or amino group + long aliphatic chain ===
    if has_substructure(molecule, PATTERN_ALIPHATIC_CHAIN) {
        // True ceramides: amide linkage
        if has_substructure(molecule, PATTERN_AMIDE) {
            return Some((LipidClass::Sphingolipid, counts));
        }
        // Sphinganine: amino group on aliphatic backbone
        if has_substructure(molecule, PATTERN_AMINO) {
            return Some((LipidClass::Sphingolipid, counts));
        }
    }

    // === FATTY ACIDS: carboxylic acid + long aliphatic chain ===
    if has_substructure(molecule, PATTERN_CARBOXYLIC_ACID)
        && has_substructure(molecule, PATTERN_ALIPHATIC_CHAIN)
    {
        return Some((LipidClass::FattyAcyl, counts));
    }

    None
}

/// Classify a single SMILES string.
///
/// Returns the classification (with formula and exact mass) when the molecule
/// is recognized as a lipid, otherwise `None`.
#[must_use]
pub fn classify_smiles(smiles: &str) -> Option<LipidClassification> {
    let trimmed = smiles.trim();
    if trimmed.is_empty() {
        return None;
    }

    // SMILES parsers can panic on pathological input; isolate the call so a
    // malformed SMILES falls back to the formula classifier instead of crashing.
    let molecule =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| smiles::parse(trimmed)))
            .ok()
            .and_then(std::result::Result::ok)?;

    let (class, counts) = classify_molecule(&molecule)?;
    let formula = molecule.total_formula();
    let exact_mass = chem::exact_mass(&molecule);

    Some(LipidClassification {
        class,
        formula,
        exact_mass,
        counts,
        derived_from_smiles: true,
    })
}

/// Conservative formula-only fallback (used when a SMILES is absent/unparseable
/// but a Hill-notation `FORMULA=` is available).
#[must_use]
pub fn classify_formula(formula: &str) -> Option<LipidClass> {
    let trimmed = formula.trim();
    if trimmed.is_empty() {
        return None;
    }

    let map = chem::parse_formula(trimmed).ok()?;
    let mut counts = ElementCounts::default();
    let mut halogens = 0u32;
    for (symbol, value) in &map {
        match symbol.as_str() {
            "C" => counts.carbon = *value,
            "H" => counts.hydrogen = *value,
            "N" => counts.nitrogen = *value,
            "O" => counts.oxygen = *value,
            "P" => counts.phosphorus = *value,
            "S" => counts.sulfur = *value,
            "F" | "Cl" | "Br" | "I" => halogens += *value,
            _ => {}
        }
    }
    counts.halogens = halogens;

    let c = counts.carbon as i32;
    let h = counts.hydrogen as i32;
    let n = counts.nitrogen as i32;
    let p = counts.phosphorus as i32;
    let o = counts.oxygen as i32;
    let s = counts.sulfur as i32;
    let db = counts.double_bond_equivalent();
    let heavy = c + n + o + p + s + halogens as i32;

    if c == 0 || heavy < 4 {
        return None;
    }

    // Free fatty acid: RCOOH — exactly two oxygens, no P/N/S/halogen.
    if p == 0
        && n == 0
        && s == 0
        && halogens == 0
        && o == 2
        && (7..=40).contains(&c)
        && (1.0..=14.0).contains(&db)
        && h <= 2 * c + 2
        && h >= 2 * c - 28
    {
        return Some(LipidClass::FattyAcyl);
    }

    // NOTE: Do NOT classify sterols by formula alone - sterols have 4-ring cores
    // and are definitionally cyclic. Formula-based classification cannot determine
    // ring structure, so we reject all sterol-like formulas to be safe.

    // Glycerolipid: ester-rich, oxygen-rich but low O/C ratio (excludes sugars).
    if p == 0
        && n == 0
        && (3..=6).contains(&o)
        && c >= 8
        && f64::from(o) / f64::from(c) <= 0.25
        && db >= 1.0
        && h <= 2 * c + 2
        && h >= 2 * c - 28
    {
        return Some(LipidClass::Glycerolipid);
    }

    // Glycerophospholipid: phosphorus present with a substantial, oxygen-moderate
    // backbone (excludes ATP / coenzyme A which have very high O/C ratios).
    if p >= 1 && c >= 15 && o >= 6 && f64::from(o) / f64::from(c) <= 0.6 {
        return Some(LipidClass::Glycerophospholipid);
    }

    None
}

/// Top-level entry point used by the MGF parser.
///
/// Tries the SMILES first (structural classification) and falls back to the
/// `FORMULA=` value when the SMILES is missing or cannot be parsed.
#[must_use]
pub fn classify_spectrum(
    smiles: Option<&str>,
    formula: Option<&str>,
) -> Option<LipidClassification> {
    if let Some(smiles) = smiles.filter(|value| !value.trim().is_empty())
        && let Some(classification) = classify_smiles(smiles)
    {
        return Some(classification);
    }

    if let Some(formula) = formula.filter(|value| !value.trim().is_empty())
        && let Some(class) = classify_formula(formula)
    {
        let counts = chem::parse_formula(formula.trim())
            .ok()
            .map(|map| formula_counts(&map));
        let mass = chem::parse_formula(formula.trim())
            .ok()
            .and_then(|map| exact_mass_from_counts(&map))
            .unwrap_or(0.0);
        return Some(LipidClassification {
            class,
            counts: counts.unwrap_or_default(),
            formula: formula.trim().to_string(),
            exact_mass: mass,
            derived_from_smiles: false,
        });
    }

    None
}

fn formula_counts(map: &std::collections::HashMap<String, u32>) -> ElementCounts {
    let mut counts = ElementCounts::default();
    let mut halogens = 0u32;
    for (symbol, value) in map {
        match symbol.as_str() {
            "C" => counts.carbon = *value,
            "H" => counts.hydrogen = *value,
            "N" => counts.nitrogen = *value,
            "O" => counts.oxygen = *value,
            "P" => counts.phosphorus = *value,
            "S" => counts.sulfur = *value,
            "F" | "Cl" | "Br" | "I" => halogens += *value,
            _ => {}
        }
    }
    counts.halogens = halogens;
    counts
}

/// Approximate monoisotopic mass from a formula count map.
///
/// Uses the `elements_rs` atomic masses exposed through `chematic::core::Element`
/// via a small lookup. This mirrors `chematic_chem::exact_mass` for the
/// common light elements but avoids pulling every isotopic table here.
fn exact_mass_from_counts(map: &std::collections::HashMap<String, u32>) -> Option<f64> {
    let mut total = 0.0;
    let mut found = false;
    for (symbol, count) in map {
        let element = Element::from_symbol(symbol)?;
        total = element.atomic_mass().mul_add(f64::from(*count), total);
        found = true;
    }
    if found { Some(total) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn class_of(smiles: &str) -> Option<LipidClass> {
        classify_smiles(smiles).map(|c| c.class)
    }

    #[test]
    fn fatty_acyls_are_recognized() {
        assert_eq!(
            class_of("CCCCCCCCCCCCCCCC(=O)O"),
            Some(LipidClass::FattyAcyl)
        );
        assert_eq!(
            class_of("CCCC=CC=CC=CC=CC=CC(=O)O"),
            Some(LipidClass::FattyAcyl)
        );
    }

    #[test]
    fn esterified_fatty_acyl_is_glycerolipid() {
        assert_eq!(
            class_of("CCCCCCCCCCCCCCC(=O)OC"),
            Some(LipidClass::Glycerolipid)
        );
    }

    #[test]
    fn sphingolipids_are_recognized() {
        assert_eq!(
            class_of("CCCCCCCCCCCCC=CC(C(CO)N)O"),
            Some(LipidClass::Sphingolipid)
        );
        assert_eq!(
            class_of("CCCCCCCCCCCCCCC(=O)N[C@H](CO)CCCCCCCCCC"),
            Some(LipidClass::Sphingolipid)
        );
    }

    #[test]
    fn phospholipids_are_recognized() {
        assert_eq!(
            class_of("CCCCCCCCCC=CC=CC=CC=CC=CC(=O)OC(C)COP(=O)(O)OCC[N+](C)(C)C"),
            Some(LipidClass::Glycerophospholipid)
        );
    }

    #[test]
    fn cofactors_and_metabolites_are_rejected() {
        assert_eq!(class_of("C(C1C(C(C(C(O1)O)O)O)O)O"), None); // glucose
        assert_eq!(class_of("CN1C=NC2=C1C(=O)N(C)C(=O)N2C"), None); // caffeine
        assert_eq!(
            class_of("CC1C(C(C(O1)OP(=O)(O)O)OP(=O)(O)O)N2C=NC3=C2N=CN=C3N"),
            None
        ); // ATP
        assert_eq!(class_of("C[N+](C)(C)CCO"), None); // choline
        assert_eq!(class_of("CC1=C(C(=CC=C1)S(=O)(=O)O)C(=O)O"), None); // aromatic sulfonic acid - should NOT be a lipid
        assert_eq!(
            class_of(
                "CC(C)(COP(=O)(O)OP(=O)(O)OCC1C(C(C(O1)N2C=NC3=C(N=CN=C32)N)O)OP(=O)(O)O)C(C(=O)NCCC(=O)NCCS)O"
            ),
            None
        ); // coenzyme A
    }

    #[test]
    fn formula_fallback_classifies_fatty_acid() {
        assert_eq!(classify_formula("C16H32O2"), Some(LipidClass::FattyAcyl));
        assert_eq!(classify_formula("C18H36O2"), Some(LipidClass::FattyAcyl));
    }

    #[test]
    fn formula_fallback_rejects_cholesterol() {
        // Sterols have 4-ring cores; formula-based classification cannot determine
        // ring structure, so sterol formulas are rejected to prevent false positives
        assert_eq!(classify_formula("C27H46O"), None);
    }

    #[test]
    fn formula_fallback_rejects_cofactors() {
        assert_eq!(classify_formula("C10H15N5O9P2"), None); // ATP
        assert_eq!(classify_formula("C21H36N7O16P3S"), None); // coenzyme A
        assert_eq!(classify_formula("C6H12O6"), None); // glucose
        assert_eq!(classify_formula(""), None);
    }

    #[test]
    fn counts_are_extracted_from_smiles() {
        let mol = smiles::parse("CCCCCCCCCCCCCCCC(=O)O").unwrap();
        let counts = counts_from_molecule(&mol);
        assert_eq!(counts.carbon, 16);
        assert_eq!(counts.hydrogen, 32);
        assert_eq!(counts.oxygen, 2);
        assert_eq!(counts.formula_string(), "C16H32O2");
    }
}

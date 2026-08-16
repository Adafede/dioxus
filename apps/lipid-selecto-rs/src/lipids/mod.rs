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

mod classify;
mod types;

pub use classify::{classify_formula, classify_smiles, classify_spectrum, is_acyclic};
pub use types::{ElementCounts, LipidClass, LipidClassification};

// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Top-level CX-SMILES generation pipeline orchestrator.

use chematic::core::Molecule;
use chematic::smiles::canonical_smiles;

use crate::error::CxError;
use crate::parse::{cluster, parse_list};
use crate::positional::build_positional_with_options;
use crate::repeating::build_repeating;
use crate::roundtrip::evaluate_construct;
use crate::types::{Confidence, ConstructKind, Coverage, CxOptions, CxResult};

const CLUSTER_TANIMOTO: f64 = 0.3;

/// Generate a CX-SMILES representation from a SMILES list using default options.
///
/// # Errors
///
/// Returns [`CxError`] if the input is empty, contains invalid SMILES syntax, or
/// exceeds safety bounds during enumeration.
pub fn generate(smiles: &[String]) -> Result<CxResult, CxError> {
    generate_with_options(smiles, CxOptions::default())
}

/// Generate a CX-SMILES representation from a SMILES list with configurable options.
///
/// # Errors
///
/// Returns [`CxError`] if the input is empty, contains invalid SMILES syntax, or
/// exceeds safety bounds during enumeration.
pub fn generate_with_options(smiles: &[String], options: CxOptions) -> Result<CxResult, CxError> {
    let mols = parse_list(smiles)?;
    if mols.is_empty() {
        return Err(CxError::EmptyInput);
    }
    if let [single] = mols.as_slice() {
        return Ok(singleton_result(single));
    }

    let group: Vec<Molecule> = cluster(&mols, CLUSTER_TANIMOTO)
        .into_iter()
        .max_by_key(Vec::len)
        .filter(|c| c.len() >= 2)
        .unwrap_or_else(|| mols.clone());

    let mut res = build_repeating(&group)
        .or_else(|_| build_positional_with_options(&group, options.expand_aromatic))?;

    let (construct, confidence) = evaluate_construct(&res.enumerated, &mols, res.construct);
    res.construct = construct;
    res.confidence = confidence;
    Ok(res)
}

fn singleton_result(mol: &Molecule) -> CxResult {
    let smi = canonical_smiles(mol);
    CxResult {
        cx_smiles: smi.clone(),
        construct: ConstructKind::BestEffort,
        scaffold_smiles: smi.clone(),
        floating: Vec::new(),
        repeat_unit: None,
        confidence: Confidence::from_coverage(Coverage {
            covered: 1,
            total: 1,
        }),
        enumerated: vec![smi],
    }
}

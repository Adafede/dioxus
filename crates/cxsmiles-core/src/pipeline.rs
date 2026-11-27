// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Top-level CX-SMILES generation pipeline orchestrator.

use std::borrow::Cow;

use chematic::core::Molecule;
use chematic::smiles::canonical_smiles;

use crate::error::CxError;
use crate::parse::{cluster_indices, parse_list};
use crate::positional::build_positional_with_canons;
use crate::repeating::build_repeating_with_canons;
use crate::roundtrip::evaluate_construct_canons;
use crate::types::{ConstructKind, Coverage, CxOptions, CxResult};

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
    let mut pairs: Vec<(Molecule, String)> = mols
        .into_iter()
        .map(|m| {
            let smi = canonical_smiles(&m);
            (m, smi)
        })
        .collect();
    pairs.sort_by(|(ma, sa), (mb, sb)| (ma.atom_count(), sa).cmp(&(mb.atom_count(), sb)));
    if let [(_, single_smiles)] = pairs.as_slice() {
        return Ok(singleton_result(single_smiles));
    }

    let (canonical_mols, canonical_strs): (Vec<Molecule>, Vec<String>) = pairs.into_iter().unzip();
    let best_indices = cluster_indices(&canonical_mols, CLUSTER_TANIMOTO)
        .into_iter()
        .max_by(|a, b| {
            a.len().cmp(&b.len()).then_with(|| {
                let sa = a.first().and_then(|&idx| canonical_strs.get(idx));
                let sb = b.first().and_then(|&idx| canonical_strs.get(idx));
                sb.cmp(&sa)
            })
        })
        .filter(|c| c.len() >= 2);

    let (group, group_canons): (Cow<'_, [Molecule]>, Cow<'_, [String]>) = best_indices.map_or_else(
        || {
            (
                Cow::Borrowed(canonical_mols.as_slice()),
                Cow::Borrowed(canonical_strs.as_slice()),
            )
        },
        |indices| {
            let sub_mols = indices
                .iter()
                .filter_map(|&i| canonical_mols.get(i).cloned())
                .collect();
            let sub_canons = indices
                .iter()
                .filter_map(|&i| canonical_strs.get(i).cloned())
                .collect();
            (Cow::Owned(sub_mols), Cow::Owned(sub_canons))
        },
    );

    let mut res = match build_repeating_with_canons(&group, &group_canons) {
        Ok(rep) if rep.coverage.is_clean() => rep,
        Ok(rep) => {
            match build_positional_with_canons(&group, &group_canons, options.expand_aromatic) {
                Ok(pos) if pos.coverage.covered > rep.coverage.covered => pos,
                _ => rep,
            }
        }
        Err(_) => build_positional_with_canons(&group, &group_canons, options.expand_aromatic)?,
    };

    let (construct, coverage) =
        evaluate_construct_canons(&res.enumerated, &canonical_strs, res.construct);
    res.construct = construct;
    res.coverage = coverage;
    Ok(res)
}

fn singleton_result(smi: &str) -> CxResult {
    let owned = smi.to_string();
    CxResult {
        cx_smiles: owned.clone(),
        construct: ConstructKind::BestEffort,
        scaffold_smiles: owned.clone(),
        floating: Vec::new(),
        repeat_unit: None,
        coverage: Coverage {
            covered: 1,
            total: 1,
        },
        enumerated: vec![owned],
    }
}

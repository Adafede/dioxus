// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Input parsing and single-linkage clustering.
//!
//! `parse_list` turns raw SMILES strings into `Molecule`s; `cluster` groups
//! them by ECFP4/Tanimoto similarity so unrelated structures are never forced
//! into one nonsensical CX-SMILES.

use chematic::core::Molecule;
use chematic::fp::{BitVec2048, ecfp4};
use chematic::smiles::parse;

use super::types::CxError;

/// Parse a list of raw SMILES strings into molecules, skipping blanks.
pub fn parse_list(smiles: &[String]) -> Result<Vec<Molecule>, CxError> {
    let mut mols = Vec::new();
    for s in smiles {
        let t = s.trim();
        if t.is_empty() {
            continue;
        }
        match parse(t) {
            Ok(m) => mols.push(m),
            Err(e) => return Err(CxError(format!("could not parse '{t}': {e:?}"))),
        }
    }
    Ok(mols)
}

/// Single-linkage clustering by ECFP4/Tanimoto above `threshold`.
pub fn cluster(mols: &[Molecule], threshold: f64) -> Vec<Vec<Molecule>> {
    let n = mols.len();
    if n == 0 {
        return Vec::new();
    }
    let fps: Vec<BitVec2048> = mols.iter().map(ecfp4).collect();
    let mut parent: Vec<usize> = (0..n).collect();
    let find = |p: &mut [usize], x: usize| -> usize {
        let mut r = x;
        while p[r] != r {
            r = p[r];
        }
        let mut x = x;
        while p[x] != r {
            let next = p[x];
            p[x] = r;
            x = next;
        }
        r
    };
    for i in 0..n {
        for j in (i + 1)..n {
            if fps[i].tanimoto(&fps[j]) >= threshold {
                let (ri, rj) = (find(&mut parent, i), find(&mut parent, j));
                if ri != rj {
                    parent[ri] = rj;
                }
            }
        }
    }
    let roots: Vec<usize> = (0..n).map(|i| find(&mut parent, i)).collect();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&i| roots[i]);
    let mut buckets: Vec<Vec<Molecule>> = Vec::new();
    let mut cur = roots[order[0]];
    let mut bucket: Vec<Molecule> = vec![mols[order[0]].clone()];
    for &i in &order[1..] {
        if roots[i] == cur {
            bucket.push(mols[i].clone());
        } else {
            buckets.push(bucket);
            cur = roots[i];
            bucket = vec![mols[i].clone()];
        }
    }
    buckets.push(bucket);
    buckets
}

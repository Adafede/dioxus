// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Input parsing and single-linkage ECFP4/Tanimoto clustering.

use chematic::core::Molecule;
use chematic::fp::{BitVec2048, ecfp4};
use chematic::smiles::parse;

use crate::error::CxError;

/// Parse non-blank SMILES lines into molecules, failing on the first invalid line.
pub(crate) fn parse_list(smiles: &[String]) -> Result<Vec<Molecule>, CxError> {
    let mut mols = Vec::new();
    for s in smiles {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mol = parse(trimmed).map_err(|err| CxError::ParseFailed {
            smiles: trimmed.to_string(),
            detail: format!("{err:?}"),
        })?;
        mols.push(mol);
    }
    Ok(mols)
}

/// Single-linkage clustering returning index buckets into `mols`.
#[expect(
    clippy::indexing_slicing,
    reason = "n == 0 returns early; every index is in 0..n on vecs of length n"
)]
pub(crate) fn cluster_indices(mols: &[Molecule], threshold: f64) -> Vec<Vec<usize>> {
    let n = mols.len();
    if n == 0 {
        return Vec::new();
    }
    let fps: Vec<BitVec2048> = mols.iter().map(ecfp4).collect();
    let mut parent: Vec<usize> = (0..n).collect();
    for i in 0..n {
        for j in i.saturating_add(1)..n {
            if fps[i].tanimoto(&fps[j]) >= threshold {
                let (ri, rj) = (find_root(&mut parent, i), find_root(&mut parent, j));
                if ri != rj {
                    parent[ri] = rj;
                }
            }
        }
    }
    let roots: Vec<usize> = (0..n).map(|i| find_root(&mut parent, i)).collect();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&i| roots[i]);

    let mut buckets: Vec<Vec<usize>> = Vec::new();
    let mut cur = roots[order[0]];
    let mut bucket = vec![order[0]];
    for &i in &order[1..] {
        if roots[i] == cur {
            bucket.push(i);
        } else {
            buckets.push(bucket);
            cur = roots[i];
            bucket = vec![i];
        }
    }
    buckets.push(bucket);
    buckets
}

/// Single-linkage clustering by ECFP4/Tanimoto similarity above `threshold`.
#[cfg(test)]
pub(crate) fn cluster(mols: &[Molecule], threshold: f64) -> Vec<Vec<Molecule>> {
    cluster_indices(mols, threshold)
        .into_iter()
        .map(|bucket| {
            bucket
                .into_iter()
                .filter_map(|i| mols.get(i).cloned())
                .collect()
        })
        .collect()
}

/// Union-find path-compressing root lookup bounded by `p.len()`.
#[expect(
    clippy::indexing_slicing,
    reason = "x < p.len() and parent values stay within 0..p.len()"
)]
pub(crate) fn find_root(p: &mut [usize], x: usize) -> usize {
    let mut r = x;
    for _ in 0..p.len() {
        if p[r] == r {
            break;
        }
        r = p[r];
    }
    let mut cur = x;
    for _ in 0..p.len() {
        if p[cur] == r {
            break;
        }
        let next = p[cur];
        p[cur] = r;
        cur = next;
    }
    r
}

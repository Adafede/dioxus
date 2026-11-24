// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Repeating-unit CX construction (`Sg:n:`) using zero-clone backtracking DFS.

use chematic::core::{AtomIdx, BondOrder, Molecule};
use chematic::smarts::{QueryMolecule, find_matches};
use chematic::smiles::{parse, write};
use rustc_hash::FxHashMap;

use crate::error::CxError;
use crate::graph::{components, matched_mask, molecule_to_query, multiset_into, unmatched_atoms};
use crate::roundtrip::{enumerate_repeating, evaluate_construct};
use crate::types::{ConstructKind, CxResult, RepeatUnit};

const MAX_FRAGMENTS: usize = 200_000;

/// Build an `Sg:n:` repeating CX-SMILES from an atom-count homologous series.
pub(crate) fn build_repeating(group: &[Molecule]) -> Result<CxResult, CxError> {
    let mut ordered = group.to_vec();
    ordered.sort_by_key(Molecule::atom_count);
    let (Some(shortest), Some(longest)) = (ordered.first(), ordered.last()) else {
        return Err(CxError::EmptyGroup("build_repeating"));
    };
    if shortest.atom_count() == longest.atom_count() {
        return Err(CxError::UniformAtomCounts);
    }
    let scaffold_smiles = write(shortest);
    let scaffold =
        parse(&scaffold_smiles).map_err(|e| CxError::ScaffoldReparse(format!("{e:?}")))?;
    let unit_size = compute_unit_size(&ordered, shortest.atom_count());
    let scaffold_q = molecule_to_query(&scaffold);
    let pattern = repeat_pattern(shortest, longest, unit_size, &scaffold_q)?;
    let unit_ms = unit_multiset(&pattern, longest, unit_size);
    let repeat_atoms = locate_repeat_in_scaffold(&scaffold, &unit_ms, unit_size)?;
    let nonrepeat = scaffold.atom_count() - repeat_atoms.len();
    let count_max = (longest.atom_count() - nonrepeat) / unit_size;
    let repeat_unit = RepeatUnit {
        atoms: repeat_atoms,
        min: 1,
        max: count_max,
    };
    let cx_smiles = format_repeat_cx(&scaffold_smiles, &repeat_unit.atoms);
    let enumerated = enumerate_repeating(&scaffold, &repeat_unit)?;
    let (construct, coverage) = evaluate_construct(&enumerated, group, ConstructKind::Repeating);
    if coverage.covered < 2 {
        return Err(CxError::NoHomologueReproduced);
    }
    Ok(CxResult {
        cx_smiles,
        construct,
        scaffold_smiles,
        floating: Vec::new(),
        repeat_unit: Some(repeat_unit),
        coverage,
        enumerated,
    })
}

fn compute_unit_size(ordered: &[Molecule], shortest_atoms: usize) -> usize {
    let deltas: Vec<usize> = ordered
        .iter()
        .skip(1)
        .map(|m| m.atom_count() - shortest_atoms)
        .collect();
    gcd_vec(&deltas).max(1)
}

fn format_repeat_cx(scaffold_smiles: &str, atoms: &[usize]) -> String {
    let atoms_field = atoms
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("{scaffold_smiles} |Sg:n:{atoms_field}:n:ht|")
}

/// Extra component from aligning `scaffold_q` into `longest` that represents repeats.
pub(crate) fn repeat_pattern(
    _shortest: &Molecule,
    longest: &Molecule,
    unit_size: usize,
    scaffold_q: &QueryMolecule,
) -> Result<Vec<u32>, CxError> {
    let hit = find_matches(scaffold_q, longest)
        .into_iter()
        .next()
        .ok_or(CxError::NoRepeatScaffoldMatch)?;
    let matched = matched_mask(&hit, longest);
    let comps = components(&unmatched_atoms(&matched), longest);
    comps
        .iter()
        .filter(|c| c.len() % unit_size == 0 && c.len() >= unit_size)
        .min_by_key(|c| c.len())
        .or_else(|| comps.iter().min_by_key(|c| c.len()))
        .cloned()
        .ok_or(CxError::NoRepeatPattern)
}

/// Sorted atomic-number multiset of one repeat unit derived from `pattern`.
pub(crate) fn unit_multiset(pattern: &[u32], longest: &Molecule, unit_size: usize) -> Vec<u8> {
    let k = pattern.len().saturating_div(unit_size).max(1);
    let mut counts: FxHashMap<u8, usize> = FxHashMap::default();
    for &i in pattern {
        let el = longest.atom(AtomIdx(i)).element.atomic_number();
        *counts.entry(el).or_insert(0) += 1;
    }
    let mut v: Vec<u8> = counts
        .iter()
        .flat_map(|(el, &c)| std::iter::repeat_n(*el, c / k))
        .collect();
    v.sort_unstable();
    v
}

struct SearchCtx<'a> {
    scaffold: &'a Molecule,
    target: &'a [u8],
    unit_size: usize,
    atom_count: usize,
}

struct SearchState {
    budget: usize,
    visited: Vec<bool>,
    frag: Vec<u32>,
    sorted: Vec<u32>,
    multiset: Vec<u8>,
    best: Option<Vec<u32>>,
}

/// Locate the centered internal connected subgraph matching `target` via backtracking DFS.
pub(crate) fn locate_repeat_in_scaffold(
    scaffold: &Molecule,
    target: &[u8],
    unit_size: usize,
) -> Result<Vec<usize>, CxError> {
    let n = scaffold.atom_count();
    if unit_size == 0 || unit_size > n {
        return Err(CxError::RepeatTooLarge {
            unit_size,
            scaffold_atoms: n,
        });
    }
    let ctx = SearchCtx {
        scaffold,
        target,
        unit_size,
        atom_count: n,
    };
    let mut state = SearchState {
        budget: MAX_FRAGMENTS,
        visited: vec![false; n],
        frag: Vec::with_capacity(unit_size),
        sorted: Vec::with_capacity(unit_size),
        multiset: Vec::with_capacity(unit_size),
        best: None,
    };
    let n_u32 = u32::try_from(n).unwrap_or(u32::MAX);
    for start in 0..n_u32 {
        let su = usize::try_from(start).unwrap_or(usize::MAX);
        state.frag.push(start);
        if let Some(slot) = state.visited.get_mut(su) {
            *slot = true;
        }
        dfs_repeat(&ctx, &mut state, start)?;
        state.frag.pop();
        if let Some(slot) = state.visited.get_mut(su) {
            *slot = false;
        }
    }
    state
        .best
        .map(|v| {
            v.into_iter()
                .map(|a| usize::try_from(a).unwrap_or(usize::MAX))
                .collect()
        })
        .ok_or(CxError::RepeatNotFound)
}

fn dfs_repeat(ctx: &SearchCtx<'_>, state: &mut SearchState, cur: u32) -> Result<(), CxError> {
    state.budget = state.budget.saturating_sub(1);
    if state.budget == 0 {
        return Err(CxError::RepeatBudgetExceeded {
            unit_size: ctx.unit_size,
            scaffold_atoms: ctx.atom_count,
            limit: MAX_FRAGMENTS,
        });
    }
    if state.frag.len() == ctx.unit_size {
        evaluate_candidate(ctx, state);
        return Ok(());
    }
    for (nbr, _) in ctx.scaffold.neighbors(AtomIdx(cur)) {
        let nu = usize::try_from(nbr.0).unwrap_or(usize::MAX);
        if state.visited.get(nu).copied() != Some(false) {
            continue;
        }
        if let Some(slot) = state.visited.get_mut(nu) {
            *slot = true;
        }
        state.frag.push(nbr.0);
        dfs_repeat(ctx, state, nbr.0)?;
        state.frag.pop();
        if let Some(slot) = state.visited.get_mut(nu) {
            *slot = false;
        }
    }
    Ok(())
}

fn evaluate_candidate(ctx: &SearchCtx<'_>, state: &mut SearchState) {
    if !is_internal(&state.frag, ctx.scaffold) {
        return;
    }
    multiset_into(&state.frag, ctx.scaffold, &mut state.multiset);
    if state.multiset.as_slice() != ctx.target {
        return;
    }
    state.sorted.clear();
    state.sorted.extend_from_slice(&state.frag);
    state.sorted.sort_unstable();
    let is_better = state.best.as_ref().is_none_or(|b| {
        center_dist(&state.sorted, ctx.atom_count) < center_dist(b, ctx.atom_count)
    });
    if is_better {
        state.best = Some(state.sorted.clone());
    }
}

pub(crate) fn is_internal(atoms: &[u32], mol: &Molecule) -> bool {
    let mut ext = 0usize;
    for &a in atoms {
        for (nbr, bond_idx) in mol.neighbors(AtomIdx(a)) {
            if !atoms.contains(&nbr.0) && nbr.0 != a {
                if mol.bond(bond_idx).order != BondOrder::Single {
                    return false;
                }
                ext += 1;
            }
        }
    }
    ext == 2
}

pub(crate) fn center_dist(atoms: &[u32], n: usize) -> i64 {
    let center = (i64::try_from(n).unwrap_or(i64::MAX) - 1) / 2;
    atoms
        .iter()
        .map(|&a| i64::from(a) - center)
        .sum::<i64>()
        .abs()
}

pub(crate) fn gcd_vec(xs: &[usize]) -> usize {
    xs.iter().copied().fold(0, gcd)
}

pub(crate) const fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

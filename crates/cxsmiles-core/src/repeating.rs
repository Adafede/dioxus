// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Repeating-unit CX construction (`Sg:n:`) using zero-clone backtracking DFS.

use std::fmt::Write as _;

use chematic::core::{AtomIdx, BondOrder, Molecule};
use chematic::smarts::{QueryMolecule, find_matches};
#[cfg(test)]
use chematic::smiles::canonical_smiles;
use chematic::smiles::{parse, write};

use crate::error::CxError;
use crate::graph::{
    components, matched_mask, molecule_to_query, multiset_into, u32_to_usize, unmatched_atoms,
    usize_to_u32,
};
use crate::roundtrip::{dedup_sort, enumerate_repeating, evaluate_construct_canons};
use crate::types::{ConstructKind, CxResult, RepeatUnit};

const MAX_FRAGMENTS: usize = 200_000;

/// Build an `Sg:n:` repeating CX-SMILES from an atom-count homologous series.
#[cfg(test)]
pub(crate) fn build_repeating(group: &[Molecule]) -> Result<CxResult, CxError> {
    let group_canons: Vec<String> = group.iter().map(canonical_smiles).collect();
    build_repeating_with_canons(group, &group_canons)
}

pub(crate) fn build_repeating_with_canons(
    group: &[Molecule],
    group_canons: &[String],
) -> Result<CxResult, CxError> {
    let mut ordered: Vec<&Molecule> = group.iter().collect();
    ordered.sort_by_key(|m| m.atom_count());
    let (Some(&shortest), Some(&longest)) = (ordered.first(), ordered.last()) else {
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
    let pattern = repeat_pattern(longest, unit_size, &scaffold_q)?;
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
    let (construct, coverage) =
        evaluate_construct_canons(&enumerated, group_canons, ConstructKind::Repeating);
    let distinct_reproduced = dedup_sort(
        group_canons
            .iter()
            .filter(|smi| enumerated.binary_search(smi).is_ok())
            .cloned()
            .collect(),
    );
    if distinct_reproduced.len() < 2 {
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

fn compute_unit_size(ordered: &[&Molecule], shortest_atoms: usize) -> usize {
    ordered
        .iter()
        .skip(1)
        .fold(0usize, |acc, m| gcd(acc, m.atom_count() - shortest_atoms))
        .max(1)
}

fn format_repeat_cx(scaffold_smiles: &str, atoms: &[usize]) -> String {
    let mut out = String::with_capacity(scaffold_smiles.len() + 16 + atoms.len() * 3);
    out.push_str(scaffold_smiles);
    out.push_str(" |Sg:n:");
    for (idx, &atom) in atoms.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        let _ = write!(out, "{atom}");
    }
    out.push_str(":n:ht|");
    out
}

/// Extra component from aligning `scaffold_q` into `longest` that represents repeats.
pub(crate) fn repeat_pattern(
    longest: &Molecule,
    unit_size: usize,
    scaffold_q: &QueryMolecule,
) -> Result<Vec<u32>, CxError> {
    let hit = find_matches(scaffold_q, longest)
        .into_iter()
        .next()
        .ok_or(CxError::NoRepeatScaffoldMatch)?;
    let matched = matched_mask(&hit, longest);
    let mut comps = components(&unmatched_atoms(&matched), longest);
    let best_idx = (0..comps.len())
        .filter(|&i| {
            comps
                .get(i)
                .is_some_and(|c| c.len() % unit_size == 0 && c.len() >= unit_size)
        })
        .min_by_key(|&i| comps.get(i).map_or(usize::MAX, Vec::len))
        .or_else(|| (0..comps.len()).min_by_key(|&i| comps.get(i).map_or(usize::MAX, Vec::len)))
        .ok_or(CxError::NoRepeatPattern)?;
    Ok(comps.swap_remove(best_idx))
}

/// Sorted atomic-number multiset of one repeat unit derived from `pattern`.
pub(crate) fn unit_multiset(pattern: &[u32], longest: &Molecule, unit_size: usize) -> Vec<u8> {
    let k = pattern.len().saturating_div(unit_size).max(1);
    let mut counts = [0usize; 128];
    for &i in pattern {
        let el = longest.atom(AtomIdx(i)).element.atomic_number();
        if let Some(slot) = counts.get_mut(usize::from(el)) {
            *slot += 1;
        }
    }
    let mut v = Vec::new();
    for (el, &c) in counts.iter().enumerate() {
        let reps = c / k;
        if reps > 0
            && let Ok(el_u8) = u8::try_from(el)
        {
            v.extend(std::iter::repeat_n(el_u8, reps));
        }
    }
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
    ext_stack: Vec<u32>,
    best: Option<Vec<u32>>,
}

/// Locate the centered internal connected subgraph matching `target` via ESU enumeration.
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
        ext_stack: Vec::with_capacity(n),
        best: None,
    };
    let n_u32 = usize_to_u32(n);
    for start in 0..n_u32 {
        let su = u32_to_usize(start);
        state.frag.push(start);
        if let Some(slot) = state.visited.get_mut(su) {
            *slot = true;
        }
        if unit_size == 1 {
            evaluate_candidate(&ctx, &mut state);
        } else {
            state.ext_stack.clear();
            extend_neighbors(
                scaffold,
                &mut state.visited,
                start,
                start,
                &mut state.ext_stack,
            );
            let ext_end = state.ext_stack.len();
            esu_repeat(&ctx, &mut state, start, 0, ext_end)?;
            clear_visited(
                &mut state.visited,
                state.ext_stack.get(..ext_end).unwrap_or_default(),
            );
        }
        state.frag.pop();
        if let Some(slot) = state.visited.get_mut(su) {
            *slot = false;
        }
    }
    state
        .best
        .map(|v| v.into_iter().map(u32_to_usize).collect())
        .ok_or(CxError::RepeatNotFound)
}

fn extend_neighbors(
    scaffold: &Molecule,
    visited: &mut [bool],
    atom: u32,
    start: u32,
    out: &mut Vec<u32>,
) {
    for (nbr, _) in scaffold.neighbors(AtomIdx(atom)) {
        let nu = u32_to_usize(nbr.0);
        if nbr.0 > start && visited.get(nu).copied() == Some(false) {
            if let Some(slot) = visited.get_mut(nu) {
                *slot = true;
            }
            out.push(nbr.0);
        }
    }
}

fn clear_visited(visited: &mut [bool], vertices: &[u32]) {
    for &v in vertices {
        let vu = u32_to_usize(v);
        if let Some(slot) = visited.get_mut(vu) {
            *slot = false;
        }
    }
}

fn esu_repeat(
    ctx: &SearchCtx<'_>,
    state: &mut SearchState,
    start: u32,
    ext_start: usize,
    ext_end: usize,
) -> Result<(), CxError> {
    for i in ext_start..ext_end {
        let Some(&w) = state.ext_stack.get(i) else {
            break;
        };
        state.budget = state.budget.saturating_sub(1);
        if state.budget == 0 {
            return Err(CxError::RepeatBudgetExceeded {
                unit_size: ctx.unit_size,
                scaffold_atoms: ctx.atom_count,
                limit: MAX_FRAGMENTS,
            });
        }
        state.frag.push(w);
        if state.frag.len() == ctx.unit_size {
            evaluate_candidate(ctx, state);
            state.frag.pop();
            continue;
        }
        let next_start = state.ext_stack.len();
        state.ext_stack.extend_from_within((i + 1)..ext_end);
        let new_nbrs_start = state.ext_stack.len();
        extend_neighbors(
            ctx.scaffold,
            &mut state.visited,
            w,
            start,
            &mut state.ext_stack,
        );
        let next_end = state.ext_stack.len();
        esu_repeat(ctx, state, start, next_start, next_end)?;
        clear_visited(
            &mut state.visited,
            state
                .ext_stack
                .get(new_nbrs_start..next_end)
                .unwrap_or_default(),
        );
        state.ext_stack.truncate(next_start);
        state.frag.pop();
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
    match &mut state.best {
        None => state.best = Some(state.sorted.clone()),
        Some(best)
            if center_dist(&state.sorted, ctx.atom_count) < center_dist(best, ctx.atom_count) =>
        {
            best.clone_from(&state.sorted);
        }
        _ => {}
    }
}

pub(crate) fn is_internal(atoms: &[u32], mol: &Molecule) -> bool {
    let mut ext = 0usize;
    let mut anchors: [Option<u32>; 2] = [None, None];
    for &a in atoms {
        for (nbr, bond_idx) in mol.neighbors(AtomIdx(a)) {
            if !atoms.contains(&nbr.0) && nbr.0 != a {
                if mol.bond(bond_idx).order != BondOrder::Single {
                    return false;
                }
                ext += 1;
                if ext > 2 {
                    return false;
                }
                match anchors {
                    [None, _] => anchors[0] = Some(nbr.0),
                    [Some(x), None] if x != nbr.0 => anchors[1] = Some(nbr.0),
                    _ => return false,
                }
            }
        }
    }
    ext == 2 && anchors[1].is_some()
}

pub(crate) fn center_dist(atoms: &[u32], n: usize) -> i64 {
    let center = (i64::try_from(n).unwrap_or(i64::MAX) - 1) / 2;
    atoms
        .iter()
        .map(|&a| i64::from(a) - center)
        .sum::<i64>()
        .abs()
}

#[cfg(test)]
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

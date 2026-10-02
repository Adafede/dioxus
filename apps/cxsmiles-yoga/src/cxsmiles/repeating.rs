// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Repeating-unit CX construction (`Sg:n:`) and its helpers.
//!
//! `build_repeating` decides the repeat-unit size (via GCD of per-input atom
//! deltas), finds the single recurring fragment, locates its copy inside the
//! scaffold, and emits `|Sg:n:atoms:n:ht|`. `splice_repeat` physically expands
//! the repeat into `n` copies between its external anchor atoms.

use chematic::core::{AtomIdx, BondOrder, Molecule, MoleculeBuilder};
use chematic::smarts::{QueryMolecule, find_matches};
use chematic::smiles::{parse, write};
use std::collections::{HashMap, HashSet};

use super::graph::{components, matched_mask, molecule_to_query, unmatched_atoms};
use super::roundtrip::{enumerate_repeating, roundtrip_coverage};
use super::types::{Confidence, Construct, CxError, CxResult, CxResult_, RepeatUnit};

pub(crate) fn build_repeating(group: &[Molecule]) -> CxResult_ {
    let mut ordered = group.to_vec();
    ordered.sort_by_key(Molecule::atom_count);
    // Both ends of a non-empty group: return an error (never panic) on empty.
    let (Some(shortest), Some(longest)) = (ordered.first(), ordered.last()) else {
        return Err(CxError("build_repeating: group is empty".into()));
    };

    let scaffold_smiles = write(shortest);
    let scaffold =
        parse(&scaffold_smiles).map_err(|e| CxError(format!("re-parse scaffold: {e:?}")))?;
    let scaffold_q = molecule_to_query(&scaffold);
    let unit_size = {
        let deltas: Vec<usize> = ordered
            .iter()
            .skip(1)
            .map(|m| m.atom_count() - shortest.atom_count())
            .collect();
        gcd_vec(&deltas).max(1)
    };

    let pattern = repeat_pattern(shortest, longest, unit_size, &scaffold_q)?;
    let unit_multiset = unit_multiset(&pattern, longest, unit_size);
    let repeat_atoms = locate_repeat_in_scaffold(&scaffold, &unit_multiset, unit_size)?;

    let nonrepeat = scaffold.atom_count() - repeat_atoms.len();
    let count_max = (longest.atom_count() - nonrepeat) / unit_size;
    let count_min = (shortest.atom_count() - nonrepeat) / unit_size;
    let count_min = count_min.max(1);
    let repeat_unit = RepeatUnit {
        atoms: repeat_atoms,
        min: count_min,
        max: count_max,
    };

    let atoms_field = repeat_unit
        .atoms
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let ext = format!("Sg:n:{atoms_field}:n:ht");
    let cx = format!("{scaffold_smiles} |{ext}|");

    let enumerated = enumerate_repeating(&scaffold, &repeat_unit)?;
    let (_, cov) = roundtrip_coverage(&enumerated, group);
    let frac = cov.fraction();

    Ok(CxResult {
        cx_smiles: cx,
        construct: Construct::Repeating,
        scaffold_smiles,
        floating: Vec::new(),
        confidence: Confidence {
            coverage: cov,
            clean: frac >= 1.0,
        },
        enumerated,
    })
}

/// One copy of the recurring fragment, taken from the extra atoms when the
/// shortest scaffold is aligned into the longest input.
pub(crate) fn repeat_pattern(
    _shortest: &Molecule,
    longest: &Molecule,
    unit_size: usize,
    scaffold_q: &QueryMolecule,
) -> Result<Vec<u32>, CxError> {
    // The first embedding, not the "best" one: an embedding covers every query
    // atom, so every hit has the same length and `unmatched_count` is the same
    // for all of them. Ranking by it chose the first entry while looking like a
    // measurement. `graph::best_match` says the same about the same invariant.
    let hits = find_matches(scaffold_q, longest);
    let hit = hits
        .into_iter()
        .next()
        .ok_or_else(|| CxError("no MCS match of shortest into longest".into()))?;
    let matched = matched_mask(&hit, longest);
    let unmatched = unmatched_atoms(&matched);
    let comps = components(&unmatched, longest);
    let best = comps
        .iter()
        .filter(|c| c.len() % unit_size == 0 && c.len() >= unit_size)
        .min_by_key(|c| c.len())
        .or_else(|| comps.iter().min_by_key(|c| c.len()))
        .ok_or_else(|| CxError("could not determine repeat pattern".into()))?
        .clone();
    Ok(best)
}

/// Element multiset of ONE repeat unit (sorted), taken from the pattern.
pub(crate) fn unit_multiset(pattern: &[u32], longest: &Molecule, unit_size: usize) -> Vec<u8> {
    let k = pattern.len() / unit_size;
    let mut counts: HashMap<u8, usize> = HashMap::new();
    for i in pattern {
        let el = longest.atom(AtomIdx(*i)).element.atomic_number();
        *counts.entry(el).or_insert(0) += 1;
    }
    let mut v: Vec<u8> = counts
        .iter()
        .flat_map(|(el, &c)| std::iter::repeat_n(*el, c / k))
        .collect();
    v.sort_unstable();
    v
}

/// How many partial fragments the repeat-unit search will examine before
/// giving up.
///
/// The search grows a fragment only at its last atom, so it enumerates *paths*,
/// not arbitrary subtrees, and costs `O(atoms · forward_degree^unit_size)`. The
/// unit size is the GCD of the atom-count differences between the caller's
/// molecules, so it is whatever those molecules happen to differ by.
///
/// Measured rather than guessed: a 38-atom branched alkane searched for a
/// 12-atom unit completes in about 250 µs, and a 60-atom chain for a 20-atom
/// unit — C(59, 19) subsets if it were enumerating subsets, though it is not —
/// also completes in microseconds. No realistic input comes near this ceiling,
/// which is why it is a backstop and not a budget.
///
/// It is here because the failure mode would be allocation, not slowness. Each
/// fragment is a `Vec` cloned once per extension and held on the stack, so a
/// search that ran away would not get slow and then stop — it would reach a
/// machine with no memory left. A ceiling makes the same search report an
/// error. At the ceiling and a 20-atom unit the stack holds roughly 30 MB.
const MAX_FRAGMENTS: usize = 200_000;

/// Locate the in-scaffold copy of the repeat unit: the internal connected
/// subgraph of `unit_size` atoms whose element multiset matches `target`.
///
/// # Errors
///
/// Returns a [`CxError`] when no such fragment exists, when `unit_size` cannot
/// fit in `scaffold` at all, or when the search spends [`MAX_FRAGMENTS`]
/// without finishing. The last is reachable with ordinary input — see the note
/// on that constant.
pub(crate) fn locate_repeat_in_scaffold(
    scaffold: &Molecule,
    target: &[u8],
    unit_size: usize,
) -> Result<Vec<usize>, CxError> {
    let n = scaffold.atom_count();
    // Zero atoms is not a fragment, and more atoms than the scaffold has cannot
    // be found by any amount of searching.
    if unit_size == 0 || unit_size > n {
        return Err(CxError(format!(
            "a repeat unit of {unit_size} atoms cannot lie inside a scaffold of {n}"
        )));
    }

    let mut best: Option<Vec<u32>> = None;
    let mut budget = MAX_FRAGMENTS;
    for start in 0..n as u32 {
        let mut stack: Vec<Vec<u32>> = vec![vec![start]];
        while let Some(frag) = stack.pop() {
            // Charged on the way in rather than on the way out, so the ceiling
            // bounds the work and the memory together: every fragment pushed is
            // eventually popped exactly once.
            budget = budget.saturating_sub(1);
            if budget == 0 {
                return Err(CxError(format!(
                    "gave up looking for a {unit_size}-atom repeat unit in a \
                     scaffold of {n} atoms after {MAX_FRAGMENTS} fragments; the \
                     inputs differ by about {unit_size} atoms, which makes the \
                     search exponential"
                )));
            }
            if frag.len() == unit_size {
                let mut f = frag.clone();
                f.sort_unstable();
                if is_internal(&f, scaffold)
                    && multiset(&f, scaffold) == target
                    && best
                        .as_ref()
                        .is_none_or(|b| center_dist(&f, n) < center_dist(b, n))
                {
                    best = Some(f);
                }
                continue;
            }
            // Non-empty by construction: the stack only holds fragments with
            // ≥1 atom (seeded single-element, only ever appended to).
            let Some(&cur) = frag.last() else { continue };
            for (nbr, _) in scaffold.neighbors(AtomIdx(cur)) {
                if !frag.contains(&nbr.0) {
                    let mut nf = frag.clone();
                    nf.push(nbr.0);
                    stack.push(nf);
                }
            }
        }
    }
    best.map(|v| v.iter().map(|&a| a as usize).collect())
        .ok_or_else(|| CxError("could not locate repeat unit in scaffold".into()))
}

pub(crate) fn multiset(atoms: &[u32], mol: &Molecule) -> Vec<u8> {
    let mut v: Vec<u8> = atoms
        .iter()
        .map(|&i| mol.atom(AtomIdx(i)).element.atomic_number())
        .collect();
    v.sort_unstable();
    v
}

pub(crate) fn is_internal(atoms: &[u32], mol: &Molecule) -> bool {
    let set: HashSet<u32> = atoms.iter().copied().collect();
    let mut ext = 0usize;
    for &a in atoms {
        for (nbr, _) in mol.neighbors(AtomIdx(a)) {
            if !set.contains(&nbr.0) && nbr.0 != a {
                ext += 1;
            }
        }
    }
    ext == 2
}

pub(crate) fn center_dist(atoms: &[u32], n: usize) -> i64 {
    let center = (n as i64 - 1) / 2;
    atoms.iter().map(|&a| a as i64 - center).sum::<i64>().abs()
}

pub(crate) fn gcd_vec(xs: &[usize]) -> usize {
    xs.iter().copied().fold(0, gcd)
}

pub(crate) fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// Splice `n` copies of the repeat unit between its two external anchors.
pub(crate) fn splice_repeat(scaffold: &Molecule, repeat_atoms: &[usize], n: usize) -> Molecule {
    if n <= 1 {
        return scaffold.clone();
    }
    let set: HashSet<u32> = repeat_atoms.iter().copied().map(|a| a as u32).collect();
    let unit: Vec<u32> = repeat_atoms.iter().map(|&a| a as u32).collect();
    let anchors: Vec<u32> = repeat_atoms
        .iter()
        .flat_map(|&a| {
            scaffold
                .neighbors(AtomIdx(a as u32))
                .filter(|(nb, _)| !set.contains(&nb.0))
                .map(|(nb, _)| nb.0)
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let anchor_a = anchors.first().copied().unwrap_or(0);
    let anchor_b = anchors.get(1).copied().unwrap_or(0);
    let ep_a = endpoint_for(scaffold, &unit, anchor_a);
    let ep_b = endpoint_for(scaffold, &unit, anchor_b);
    let unit_internal: Vec<(u32, u32, BondOrder)> = scaffold
        .bonds()
        .filter_map(|(_, be)| {
            if set.contains(&be.atom1.0) && set.contains(&be.atom2.0) {
                Some((be.atom1.0, be.atom2.0, be.order))
            } else {
                None
            }
        })
        .collect();
    // `endpoint_for` only ever returns an atom inside `unit` (it either finds an
    // anchor-neighbour that is in `unit`, or falls back to `unit[0]`), so the
    // position always exists; a panic here is unreachable by construction.
    #[expect(clippy::expect_used)]
    let pos_in_unit = |x: u32| -> usize {
        unit.iter()
            .position(|&v| v == x)
            .expect("endpoint atom must occur within the repeat unit")
    };
    let rpos_a = pos_in_unit(ep_a);
    let rpos_b = pos_in_unit(ep_b);

    let sn = scaffold.atom_count();
    let mut b = MoleculeBuilder::new();
    let mut sc: Vec<AtomIdx> = Vec::with_capacity(sn);
    for (_, atom) in scaffold.atoms() {
        sc.push(b.add_atom(atom.clone()));
    }
    // copies of the unit: copy 0 = original (in scaffold), copies 1..n = new.
    let mut copy_atoms: Vec<Vec<AtomIdx>> = Vec::with_capacity(n);
    copy_atoms.push(
        unit.iter()
            .filter_map(|&a| sc.get(a as usize))
            .copied()
            .collect(),
    );
    for _ in 1..n {
        let block: Vec<AtomIdx> = unit
            .iter()
            .map(|&a| b.add_atom(scaffold.atom(AtomIdx(a)).clone()))
            .collect();
        for (a1, a2, order) in &unit_internal {
            if let (Some(p1), Some(p2)) = (
                block.get(pos_in_unit(*a1)).copied(),
                block.get(pos_in_unit(*a2)).copied(),
            ) {
                let _ = b.add_bond(p1, p2, *order);
            }
        }
        copy_atoms.push(block);
    }
    add_scaffold_bonds(&mut b, scaffold, &sc, anchor_a, ep_a, anchor_b, ep_b);
    // Splice bonds. Every lookup is bounded by construction — `copy_atoms` has
    // `n` copies, each of `unit.len()` atoms, and `sc` has one slot per scaffold
    // atom — but a missing slot would silently drop a bond, so each is checked.
    if let (Some(&ra0), Some(&anchor_a_idx)) = (
        copy_atoms.first().and_then(|c| c.get(rpos_a)),
        sc.get(anchor_a as usize),
    ) {
        let _ = b.add_bond(anchor_a_idx, ra0, BondOrder::Single);
    }
    for (k, copy) in copy_atoms.iter().enumerate().take(n.saturating_sub(1)) {
        let Some(next) = copy_atoms.get(k + 1) else {
            break;
        };
        if let (Some(&rb_k), Some(&ra_k1)) = (copy.get(rpos_b), next.get(rpos_a)) {
            let _ = b.add_bond(rb_k, ra_k1, BondOrder::Single);
        }
    }
    if let (Some(&rb_last), Some(&anchor_b_idx)) = (
        copy_atoms.get(n - 1).and_then(|c| c.get(rpos_b)),
        sc.get(anchor_b as usize),
    ) {
        let _ = b.add_bond(rb_last, anchor_b_idx, BondOrder::Single);
    }
    b.build()
}

/// Copy every scaffold bond into `b`, except the two that attach the repeat
/// unit's endpoints to the scaffold — those are the bonds `splice_repeat`
/// replaces with the spliced chain, so copying them too would leave the unit
/// attached at both ends of the original.
fn add_scaffold_bonds(
    b: &mut MoleculeBuilder,
    scaffold: &Molecule,
    sc: &[AtomIdx],
    anchor_a: u32,
    ep_a: u32,
    anchor_b: u32,
    ep_b: u32,
) {
    for (_, be) in scaffold.bonds() {
        let (a, c) = (be.atom1.0, be.atom2.0);
        let is_anchor_bond = (a == anchor_a && c == ep_a)
            || (c == anchor_a && a == ep_a)
            || (a == anchor_b && c == ep_b)
            || (c == anchor_b && a == ep_b);
        if !is_anchor_bond
            && let (Some(ma), Some(mc)) = (sc.get(a as usize).copied(), sc.get(c as usize).copied())
        {
            let _ = b.add_bond(ma, mc, be.order);
        }
    }
}

/// First atom of `unit` that is bonded to `anchor` in `scaffold`.
///
/// When no such atom exists the unit is wholly the endpoint, and its first atom
/// is the answer. `unit` is non-empty by construction — it is the single
/// recurring fragment `find_unit` returned, and `splice_repeat` indexes into it
/// — so the `0` stands only for the case that cannot occur, as `anchor_a` and
/// `anchor_b` above already do.
pub(crate) fn endpoint_for(scaffold: &Molecule, unit: &[u32], anchor: u32) -> u32 {
    let uset: HashSet<u32> = unit.iter().copied().collect();
    for (nbr, _) in scaffold.neighbors(AtomIdx(anchor)) {
        if uset.contains(&nbr.0) {
            return nbr.0;
        }
    }
    unit.first().copied().unwrap_or(0)
}

#[cfg(test)]
// The tests below reach into the molecules they just built, so an out-of-range
// read is a failure of the assertion rather than a separate thing to report.
// `expect` is here because a fixture that stopped parsing is a broken test, not
// a defect in the code under test — that is the distinction the `expect_used`
// denial draws everywhere else.
#[allow(clippy::indexing_slicing, clippy::expect_used)]
mod tests;

#[cfg(test)]
// The crash guard. `MAX_FRAGMENTS` exists because the search fails by
// allocating, and a test that has to allocate its way to an out-of-memory error
// to prove the guard works is not a test — it is a crash. These pin the two
// ways the guard is reached without producing the blow-up.
// `expect` is here because a fixture that stopped parsing is a broken test.
#[allow(clippy::expect_used)]
mod budget;

/// The guard, on the inputs it is most likely to meet.
///
/// These do not exhaust the ceiling, and the comments say so: the point is that
/// the search terminates and answers on inputs from "a unit that cannot exist"
/// to "a unit that is most of the molecule", without the caller having to
/// discover that by running out of memory.
#[cfg(test)]
// `expect` here is a fixture that stopped parsing.
#[allow(clippy::expect_used)]
mod pathological;

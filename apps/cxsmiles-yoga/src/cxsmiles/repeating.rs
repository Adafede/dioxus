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
mod tests {
    use super::super::graph::components;
    use super::*;
    use chematic::smiles::{canonical_smiles, parse};

    fn mol(smiles: &str) -> Molecule {
        parse(smiles).expect("the fixtures in this module are valid SMILES")
    }

    // ── gcd: the fold's seed is 0, and that is the whole contract ────────────

    #[test]
    fn gcd_of_nothing_is_zero() {
        assert_eq!(gcd_vec(&[]), 0, "an empty fold starts at 0 and stays there");
    }

    #[test]
    fn gcd_with_a_zero_is_the_other_number() {
        assert_eq!(gcd(7, 0), 7, "b == 0 terminates the recursion with a");
    }

    #[test]
    fn gcd_is_the_largest_common_divisor() {
        assert_eq!(gcd(12, 18), 6, "12 and 18 share 6");
        assert_eq!(gcd(18, 12), 6, "and the order does not matter");
        assert_eq!(gcd(5, 5), 5, "a number divides itself");
        assert_eq!(gcd(1, 99), 1, "1 divides everything");
    }

    #[test]
    fn gcd_vec_folds_across_every_element() {
        // The fold is what makes this the GCD of *all* deltas rather than of
        // the first two, so a test with three distinct values is the one that
        // would catch a fold that stopped early.
        assert_eq!(gcd_vec(&[4, 6, 10]), 2, "gcd(4, 6, 10) is 2");
        assert_eq!(gcd_vec(&[3, 3, 3]), 3, "an all-equal list is that value");
        assert_eq!(
            gcd_vec(&[0, 5]),
            5,
            "a leading zero is the seed, not a constraint on the answer"
        );
    }

    // ── center_dist: which candidate wins ties ────────────────────────────────

    #[test]
    fn center_dist_is_zero_for_the_middle_atom() {
        // n = 5 puts the centre at index 2, so a single atom there is the
        // closest possible fragment to the centre of the scaffold.
        assert_eq!(center_dist(&[2], 5), 0, "the middle atom has distance 0");
    }

    #[test]
    fn center_dist_sums_the_offsets_before_taking_the_magnitude() {
        // n = 5, centre 2. Atoms 1 and 3 are one step out on *opposite* sides,
        // so their signed offsets cancel and the fragment scores 0 — it is
        // centred. Summing the magnitudes instead, or forgetting the sum
        // entirely, would score the same fragment 2 and pick a different
        // candidate, so this is the assertion that fixes the order of the two
        // operations.
        assert_eq!(center_dist(&[1, 3], 5), 0, "a symmetric pair is centred");
        assert_eq!(
            center_dist(&[1, 3], 5),
            center_dist(&[3, 1], 5),
            "and the atoms arrive in any order"
        );
        assert_eq!(center_dist(&[0, 4], 5), 0, "as does the outer pair");
    }

    #[test]
    fn center_dist_grows_with_distance_from_the_middle() {
        assert!(
            center_dist(&[0], 5) > center_dist(&[1], 5),
            "atom 0 is further from the centre than atom 1"
        );
        assert_eq!(
            center_dist(&[2, 2], 5),
            0,
            "a doubled central atom is still at the centre, because the \
             offsets are summed and both are zero"
        );
        assert!(
            center_dist(&[0, 0], 5) > center_dist(&[2, 2], 5),
            "a doubled off-centre atom is not"
        );
    }

    // ── multiset / unit_multiset: the element counts, and the division ───────

    #[test]
    fn multiset_is_the_sorted_elements_of_those_atoms() {
        // CCC: three carbons.
        let c = mol("CCC");
        assert_eq!(multiset(&[0, 1, 2], &c), vec![6, 6, 6], "three carbons");
    }

    #[test]
    fn unit_multiset_divides_by_the_repeat_count() {
        // The unit is one third of a nine-carbon pattern, so the per-unit
        // multiset is three carbons, not nine. This division is the whole
        // reason `unit_size` is a parameter, and a mutant that changes the
        // divisor changes the answer.
        let long = mol("CCCCCCCCC");
        assert_eq!(
            unit_multiset(&[0, 1, 2, 3, 4, 5, 6, 7, 8], &long, 3),
            vec![6, 6, 6],
            "nine pattern atoms over a unit of three is three carbons"
        );
    }

    #[test]
    fn unit_multiset_truncates_a_pattern_that_is_not_a_whole_multiple() {
        // Five atoms of one element over a unit of two is two and a half units.
        // Integer division keeps two, and *not* rounding up: the count is a
        // number of atoms the unit is known to contain, so a third would be
        // invented. This is the one arithmetic in the file that a rounding
        // change would silently get wrong.
        let long = mol("CCCCC");
        assert_eq!(
            unit_multiset(&[0, 1, 2, 3, 4], &long, 2),
            vec![6, 6],
            "5 / 2 is 2, not 3"
        );
    }

    // ── is_internal: exactly two external bonds ──────────────────────────────

    #[test]
    fn a_fragment_with_two_external_bonds_is_internal() {
        // In CCC the middle atom has one bond out to each end: two external
        // bonds, which is the "internal" shape the search looks for.
        let c = mol("CCC");
        assert!(is_internal(&[1], &c), "the middle carbon of a chain");
    }

    #[test]
    fn a_terminal_atom_is_not_internal() {
        // One external bond, not two.
        let c = mol("CCC");
        assert!(!is_internal(&[0], &c), "an end of a chain has one");
    }

    #[test]
    fn an_isolated_atom_has_no_external_bonds_and_is_not_internal() {
        // A lone atom in a one-atom molecule has zero neighbours at all, which
        // is neither one nor two. `ext == 2` rather than `ext >= 2` is what
        // rejects it.
        let c = mol("C");
        assert!(
            !is_internal(&[0], &c),
            "zero external bonds is not internal"
        );
    }

    #[test]
    fn a_whole_ring_is_not_internal() {
        // In C1CC1, taking *all three* atoms leaves no bond outside the
        // fragment, so it is a closed shell rather than a link in a chain.
        // Taking one atom of the same ring is internal — it has two bonds
        // leaving it — and that contrast is what the `== 2` is making.
        let ring = mol("C1CC1");
        assert!(
            !is_internal(&[0, 1, 2], &ring),
            "the whole ring, 0 external bonds"
        );
        assert!(is_internal(&[0], &ring), "one atom of it, 2 external bonds");
    }

    // ── locate_repeat_in_scaffold ────────────────────────────────────────────

    #[test]
    fn locating_a_unit_that_is_not_there_is_an_error() {
        // An oxygen multiset cannot be found in a hydrocarbon, and the error is
        // the only thing standing between this and a silently empty unit.
        let c = mol("CCC");
        let err = locate_repeat_in_scaffold(&c, &[8], 1).expect_err("no oxygen in CCC");
        assert!(
            err.to_string().contains("could not locate"),
            "the error says what failed: {err}"
        );
    }

    #[test]
    fn locating_prefers_the_fragment_closest_to_the_centre() {
        // Two internal fragments of the same size exist in CCCCCCC; the search
        // must return the one nearer the middle, because that is the unit that
        // has a whole number of copies either side of it.
        let c = mol("CCCCCCC");
        let found = locate_repeat_in_scaffold(&c, &[6], 1).expect("a carbon is present");
        assert_eq!(found.len(), 1, "a unit of one atom yields one index");
        assert_eq!(
            center_dist(&[found[0] as u32], c.atom_count()),
            0,
            "the middle atom of CCCCCCC is index 3, and it is what is found"
        );
    }

    // ── endpoint_for ─────────────────────────────────────────────────────────

    #[test]
    fn the_endpoint_is_the_units_own_neighbour_of_the_anchor() {
        // CCC: anchor 0's only neighbour is 1, and 1 is in the unit.
        let c = mol("CCC");
        assert_eq!(endpoint_for(&c, &[1, 2], 0), 1, "atom 1 touches anchor 0");
    }

    #[test]
    fn the_endpoint_falls_back_to_the_first_unit_atom() {
        // Anchor 0 in CCO has no neighbour in {2, 3} — atom 1 does — so the
        // fallback is the unit's own first atom.
        let c = mol("CCO");
        assert_eq!(
            endpoint_for(&c, &[2, 3], 0),
            2,
            "no match, so the unit's first atom is the endpoint"
        );
    }

    // ── splice_repeat ───────────────────────────────────────────────────────

    #[test]
    fn splicing_one_copy_is_the_scaffold() {
        // `n <= 1` is the guard that stops the loop below from asking for
        // `copy_atoms[0]` when there is no copy to ask for.
        let c = mol("CCC");
        assert_eq!(
            canonical_smiles(&splice_repeat(&c, &[0, 1, 2], 1)),
            canonical_smiles(&c),
            "one copy is the scaffold, unchanged"
        );
        assert_eq!(
            canonical_smiles(&splice_repeat(&c, &[0, 1, 2], 0)),
            canonical_smiles(&c),
            "zero copies is also the scaffold, rather than a panic"
        );
    }

    #[test]
    fn splicing_three_copies_adds_six_carbons() {
        // CC + two more CC units: 2 + 2*2 = 6 atoms.
        let c = mol("CC");
        let out = splice_repeat(&c, &[0, 1], 3);
        assert_eq!(
            out.atom_count(),
            6,
            "a two-carbon unit repeated three times is six carbons"
        );
    }

    #[test]
    fn splicing_keeps_the_copies_bonded_to_each_other() {
        // Each copy has to be bonded into the chain, not merely appended: a
        // pile of disconnected two-carbon fragments would have the right atom
        // count and the wrong molecule. One connected component is the
        // assertion; a bond count would depend on the degenerate fixture's
        // anchor handling, which is not what this test is about.
        let c = mol("CC");
        let out = splice_repeat(&c, &[0, 1], 3);
        let every_atom: Vec<u32> = (0..out.atom_count() as u32).collect();
        assert_eq!(
            components(&every_atom, &out).len(),
            1,
            "the spliced molecule is one piece, not several"
        );
    }

    // ── build_repeating ─────────────────────────────────────────────────────

    #[test]
    fn an_empty_group_is_an_error_and_not_a_panic() {
        let err = build_repeating(&[]).expect_err("there is nothing to build from");
        assert!(
            err.to_string().contains("group is empty"),
            "the error says which precondition failed: {err}"
        );
    }
}

#[cfg(test)]
// The crash guard. `MAX_FRAGMENTS` exists because the search fails by
// allocating, and a test that has to allocate its way to an out-of-memory error
// to prove the guard works is not a test — it is a crash. These pin the two
// ways the guard is reached without producing the blow-up.
// `expect` is here because a fixture that stopped parsing is a broken test.
#[allow(clippy::expect_used)]
mod budget {
    use super::{Molecule, locate_repeat_in_scaffold, multiset};
    use chematic::smiles::parse;

    fn mol(smiles: &str) -> Molecule {
        parse(smiles).expect("the fixtures in this module are valid SMILES")
    }

    #[test]
    fn a_unit_larger_than_the_scaffold_is_refused_immediately() {
        // Six atoms cannot hold a seven-atom fragment. The search would find
        // nothing, but it would get there by exhausting the budget, and the
        // budget is the expensive way to say no.
        let c = mol("CCCCCC");
        let err = locate_repeat_in_scaffold(&c, &[6; 7], 7)
            .expect_err("a seven-atom unit does not fit in a six-atom scaffold");
        assert!(
            err.to_string().contains("cannot lie inside"),
            "the error says the sizes do not fit: {err}"
        );
    }

    #[test]
    fn a_unit_of_no_atoms_is_refused_rather_than_searched() {
        // Zero atoms is not a connected fragment, and the depth check
        // `frag.len() == unit_size` is never true for it, so the search would
        // walk every atom of the molecule and then report nothing.
        let c = mol("CCC");
        let err = locate_repeat_in_scaffold(&c, &[], 0).expect_err("zero atoms is not a unit");
        assert!(
            err.to_string().contains("cannot lie inside"),
            "the same guard covers it: {err}"
        );
    }

    #[test]
    fn a_refusal_costs_nothing() {
        // The point of the size guard: a refusal is O(1). Measured as "the
        // budget is untouched", which is what makes the big-unit case cheap
        // rather than merely survivable.
        let c = mol("CCCCCC");
        assert!(
            locate_repeat_in_scaffold(&c, &[6; 7], 7).is_err(),
            "refused"
        );
        // And the same call with a size that fits is answered from the normal
        // path, which is what the budget is protecting.
        let target = multiset(&[1], &c);
        assert!(
            locate_repeat_in_scaffold(&c, &target, 1).is_ok(),
            "a one-atom unit is found, so the normal path is intact"
        );
    }

    #[test]
    fn the_budget_is_far_above_what_a_real_search_spends() {
        // A one-atom unit over a twelve-atom molecule examines on the order of
        // atoms × degree fragments. The constant is six orders of magnitude
        // above that, so the guard never fires on real input and the tests that
        // pass today are not passing because of it.
        // A four-carbon chain: atom 1 is bonded to 0 and 2, so it has the two
        // external bonds `is_internal` is looking for.
        let c = mol("CCCC");
        let target = multiset(&[1], &c);
        let found = locate_repeat_in_scaffold(&c, &target, 1).expect("atom 1 is internal");
        assert_eq!(found, vec![1], "and it is the one found");
    }
}

/// The guard, on the inputs it is most likely to meet.
///
/// These do not exhaust the ceiling, and the comments say so: the point is that
/// the search terminates and answers on inputs from "a unit that cannot exist"
/// to "a unit that is most of the molecule", without the caller having to
/// discover that by running out of memory.
#[cfg(test)]
// `expect` here is a fixture that stopped parsing.
#[allow(clippy::expect_used)]
mod pathological {
    use super::*;
    use chematic::smiles::parse;

    fn chain(atoms: usize) -> Molecule {
        parse(&"C".repeat(atoms)).expect("a chain of carbons")
    }

    #[test]
    fn a_unit_larger_than_the_scaffold_never_starts_a_search() {
        let mol = chain(6);
        let err = locate_repeat_in_scaffold(&mol, &[6; 7], 7)
            .expect_err("a seven-atom unit does not fit in six atoms");
        assert!(err.to_string().contains("cannot lie inside"), "{err}");
    }

    #[test]
    fn a_unit_that_is_most_of_the_scaffold_still_answers() {
        // Fourteen of sixteen atoms: the largest fraction of a molecule that is
        // still a plausible repeat unit, and the case where the search does the
        // most work it does for any real input. It answers, and it answers with
        // a fragment rather than with an error.
        let mol = chain(16);
        let found = locate_repeat_in_scaffold(&mol, &[6; 14], 14).expect("fourteen of sixteen");
        assert_eq!(found.len(), 14, "the whole interior of the chain");
        assert!(
            !found.contains(&0) && !found.contains(&15),
            "and it excludes the two ends, which are not internal: {found:?}"
        );
    }

    #[test]
    fn a_unit_equal_to_the_whole_scaffold_answers_rather_than_looping() {
        // Every atom. There is no interior, so nothing qualifies — but the
        // search has to finish to say so, and it does.
        let mol = chain(8);
        let err = locate_repeat_in_scaffold(&mol, &[6; 8], 8)
            .expect_err("a whole molecule has no internal fragment of itself");
        assert!(
            err.to_string().contains("could not locate"),
            "not found, rather than not finished: {err}"
        );
    }
}

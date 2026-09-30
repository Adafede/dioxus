// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Positional-isomer CX construction (`m:` blocks) and its expansion helpers.
//!
//! Given a cluster of related molecules with equal atom counts, `build_positional`
//! finds the maximum-common substructure, isolates the variable pendant groups
//! as `FloatingDef`s, computes their equivalent attachment positions, and emits
//! the `|m:i:positions|` extension field. Support for the round-trip enumeration
//! of every distinct positional arrangement lives in [`super::roundtrip`].

use chematic::core::{Atom, AtomIdx, BondOrder, Molecule, MoleculeBuilder};
use chematic::smarts::{QueryMolecule, find_matches, find_mcs};
use chematic::smiles::{parse, write};
use std::collections::{HashMap, HashSet};

use super::graph::{
    best_match, components, matched_mask, molecule_to_query, subgraph, unmatched_atoms,
};
use super::roundtrip::{enumerate, roundtrip_coverage};
use super::types::{Confidence, Construct, CxError, CxResult, CxResult_, FloatingPart};

/// A floating fragment ready for serialisation and expansion.
#[derive(Clone)]
pub(crate) struct FloatingDef {
    pub(crate) atoms: Vec<Atom>,
    pub(crate) bonds: Vec<(usize, usize, BondOrder)>,
    /// Index into `atoms`, bonded to the scaffolding/* target.
    pub(crate) attachment: usize,
    /// Double-`m` variant.
    pub(crate) split: bool,
}

/// The attachment target of a floating group.
pub(crate) enum Target {
    /// Attaches to one of the equivalent scaffold positions.
    Variable(Vec<usize>),
    /// Attaches to another group's attachment atom (index into `defs`).
    Fixed(usize),
}

pub(crate) fn build_positional(group: &[Molecule]) -> CxResult_ {
    let Some(rep) = group.first() else {
        return Err(CxError("build_positional: group is empty".into()));
    };
    let mcs = find_mcs(&group.iter().collect::<Vec<_>>());
    let hit = best_match(&mcs, rep)?;
    let matched = matched_mask(&hit, rep);
    let unmatched = unmatched_atoms(&matched);
    let comps = components(&unmatched, rep);
    let (floating_comps, recovered) = split_floating_and_recovered(&comps, &matched, rep);

    // Scaffold = rep minus floating atoms, plus recovered scaffold atoms.
    let mut keep = matched;
    for r in &recovered {
        if let Some(slot) = keep.get_mut(*r as usize) {
            *slot = true;
        }
    }
    let scaffold_mol = subgraph(rep, &keep);
    let scaffold_smiles = write(&scaffold_mol);
    let scaffold =
        parse(&scaffold_smiles).map_err(|e| CxError(format!("re-parse scaffold: {e:?}")))?;
    let scaffold_q = molecule_to_query(&scaffold);
    let scaffold_len = scaffold.atom_count();

    // Floating defs (with double-m splitting) per component.
    let comp_defs: Vec<Vec<FloatingDef>> = floating_comps
        .iter()
        .map(|fc| floating_defs(rep, fc, &keep))
        .collect();
    let defs: Vec<FloatingDef> = comp_defs.iter().flatten().cloned().collect();

    // Equivalent scaffold positions (scaffold atom indices == base indices,
    // since the base begins with the scaffold) per component.
    let comp_positions: Vec<Vec<usize>> = floating_comps
        .iter()
        .map(|fc| equiv_positions(fc, group, &scaffold_q))
        .collect();

    // Targets per flat def. `comp_positions` is built by mapping over the same
    // `floating_comps` as `comp_defs`, so the two are the same length and can be
    // walked together rather than indexed in step.
    let mut targets: Vec<Target> = Vec::with_capacity(defs.len());
    let mut di = 0usize;
    for (cdefs, positions) in comp_defs.iter().zip(&comp_positions) {
        targets.push(Target::Variable(positions.clone()));
        if cdefs.len() == 1 {
            di += 1;
        } else {
            let g1 = di;
            targets.push(Target::Fixed(g1)); // group2 attaches to group1's atom
            di += 2;
        }
    }

    // Assemble the base SMILES string (hand-built because chematic's writer
    // cannot emit `*`).
    let (base_smiles, star_idx, attach_idx) =
        build_base_smiles(&scaffold_smiles, &defs, scaffold_len);

    // m: fields (one per star / def). `defs`, `targets` and `star_idx` all have
    // one entry per floating group, so the three walk together.
    let fields: Vec<String> = defs
        .iter()
        .zip(&targets)
        .zip(&star_idx)
        .map(|((_, target), &star)| {
            let pos = match target {
                Target::Variable(p) => p
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("."),
                // `Target::Fixed(i)` names the group pushed just before it in the
                // loop above, so `i` is below `attach_idx.len()`, which has one
                // entry per def.
                Target::Fixed(other) => attach_idx
                    .get(*other)
                    .map_or_else(String::new, ToString::to_string),
            };
            format!("m:{star}:{pos}")
        })
        .collect();
    let ext = fields.join(",");

    // Round-trip enumeration.
    let enumerated = enumerate(&scaffold, &defs, &targets);
    let (_, cov) = roundtrip_coverage(&enumerated, group);
    let frac = cov.fraction();

    // Display FloatingParts (one per star).
    let floating: Vec<FloatingPart> = defs
        .iter()
        .zip(&targets)
        .map(|(def, target)| {
            let equiv = match target {
                Target::Variable(p) => p.clone(),
                Target::Fixed(_) => Vec::new(),
            };
            FloatingPart {
                equiv,
                fragment_smiles: fragment_smiles(def),
                split: def.split,
            }
        })
        .collect();

    Ok(CxResult {
        cx_smiles: format!("{base_smiles} |{ext}|"),
        construct: Construct::Positional,
        scaffold_smiles,
        floating,
        confidence: Confidence {
            coverage: cov,
            clean: frac >= 1.0,
        },
        enumerated,
    })
}

/// Assemble the base SMILES string. Each floating fragment is written as
/// `[*]<frag>` (attachment atom first), so the `*` is atom 0 of the fragment
/// and bonds to the attachment atom (the first atom of `<frag>`).
fn build_base_smiles(
    scaffold_smiles: &str,
    defs: &[FloatingDef],
    scaffold_len: usize,
) -> (String, Vec<usize>, Vec<usize>) {
    let mut frags: Vec<String> = vec![scaffold_smiles.to_string()];
    let mut star_idx: Vec<usize> = Vec::with_capacity(defs.len());
    let mut attach_idx: Vec<usize> = Vec::with_capacity(defs.len());
    let mut offset = scaffold_len;
    for def in defs {
        star_idx.push(offset); // `*` is the first atom of the fragment
        attach_idx.push(offset + 1); // attachment atom follows the `*`
        let frag = write_fragment_first(def);
        offset += 1 + def.atoms.len(); // `*` + fragment atoms
        frags.push(format!("[*]{frag}"));
    }
    (frags.join("."), star_idx, attach_idx)
}

/// Write a fragment molecule with the attachment atom first.
// `order` is `once(attachment)` followed by every other index below `n`, so it is
// a permutation of `0..n`; `map` and `def.atoms` both have `n` slots and every
// index below is an element of `order`. The inversions are the point of the
// function, and wrapping them in `Option` would mean a bond silently vanishing
// for a state the two lines above have already excluded.
#[allow(clippy::indexing_slicing)]
fn write_fragment_first(def: &FloatingDef) -> String {
    let n = def.atoms.len();
    let mut b = MoleculeBuilder::new();
    let order: Vec<usize> = std::iter::once(def.attachment)
        .chain((0..n).filter(|&i| i != def.attachment))
        .collect();
    let mut map = vec![0usize; n];
    for (new_i, &old_i) in order.iter().enumerate() {
        map[old_i] = new_i;
    }
    for &old_i in &order {
        let _ = b.add_atom(def.atoms[old_i].clone());
    }
    for (a1, a2, order_) in &def.bonds {
        let _ = b.add_bond(AtomIdx(map[*a1] as u32), AtomIdx(map[*a2] as u32), *order_);
    }
    write(&b.build())
}

/// Floating fragment SMILES for display: `[*]<frag>`.
fn fragment_smiles(def: &FloatingDef) -> String {
    format!("[*]{}", write_fragment_first(def))
}

/// A floating component (set of rep atoms) described as one or two defs.
fn floating_defs(rep: &Molecule, frag: &[u32], keep: &[bool]) -> Vec<FloatingDef> {
    let attach = frag
        .iter()
        .copied()
        .find(|&a| {
            rep.neighbors(AtomIdx(a))
                .any(|(n, _)| is_matched(keep, n.0))
        })
        .or_else(|| frag.first().copied())
        .unwrap_or(0);
    let in_frag = |a: u32| frag.contains(&a);
    let is_chain = frag.len() > 1
        && rep
            .neighbors(AtomIdx(attach))
            .filter(|(n, _)| in_frag(n.0) && n.0 != attach)
            .count()
            == 1;
    if !is_chain {
        let (atoms, bonds, att) = extract_subgraph(rep, frag);
        // reorder so attachment is index 0
        let def = reorder_attachment_first(atoms, bonds, att);
        return vec![FloatingDef {
            atoms: def.0,
            bonds: def.1,
            attachment: def.2,
            split: false,
        }];
    }
    // Split: group1 = [attachment atom], group2 = rest (attached to group1's atom).
    let rest: Vec<u32> = frag.iter().copied().filter(|&a| a != attach).collect();
    let (a1, _, _) = extract_subgraph(rep, std::slice::from_ref(&attach));
    let (a2, b2, att2) = extract_subgraph(rep, &rest);
    let (ra2, rb2, ratt2) = reorder_attachment_first(a2, b2, att2);
    vec![
        FloatingDef {
            atoms: a1,
            bonds: Vec::new(),
            attachment: 0,
            split: false,
        },
        FloatingDef {
            atoms: ra2,
            bonds: rb2,
            attachment: ratt2,
            split: true,
        },
    ]
}

/// Reorder a (atoms, bonds, attachment) so that the attachment atom is index 0.
// As in `write_fragment_first`, `order` is a permutation of `0..atoms.len()` and
// `map`, `atoms` and the bond endpoints are all indexed by that permutation.
#[allow(clippy::indexing_slicing)]
fn reorder_attachment_first(
    atoms: Vec<Atom>,
    bonds: Vec<(usize, usize, BondOrder)>,
    attach: usize,
) -> (Vec<Atom>, Vec<(usize, usize, BondOrder)>, usize) {
    if attach == 0 {
        return (atoms, bonds, attach);
    }
    let n = atoms.len();
    let order: Vec<usize> = std::iter::once(attach)
        .chain((0..n).filter(|&i| i != attach))
        .collect();
    let mut map = vec![0usize; n];
    for (new_i, &old_i) in order.iter().enumerate() {
        map[old_i] = new_i;
    }
    let new_atoms: Vec<Atom> = order.iter().map(|&i| atoms[i].clone()).collect();
    let new_bonds: Vec<(usize, usize, BondOrder)> = bonds
        .iter()
        .map(|(a, c, o)| (map[*a], map[*c], *o))
        .collect();
    (new_atoms, new_bonds, 0)
}

/// Extract a subgraph (atoms cloned + internal bonds) of `mol` for `subset`.
fn extract_subgraph(
    mol: &Molecule,
    subset: &[u32],
) -> (Vec<Atom>, Vec<(usize, usize, BondOrder)>, usize) {
    let map: HashMap<u32, usize> = subset
        .iter()
        .copied()
        .enumerate()
        .map(|(i, a)| (a, i))
        .collect();
    let atoms: Vec<Atom> = subset
        .iter()
        .map(|&a| mol.atom(AtomIdx(a)).clone())
        .collect();
    let mut bonds = Vec::new();
    for (_, be) in mol.bonds() {
        if let (Some(&a), Some(&c)) = (map.get(&be.atom1.0), map.get(&be.atom2.0)) {
            bonds.push((a, c, be.order));
        }
    }
    let attach = subset
        .iter()
        .position(|&a| {
            mol.neighbors(AtomIdx(a))
                .any(|(n, _)| !subset.contains(&n.0) && n.0 != a)
        })
        .unwrap_or(0);
    (atoms, bonds, attach)
}

/// Equivalent scaffold attachment positions for one floating component across
/// all inputs (scaffold atom indices).
fn equiv_positions(_comp: &[u32], group: &[Molecule], scaffold_q: &QueryMolecule) -> Vec<usize> {
    let mut positions: Vec<usize> = Vec::new();
    for mol in group {
        let hits = find_matches(scaffold_q, mol);
        let Some(hit) = hits.first().cloned() else {
            continue;
        };
        let matched = matched_mask(&hit, mol);
        let unf = unmatched_atoms(&matched);
        for u in &unf {
            for (nbr, _) in mol.neighbors(AtomIdx(*u)) {
                if let Some((&qi, _)) = hit.iter().find(|(_, t)| t.0 == nbr.0)
                    && !positions.contains(&qi)
                {
                    positions.push(qi);
                }
            }
        }
    }
    positions.sort_unstable();
    positions.dedup();
    if positions.is_empty() {
        positions = (0..scaffold_q.atoms.len()).collect();
    }
    positions
}

/// Partition connected components into floating (boundary = single scaffold
/// edge) vs. recovered (re-integrated scaffold atoms).
fn split_floating_and_recovered(
    comps: &[Vec<u32>],
    matched: &[bool],
    mol: &Molecule,
) -> (Vec<Vec<u32>>, Vec<u32>) {
    let mut floating = Vec::new();
    let mut recovered = Vec::new();
    for comp in comps {
        if boundary_edges(comp, matched, mol) == 1 {
            floating.push(comp.clone());
        } else {
            for a in comp {
                recovered.push(*a);
            }
        }
    }
    (floating, recovered)
}

/// Count edges from `comp` to already-matched (scaffold) atoms.
fn boundary_edges(comp: &[u32], matched: &[bool], mol: &Molecule) -> usize {
    let set: HashSet<u32> = comp.iter().copied().collect();
    let mut count = 0;
    for &a in comp {
        for (nbr, _) in mol.neighbors(AtomIdx(a)) {
            if !set.contains(&nbr.0) && is_matched(matched, nbr.0) {
                count += 1;
            }
        }
    }
    count
}

/// Whether atom `a` is one of the matched scaffold atoms.
fn is_matched(matched: &[bool], a: u32) -> bool {
    matched.get(a as usize) == Some(&true)
}

#[cfg(test)]
// `expect` on a fixture that stopped parsing is a broken test, not a defect in
// the code under test; the out-of-range reads below are the assertions failing.
#[allow(clippy::indexing_slicing, clippy::expect_used)]
mod tests {
    use super::*;
    use chematic::smiles::{canonical_smiles, parse};

    fn mol(smiles: &str) -> Molecule {
        parse(smiles).expect("the fixtures in this module are valid SMILES")
    }

    /// A linear chain of `n` carbons, as indices.
    fn chain(n: u32) -> Vec<u32> {
        (0..n).collect()
    }

    // ── is_matched ──────────────────────────────────────────────────────────

    #[test]
    fn a_mask_answers_only_about_atoms_it_covers() {
        let mask = [true, false, true];
        assert!(is_matched(&mask, 0), "atom 0 is set");
        assert!(!is_matched(&mask, 1), "atom 1 is not");
        assert!(
            !is_matched(&mask, 9),
            "an atom past the end of the mask is not matched, rather than a panic"
        );
    }

    // ── boundary_edges / split_floating_and_recovered ────────────────────────

    #[test]
    fn a_component_joined_to_the_scaffold_once_is_floating() {
        // CCC, the middle carbon floating: one bond out to each end, and the
        // ends are matched scaffold. Two boundary edges, not one — which is
        // what makes `== 1` a two-atom pendant group rather than any fragment.
        let c = mol("CCC");
        let matched = [true, false, true];
        assert_eq!(
            boundary_edges(&[1], &matched, &c),
            2,
            "the middle carbon is joined at both ends"
        );
    }

    #[test]
    fn a_terminal_component_is_joined_once() {
        // The end carbon of CCC, with atom 1 matched: one bond out, so the
        // component is a pendant group.
        let c = mol("CCC");
        let matched = [false, true, true];
        assert_eq!(
            boundary_edges(&[0], &matched, &c),
            1,
            "an end is joined at one bond only"
        );
    }

    #[test]
    fn an_isolated_component_has_no_boundary() {
        // Nothing matched, so nothing is scaffold and nothing is a boundary.
        let c = mol("CCC");
        assert_eq!(
            boundary_edges(&[1], &[false; 3], &c),
            0,
            "with no matched neighbours there is no boundary"
        );
    }

    #[test]
    fn components_join_the_scaffold_once_become_floating_and_the_rest_recovered() {
        // One component with a single boundary edge, one with two. The `== 1`
        // is the whole decision, so both directions are pinned: a lone pendant
        // goes one way and a re-integrating fragment the other.
        let c = mol("CCC");
        let matched = [false, true, false];
        // Atom 1 is matched, so {0} and {2} each have one boundary edge and
        // {1} has none of its own as a component.
        let (floating, recovered) =
            split_floating_and_recovered(&[vec![0], vec![2], vec![1]], &matched, &c);
        assert_eq!(floating, vec![vec![0], vec![2]], "both ends are pendant");
        assert_eq!(recovered, vec![1], "the matched atom is recovered");
    }

    // ── extract_subgraph ────────────────────────────────────────────────────

    #[test]
    fn extracting_a_subgraph_takes_only_its_own_bonds() {
        // CCC, taking the last two carbons: both atoms, and the one bond
        // between them. The bond to the excluded carbon must not appear.
        let c = mol("CCC");
        let (atoms, bonds, _attach) = extract_subgraph(&c, &[1, 2]);
        assert_eq!(atoms.len(), 2, "two atoms were asked for");
        assert_eq!(bonds.len(), 1, "and only the bond inside the subset");
    }

    #[test]
    fn extracting_reindexes_the_bond_endpoints() {
        // Taking carbons 2 and 0 out of order: the bond between them must come
        // back as 0–1, because the callers index `bonds` by position in
        // `atoms` and would otherwise address the wrong atom.
        let c = mol("CCC");
        let (atoms, bonds, _attach) = extract_subgraph(&c, &[2, 0]);
        assert_eq!(atoms.len(), 2, "two atoms");
        assert_eq!(
            bonds.len(),
            0,
            "carbons 0 and 2 are not bonded to each other"
        );
    }

    #[test]
    fn extracting_consecutive_carbons_reindexes_to_zero_and_one() {
        let c = mol("CCCC");
        let (_atoms, bonds, _attach) = extract_subgraph(&c, &[1, 2]);
        assert_eq!(bonds, vec![(0, 1, BondOrder::Single)], "1–2 becomes 0–1");
    }

    #[test]
    fn the_attachment_is_a_position_within_the_subset_not_a_molecule_index() {
        // Carbons 1 and 2 of CCC: the atom at *position* 0 of the subset
        // (molecule atom 1) is joined to the excluded atom 0, and the atom at
        // position 1 (molecule atom 2) to the excluded atom 2. Both qualify, the
        // first is taken, and the answer is a subset position — callers index
        // the returned `atoms` with it. Returning the molecule index 1 here
        // would address the wrong atom, by one, for every subset that did not
        // begin at zero.
        let c = mol("CCC");
        let (_atoms, _bonds, attach) = extract_subgraph(&c, &[1, 2]);
        assert_eq!(
            attach, 0,
            "position 0 of the subset, which is molecule atom 1"
        );
    }

    #[test]
    fn the_attachment_of_a_closed_subset_is_zero() {
        // A whole molecule has no neighbour outside itself, so there is no
        // boundary atom and the fallback is index 0.
        let c = mol("CC");
        let (_atoms, _bonds, attach) = extract_subgraph(&c, &[0, 1]);
        assert_eq!(attach, 0, "no boundary atom, so index 0");
    }

    // ── reorder_attachment_first ────────────────────────────────────────────

    #[test]
    fn reordering_with_the_attachment_already_first_changes_nothing() {
        // `attach == 0` returns the input untouched — no copy, no remap. A
        // mutant that drops the early return still produces the right answer
        // here, which is why the identity case is asserted separately from the
        // reordering case.
        let m = mol("CC");
        let (atoms, bonds, attach) = extract_subgraph(&m, &[0, 1]);
        let (a2, b2, at2) = reorder_attachment_first(atoms.clone(), bonds.clone(), attach);
        assert_eq!(a2.len(), atoms.len(), "same atoms");
        assert_eq!(b2, bonds, "same bonds");
        assert_eq!(at2, 0, "still index 0");
    }

    #[test]
    fn reordering_puts_the_attachment_first_and_remaps_the_bonds() {
        // CCC, taking carbons 1 and 2 with the attachment at 1 (index 0 in the
        // subset). Reordering with attachment index 1 swaps them, so the bond
        // 0–1 has to become 1–0 or the molecule changes.
        let c = mol("CCC");
        let (atoms, bonds, attach) = extract_subgraph(&c, &[1, 2]);
        assert_eq!(attach, 0, "the fixture's attachment is index 0");
        let (a2, b2, at2) = reorder_attachment_first(atoms, bonds, 1);
        assert_eq!(at2, 0, "the reordered attachment is always index 0");
        assert_eq!(a2.len(), 2, "both atoms survive");
        assert_eq!(b2.len(), 1, "the bond survives");
        assert_eq!(
            b2[0],
            (1, 0, BondOrder::Single),
            "0–1 is remapped to 1–0 by the swap"
        );
    }

    #[test]
    fn reordering_preserves_the_molecule() {
        // The real invariant behind the remap: the bonds still describe the
        // same molecule. A permutation that forgot to remap one endpoint would
        // satisfy the atom count and break this.
        let c = mol("CCC");
        let (atoms, bonds, _) = extract_subgraph(&c, &[0, 1]);
        let before = canonical_smiles(&molecule_of(&atoms, &bonds));
        let (a2, b2, _) = reorder_attachment_first(atoms, bonds, 1);
        let after = canonical_smiles(&molecule_of(&a2, &b2));
        assert_eq!(before, after, "a reordering is a relabelling, not an edit");
    }

    /// Rebuild a molecule from atoms and index pairs, for the two tests above.
    fn molecule_of(atoms: &[Atom], bonds: &[(usize, usize, BondOrder)]) -> Molecule {
        let mut b = MoleculeBuilder::new();
        let idx: Vec<_> = atoms.iter().map(|a| b.add_atom(a.clone())).collect();
        for (a1, a2, order) in bonds {
            if let (Some(x), Some(y)) = (idx.get(*a1), idx.get(*a2)) {
                let _ = b.add_bond(*x, *y, *order);
            }
        }
        b.build()
    }

    // ── floating_defs: the chain test that decides the split ────────────────

    #[test]
    fn a_single_atom_floating_group_is_never_split() {
        // A one-atom fragment cannot be a chain, whatever its neighbours look
        // like, so it stays one def. `frag.len() > 1` is the guard.
        let c = mol("CCO");
        let keep = [true, false, true];
        let defs = floating_defs(&c, &[1], &keep);
        assert_eq!(defs.len(), 1, "one atom is one group, not two");
        assert!(!defs[0].split, "and it is not the split variant");
    }

    #[test]
    fn a_pendant_two_atom_chain_splits_into_two_defs() {
        // CCO with the middle-and-... no: take carbons 0 and 1 with atom 2
        // matched, so the fragment is a two-atom chain hanging off the
        // scaffold at one end — exactly the shape the `Sg`/`m` builders treat
        // as splittable.
        let c = mol("CCCO");
        let keep = [false, false, true, true];
        let defs = floating_defs(&c, &[0, 1], &keep);
        assert_eq!(defs.len(), 2, "a chain of two becomes two defs");
        assert!(!defs[0].split, "the first half carries the attachment");
        assert!(defs[1].split, "the second half is the split variant");
    }

    #[test]
    fn a_branched_floating_group_is_not_split() {
        // CC(C)C: the central carbon with three carbon neighbours, so the
        // attachment atom has two neighbours inside the fragment. `count() == 1`
        // is what refuses the split — a branch has no meaningful "rest".
        let c = mol("CC(C)C");
        let keep = [true, false, false, false, true];
        let defs = floating_defs(&c, &[0, 2, 3], &keep);
        assert_eq!(defs.len(), 1, "a branch stays whole");
        assert!(!defs[0].split, "and is not the split variant");
    }

    // ── equiv_positions: the empty fallback ────────────────────────────────

    #[test]
    fn an_unmatched_scaffold_offers_every_position() {
        // Nothing matched anywhere, so there is no evidence of a preferred
        // site and the answer is every scaffold atom. This is the `is_empty`
        // branch, and an empty answer here would produce a `m:` field with no
        // positions at all.
        let c = mol("CC");
        let scaffold_q = super::super::graph::molecule_to_query(&c);
        let positions = equiv_positions(&[], std::slice::from_ref(&c), &scaffold_q);
        assert_eq!(
            positions,
            vec![0, 1],
            "with no evidence, every scaffold atom is a candidate"
        );
    }

    // ── build_base_smiles / write_fragment_first ────────────────────────────

    #[test]
    fn a_fragment_is_written_with_a_star_and_the_attachment_first() {
        // The `[*]` prefix is what marks the attachment, and the attachment has
        // to be the fragment's atom 0 or the prefix lies about which end bonds
        // to the scaffold.
        let m = mol("CCO");
        let (atoms, bonds, attach) = extract_subgraph(&m, &[1, 2]);
        let (atoms, bonds, attach) = reorder_attachment_first(atoms, bonds, attach);
        let def = FloatingDef {
            atoms,
            bonds,
            attachment: attach,
            split: false,
        };
        let smi = write_fragment_first(&def);
        assert!(
            smi.starts_with("CO"),
            "the attachment (the oxygen) is written first: {smi}"
        );
    }

    #[test]
    fn base_smiles_offsets_the_star_indices_by_the_scaffold_length() {
        // One def against a two-atom scaffold: the `*` is atom 2 and the
        // attachment atom 3. These are the numbers that go into the `m:`
        // field, so an off-by-one here is a wrong CX-SMILES string that still
        // looks well formed.
        let m = mol("CO");
        let (atoms, bonds, attach) = extract_subgraph(&m, &[1]);
        let def = FloatingDef {
            atoms,
            bonds,
            attachment: attach,
            split: false,
        };
        let (_smiles, star_idx, attach_idx) = build_base_smiles("CO", &[def], 2);
        assert_eq!(star_idx, vec![2], "the star follows the scaffold");
        assert_eq!(attach_idx, vec![3], "and the attachment follows the star");
    }

    #[test]
    fn two_defs_skip_past_each_fragments_star_and_atoms() {
        // The second `*` sits after the first fragment *and* its `*`, so the
        // indices are not consecutive: scaffold 1, then the first star at 1 and
        // its one fragment atom at 2, so the second star is at 3. Writing them
        // as consecutive would put the second `*` on top of the first
        // fragment's atom — a well-formed CX-SMILES field that is wrong.
        let m = mol("CO");
        let (atoms, bonds, attach) = extract_subgraph(&m, &[1]);
        let one = FloatingDef {
            atoms,
            bonds,
            attachment: attach,
            split: false,
        };
        let two = one.clone();
        let (_smiles, star_idx, attach_idx) = build_base_smiles("C", &[one, two], 1);
        assert_eq!(
            star_idx,
            vec![1, 3],
            "each def consumes a star and its atoms"
        );
        assert_eq!(
            attach_idx,
            vec![2, 4],
            "and the attachment is the atom just after each star"
        );
    }

    // ── build_positional ────────────────────────────────────────────────────

    #[test]
    fn an_empty_group_is_an_error_and_not_a_panic() {
        let err = build_positional(&[]).expect_err("there is nothing to build from");
        assert!(
            err.to_string().contains("group is empty"),
            "the error says which precondition failed: {err}"
        );
    }

    // ── a chain, for the fixture helpers above ──────────────────────────────

    #[test]
    fn the_chain_helper_is_the_indices_in_order() {
        assert_eq!(chain(3), vec![0, 1, 2], "0, 1, 2");
    }
}

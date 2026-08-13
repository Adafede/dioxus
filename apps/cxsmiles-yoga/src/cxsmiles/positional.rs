// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

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
pub struct FloatingDef {
    pub atoms: Vec<Atom>,
    pub bonds: Vec<(usize, usize, BondOrder)>,
    /// Index into `atoms`, bonded to the scaffolding/* target.
    pub attachment: usize,
    /// Double-`m` variant.
    pub split: bool,
}

/// The attachment target of a floating group.
pub enum Target {
    /// Attaches to one of the equivalent scaffold positions.
    Variable(Vec<usize>),
    /// Attaches to another group's attachment atom (index into `defs`).
    Fixed(usize),
}

pub fn build_positional(group: &[Molecule]) -> CxResult_ {
    let mcs = find_mcs(&group.iter().collect::<Vec<_>>());
    let rep = &group[0];
    let hit = best_match(&mcs, rep)?;
    let matched = matched_mask(&hit, rep);
    let unmatched = unmatched_atoms(&matched);
    let comps = components(&unmatched, rep);
    let (floating_comps, recovered) = split_floating_and_recovered(&comps, &matched, rep);

    // Scaffold = rep minus floating atoms, plus recovered scaffold atoms.
    let mut keep = matched;
    for r in &recovered {
        keep[*r as usize] = true;
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

    // Targets per flat def.
    let mut targets: Vec<Target> = Vec::with_capacity(defs.len());
    let mut di = 0usize;
    for (ci, cdefs) in comp_defs.iter().enumerate() {
        let positions = comp_positions[ci].clone();
        targets.push(Target::Variable(positions));
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

    // m: fields (one per star / def).
    let fields: Vec<String> = (0..defs.len())
        .map(|j| {
            let pos = match &targets[j] {
                Target::Variable(p) => p
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("."),
                Target::Fixed(other) => attach_idx[*other].to_string(),
            };
            format!("m:{}:{}", star_idx[j], pos)
        })
        .collect();
    let ext = fields.join(",");

    // Round-trip enumeration.
    let enumerated = enumerate(&scaffold, &defs, &targets);
    let (_, cov) = roundtrip_coverage(&enumerated, group);
    let frac = cov.fraction();

    // Display FloatingParts (one per star).
    let floating: Vec<FloatingPart> = (0..defs.len())
        .map(|j| {
            let equiv = match &targets[j] {
                Target::Variable(p) => p.clone(),
                Target::Fixed(_) => Vec::new(),
            };
            FloatingPart {
                star_idx: star_idx[j],
                equiv,
                fragment_smiles: fragment_smiles(&defs[j]),
                split: defs[j].split,
            }
        })
        .collect();

    Ok(CxResult {
        cx_smiles: format!("{base_smiles} |{ext}|"),
        base_smiles,
        construct: Construct::Positional,
        scaffold_smiles,
        floating,
        repeating: None,
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
    let attach = *frag
        .iter()
        .find(|&&a| rep.neighbors(AtomIdx(a)).any(|(n, _)| keep[n.0 as usize]))
        .unwrap_or(&frag[0]);
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
        if hits.is_empty() {
            continue;
        }
        let hit = hits[0].clone();
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
            if !set.contains(&nbr.0) && matched[nbr.0 as usize] {
                count += 1;
            }
        }
    }
    count
}

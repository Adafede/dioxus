// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Positional-isomer CX construction (`m:` blocks).

use chematic::core::{AtomIdx, BondOrder, Molecule};
use chematic::smarts::{QueryMolecule, find_matches, find_mcs};
use chematic::smiles::{canonical_smiles, parse};

use crate::aromatic::expand_aromatic_equiv;
use crate::error::CxError;
use crate::fragment::{
    FloatingDef, build_base_smiles, extract_subgraph, fragment_smiles, matches_multiset,
    reorder_attachment_first, reverse_match_table,
};
use crate::graph::{
    atom_usize, best_match, components, matched_mask, multiset, subgraph, unmatched_atoms,
};
use crate::roundtrip::{enumerate, evaluate_construct};
use crate::types::{AttachmentTarget, ConstructKind, CxResult, FloatingPart};

struct ExtractedScaffold {
    smiles: String,
    mol: Molecule,
    floating_comps: Vec<Vec<u32>>,
    keep: Vec<bool>,
}

pub(crate) fn build_positional_with_options(
    group: &[Molecule],
    aromatic: bool,
) -> Result<CxResult, CxError> {
    let Some(rep) = group.first() else {
        return Err(CxError::EmptyGroup("build_positional"));
    };
    let sc = extract_scaffold(rep, group)?;
    let scaffold_q = crate::graph::molecule_to_query(&sc.mol);
    let comp_defs: Vec<Vec<FloatingDef>> = sc
        .floating_comps
        .iter()
        .map(|fc| floating_defs(rep, fc, &sc.keep))
        .collect();
    let defs: Vec<FloatingDef> = comp_defs.iter().flatten().cloned().collect();
    let comp_positions = collect_positions(
        &sc.floating_comps,
        rep,
        group,
        &scaffold_q,
        &sc.mol,
        aromatic,
    );
    let targets = build_targets(&comp_defs, &comp_positions);
    let (base_smiles, star_indices, attach_idx) =
        build_base_smiles(&sc.smiles, &defs, sc.mol.atom_count());
    let ext = format_m_extension(&targets, &star_indices, &attach_idx);
    let enumerated = enumerate(&sc.mol, &defs, &targets)?;
    let (construct, coverage) = evaluate_construct(&enumerated, group, ConstructKind::Positional);
    Ok(CxResult {
        cx_smiles: if ext.is_empty() {
            base_smiles
        } else {
            format!("{base_smiles} |{ext}|")
        },
        construct,
        scaffold_smiles: sc.smiles,
        floating: build_floating_parts(&defs, &targets),
        repeat_unit: None,
        coverage,
        enumerated,
    })
}

fn extract_scaffold(rep: &Molecule, group: &[Molecule]) -> Result<ExtractedScaffold, CxError> {
    let mcs = find_mcs(&group.iter().collect::<Vec<_>>());
    let hit = best_match(&mcs, rep)?;
    let matched = matched_mask(&hit, rep);
    let comps = components(&unmatched_atoms(&matched), rep);
    let (floating_comps, recovered) = split_floating_and_recovered(&comps, &matched, rep);
    let mut keep = matched;
    for r in &recovered {
        if let Some(slot) = keep.get_mut(usize::try_from(*r).unwrap_or(usize::MAX)) {
            *slot = true;
        }
    }
    let scaffold_mol = subgraph(rep, &keep);
    let smiles = canonical_smiles(&scaffold_mol);
    let mol = parse(&smiles).map_err(|e| CxError::ScaffoldReparse(format!("{e:?}")))?;
    Ok(ExtractedScaffold {
        smiles,
        mol,
        floating_comps,
        keep,
    })
}

fn collect_positions(
    floating_comps: &[Vec<u32>],
    rep: &Molecule,
    group: &[Molecule],
    scaffold_q: &QueryMolecule,
    scaffold: &Molecule,
    aromatic: bool,
) -> Vec<Vec<usize>> {
    floating_comps
        .iter()
        .map(|fc| {
            let pos = equiv_positions(rep, fc, group, scaffold_q);
            if aromatic {
                expand_aromatic_equiv(&pos, scaffold)
            } else {
                pos
            }
        })
        .collect()
}

fn build_targets(
    comp_defs: &[Vec<FloatingDef>],
    comp_positions: &[Vec<usize>],
) -> Vec<AttachmentTarget> {
    let mut targets = Vec::with_capacity(comp_defs.iter().map(Vec::len).sum());
    for (cdefs, positions) in comp_defs.iter().zip(comp_positions) {
        targets.push(AttachmentTarget::Variable(positions.clone()));
        if cdefs.len() != 1 {
            targets.push(AttachmentTarget::Fixed(targets.len() - 1));
        }
    }
    targets
}

fn format_m_extension(
    targets: &[AttachmentTarget],
    star_indices: &[usize],
    attach_idx: &[usize],
) -> String {
    targets
        .iter()
        .zip(star_indices)
        .map(|(target, &star)| {
            let pos = match target {
                AttachmentTarget::Variable(p) => p
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("."),
                AttachmentTarget::Fixed(other) => attach_idx
                    .get(*other)
                    .map_or_else(String::new, ToString::to_string),
            };
            format!("m:{star}:{pos}")
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn build_floating_parts(defs: &[FloatingDef], targets: &[AttachmentTarget]) -> Vec<FloatingPart> {
    defs.iter()
        .zip(targets)
        .map(|(def, target)| FloatingPart {
            equiv: match target {
                AttachmentTarget::Variable(p) => p.clone(),
                AttachmentTarget::Fixed(_) => Vec::new(),
            },
            fragment_smiles: fragment_smiles(def),
            split: def.split,
        })
        .collect()
}

pub(crate) fn floating_defs(rep: &Molecule, frag: &[u32], keep: &[bool]) -> Vec<FloatingDef> {
    let attach = frag
        .iter()
        .copied()
        .find(|&a| {
            rep.neighbors(AtomIdx(a))
                .any(|(n, _)| is_matched(keep, n.0))
        })
        .or_else(|| frag.first().copied())
        .unwrap_or(0);
    let scaffold_order = rep
        .neighbors(AtomIdx(attach))
        .find(|(n, _)| is_matched(keep, n.0))
        .map_or(BondOrder::Single, |(_, bi)| rep.bond(bi).order);
    let is_chain = rep
        .neighbors(AtomIdx(attach))
        .filter(|(n, _)| frag.contains(&n.0) && n.0 != attach)
        .count()
        == 1;
    if !is_chain {
        let (atoms, bonds, att) = extract_subgraph(rep, frag);
        let (ra, rb, ratt) = reorder_attachment_first(atoms, bonds, att);
        return vec![FloatingDef {
            atoms: ra,
            bonds: rb,
            attachment: ratt,
            attach_order: scaffold_order,
            split: false,
        }];
    }
    let rest: Vec<u32> = frag.iter().copied().filter(|&a| a != attach).collect();
    let split_order = rep
        .neighbors(AtomIdx(attach))
        .find(|(n, _)| rest.contains(&n.0))
        .map_or(BondOrder::Single, |(_, bi)| rep.bond(bi).order);
    let (a1, _, _) = extract_subgraph(rep, std::slice::from_ref(&attach));
    let (a2, b2, att2) = extract_subgraph(rep, &rest);
    let (ra2, rb2, ratt2) = reorder_attachment_first(a2, b2, att2);
    vec![
        FloatingDef {
            atoms: a1,
            bonds: Vec::new(),
            attachment: 0,
            attach_order: scaffold_order,
            split: false,
        },
        FloatingDef {
            atoms: ra2,
            bonds: rb2,
            attachment: ratt2,
            attach_order: split_order,
            split: true,
        },
    ]
}

/// Equivalent scaffold attachment positions for one floating component.
pub(crate) fn equiv_positions(
    rep: &Molecule,
    comp: &[u32],
    group: &[Molecule],
    scaffold_q: &QueryMolecule,
) -> Vec<usize> {
    let target_multiset = (!comp.is_empty()).then(|| multiset(comp, rep));
    let mut seen_pos = vec![false; scaffold_q.atoms.len()];
    let mut positions = Vec::new();
    let mut scratch = Vec::new();
    for mol in group {
        let hits = find_matches(scaffold_q, mol);
        let mut best_new: Option<Vec<usize>> = None;
        for hit in hits {
            let rev = reverse_match_table(&hit, mol);
            let unf = unmatched_atoms(&matched_mask(&hit, mol));
            let mut candidate = Vec::new();
            for mc in components(&unf, mol) {
                if let Some(target) = &target_multiset
                    && !matches_multiset(&mc, mol, target, &mut scratch)
                {
                    continue;
                }
                for &u in &mc {
                    for (nbr, _) in mol.neighbors(AtomIdx(u)) {
                        if let Some(qi) = rev.get(atom_usize(nbr)).copied().flatten()
                            && seen_pos.get(qi) == Some(&false)
                            && !candidate.contains(&qi)
                        {
                            candidate.push(qi);
                        }
                    }
                }
            }
            candidate.sort_unstable();
            let take = best_new
                .as_ref()
                .is_none_or(|prev| (candidate.len(), &candidate) < (prev.len(), prev));
            if take {
                best_new = Some(candidate);
            }
        }
        if let Some(new_positions) = best_new {
            for qi in new_positions {
                if let Some(slot) = seen_pos.get_mut(qi)
                    && !*slot
                {
                    *slot = true;
                    positions.push(qi);
                }
            }
        }
    }
    positions.sort_unstable();
    if positions.is_empty() {
        positions = (0..scaffold_q.atoms.len()).collect();
    }
    positions
}

pub(crate) fn split_floating_and_recovered(
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
            recovered.extend_from_slice(comp);
        }
    }
    (floating, recovered)
}

pub(crate) fn boundary_edges(comp: &[u32], matched: &[bool], mol: &Molecule) -> usize {
    let mut count = 0;
    for &a in comp {
        for (nbr, _) in mol.neighbors(AtomIdx(a)) {
            if !comp.contains(&nbr.0) && is_matched(matched, nbr.0) {
                count += 1;
            }
        }
    }
    count
}

pub(crate) fn is_matched(matched: &[bool], a: u32) -> bool {
    matched.get(usize::try_from(a).unwrap_or(usize::MAX)) == Some(&true)
}

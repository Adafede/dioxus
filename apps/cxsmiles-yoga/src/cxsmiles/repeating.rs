// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

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

use super::graph::{components, matched_mask, molecule_to_query, unmatched_atoms, unmatched_count};
use super::roundtrip::{enumerate_repeating, roundtrip_coverage};
use super::types::{Confidence, Construct, CxError, CxResult, CxResult_, RepeatUnit};

pub fn build_repeating(group: &[Molecule]) -> CxResult_ {
    let mut ordered = group.to_vec();
    ordered.sort_by_key(Molecule::atom_count);
    let shortest = &ordered[0];
    let longest = ordered.last().unwrap();

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

    let enumerated = enumerate_repeating(&scaffold, &repeat_unit);
    let (_, cov) = roundtrip_coverage(&enumerated, group);
    let frac = cov.fraction();

    Ok(CxResult {
        cx_smiles: cx,
        base_smiles: scaffold_smiles.clone(),
        construct: Construct::Repeating,
        scaffold_smiles,
        floating: Vec::new(),
        repeating: Some(repeat_unit),
        confidence: Confidence {
            coverage: cov,
            clean: frac >= 1.0,
        },
        enumerated,
    })
}

/// One copy of the recurring fragment, taken from the extra atoms when the
/// shortest scaffold is aligned into the longest input.
pub fn repeat_pattern(
    _shortest: &Molecule,
    longest: &Molecule,
    unit_size: usize,
    scaffold_q: &QueryMolecule,
) -> Result<Vec<u32>, CxError> {
    let hits = find_matches(scaffold_q, longest);
    let hit = hits
        .iter()
        .min_by_key(|h| unmatched_count(h, longest))
        .ok_or_else(|| CxError("no MCS match of shortest into longest".into()))?;
    let matched = matched_mask(hit, longest);
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
pub fn unit_multiset(pattern: &[u32], longest: &Molecule, unit_size: usize) -> Vec<u8> {
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

/// Locate the in-scaffold copy of the repeat unit: the internal connected
/// subgraph of `unit_size` atoms whose element multiset matches `target`.
pub fn locate_repeat_in_scaffold(
    scaffold: &Molecule,
    target: &[u8],
    unit_size: usize,
) -> Result<Vec<usize>, CxError> {
    let n = scaffold.atom_count();
    let mut best: Option<Vec<u32>> = None;
    for start in 0..n as u32 {
        let mut stack: Vec<Vec<u32>> = vec![vec![start]];
        while let Some(frag) = stack.pop() {
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
            let cur = *frag.last().unwrap();
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

pub fn multiset(atoms: &[u32], mol: &Molecule) -> Vec<u8> {
    let mut v: Vec<u8> = atoms
        .iter()
        .map(|&i| mol.atom(AtomIdx(i)).element.atomic_number())
        .collect();
    v.sort_unstable();
    v
}

pub fn is_internal(atoms: &[u32], mol: &Molecule) -> bool {
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

pub fn center_dist(atoms: &[u32], n: usize) -> i64 {
    let center = (n as i64 - 1) / 2;
    atoms.iter().map(|&a| a as i64 - center).sum::<i64>().abs()
}

pub fn gcd_vec(xs: &[usize]) -> usize {
    xs.iter().copied().fold(0, gcd)
}

pub fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// Splice `n` copies of the repeat unit between its two external anchors.
pub fn splice_repeat(scaffold: &Molecule, repeat_atoms: &[usize], n: usize) -> Molecule {
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
    let pos_in_unit = |x: u32| -> usize { unit.iter().position(|&v| v == x).unwrap() };
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
    copy_atoms.push(unit.iter().map(|&a| sc[a as usize]).collect());
    for _ in 1..n {
        let block: Vec<AtomIdx> = unit
            .iter()
            .map(|&a| b.add_atom(scaffold.atom(AtomIdx(a)).clone()))
            .collect();
        for (a1, a2, order) in &unit_internal {
            let _ = b.add_bond(block[pos_in_unit(*a1)], block[pos_in_unit(*a2)], *order);
        }
        copy_atoms.push(block);
    }
    // Scaffold bonds, omitting the two anchor bonds.
    for (_, be) in scaffold.bonds() {
        let (a, c) = (be.atom1.0, be.atom2.0);
        let omit = (a == anchor_a && c == ep_a)
            || (c == anchor_a && a == ep_a)
            || (a == anchor_b && c == ep_b)
            || (c == anchor_b && a == ep_b);
        if !omit {
            let _ = b.add_bond(sc[a as usize], sc[c as usize], be.order);
        }
    }
    // Splice bonds.
    let ra0 = copy_atoms[0][rpos_a];
    let _ = b.add_bond(sc[anchor_a as usize], ra0, BondOrder::Single);
    for k in 0..(n - 1) {
        let rb_k = copy_atoms[k][rpos_b];
        let ra_k1 = copy_atoms[k + 1][rpos_a];
        let _ = b.add_bond(rb_k, ra_k1, BondOrder::Single);
    }
    let rb_last = copy_atoms[n - 1][rpos_b];
    let _ = b.add_bond(rb_last, sc[anchor_b as usize], BondOrder::Single);
    b.build()
}

/// First atom of `unit` that is bonded to `anchor` in `scaffold`.
pub fn endpoint_for(scaffold: &Molecule, unit: &[u32], anchor: u32) -> u32 {
    let uset: HashSet<u32> = unit.iter().copied().collect();
    for (nbr, _) in scaffold.neighbors(AtomIdx(anchor)) {
        if uset.contains(&nbr.0) {
            return nbr.0;
        }
    }
    unit[0]
}

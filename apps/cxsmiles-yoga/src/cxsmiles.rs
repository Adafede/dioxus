//! CX-SMILES generation core (UI-free, unit-testable).
//!
//! Pipeline (`generate_cxsmiles`): parse → cluster → maximum-common
//! substructure → diff & classify → serialise → round-trip confidence.
//!
//! `chematic::cx` only handles atom-level CX fields (labels/props/radicals/
//! zero-bonds/wavy bonds) — **not** `m:` (positional equivalence) or
//! `Sg:n:` (repeating units). Both are hand-rolled here, along with their
//! expansion logic used for round-tripping.
//!
//! Note: chematic's SMILES *writer* does not emit `*` for wildcard atoms (it
//! writes the placeholder element `C`), so the CX base string is assembled by
//! hand: `scaffold_smiles` followed by `.[*]<frag>` for each floating group.
//! This keeps atom indices fully under our control.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
)]

use chematic::core::{Atom, AtomIdx, BondOrder, Molecule, MoleculeBuilder};
use chematic::fp::{BitVec2048, ecfp4};
use chematic::smarts::{
    AtomPrimitive, AtomQuery, BondPrimitive, BondQuery, QueryAtom, QueryBond, QueryMolecule,
    find_matches, find_mcs,
};
use rustc_hash::FxHashMap;
use std::collections::{HashMap, HashSet};

pub use chematic::smiles::{canonical_smiles, parse, write};

// ---------------------------------------------------------------------------
// Public result types
// ---------------------------------------------------------------------------

/// High-level classification of a generated CX-SMILES construct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Construct {
    /// A pendant group moves to equivalent attachment points (`m:` blocks).
    Positional,
    /// The inputs differ only in the repeat count of a sub-unit (`Sg:n:`).
    Repeating,
    /// The set could not be reconciled into one clean construct (e.g.
    /// constitutional isomers). A best-effort result is still produced.
    BestEffort,
}

/// Round-trip report: how many original inputs re-appear after expanding CX-SMILES.
#[derive(Debug, Clone)]
pub struct Coverage {
    pub covered: usize,
    pub total: usize,
}

impl Coverage {
    #[must_use]
    pub fn fraction(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.covered as f64 / self.total as f64
        }
    }
}

/// Confidence descriptor surfaced to the UI.
#[derive(Debug, Clone)]
pub struct Confidence {
    pub coverage: Coverage,
    /// `true` when the fit was clean: 100% round-trip coverage.
    pub clean: bool,
}

/// A floating (pendant) group, described for serialisation and expansion.
#[derive(Debug, Clone)]
pub struct FloatingPart {
    /// Atom index of the `*` in the *base* SMILES.
    pub star_idx: usize,
    /// Equivalent base atom indices where this group may attach.
    pub equiv: Vec<usize>,
    /// Raw SMILES of the floating fragment (e.g. `[*O`, `[*]C(=O)C`).
    pub fragment_smiles: String,
    /// `true` when emitted as the second half of a split (double-`m`) group.
    pub split: bool,
}

/// A repeating unit marked with `Sg:n:`.
#[derive(Debug, Clone)]
pub struct RepeatUnit {
    /// Atom indices (into the base SMILES) of one copy of the repeat unit.
    pub atoms: Vec<usize>,
    pub min: usize,
    pub max: usize,
}

/// The result of generating a CX-SMILES from a SMILES list.
#[derive(Debug, Clone)]
pub struct CxResult {
    pub cx_smiles: String,
    pub base_smiles: String,
    pub construct: Construct,
    pub scaffold_smiles: String,
    pub floating: Vec<FloatingPart>,
    pub repeating: Option<RepeatUnit>,
    pub confidence: Confidence,
    /// All distinct molecules enumerated from the generated CX-SMILES
    /// (canonical SMILES) — used by the UI to depict candidates.
    pub enumerated: Vec<String>,
}

#[derive(Debug)]
pub struct CxError(pub String);
impl std::fmt::Display for CxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for CxError {}

pub type CxResult_ = Result<CxResult, CxError>;

/// Minimum ECFP4/Tanimoto similarity for two structures to share a coherent
/// candidate group (single-linkage). Biphenyl variants pairwise reach ≥0.37.
const CLUSTER_TANIMOTO: f64 = 0.3;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Generate a CX-SMILES from a list of related SMILES (one per entry).
///
/// # Panics
///
/// Panics if clustering discards every group (unreachable: at least one
/// cluster is always retained).
///
/// # Errors
///
/// Returns a [`CxError`] if any input fails to parse as SMILES.
#[allow(clippy::module_name_repetitions)]
pub fn generate_cxsmiles(smiles: &[String]) -> CxResult_ {
    let mols = parse_list(smiles)?;
    if mols.is_empty() {
        return Err(CxError("no parseable SMILES in input".into()));
    }
    if mols.len() == 1 {
        let mol = &mols[0];
        let smi = canonical_smiles(mol);
        return Ok(CxResult {
            cx_smiles: smi.clone(),
            base_smiles: smi.clone(),
            construct: Construct::BestEffort,
            scaffold_smiles: smi.clone(),
            floating: Vec::new(),
            repeating: None,
            confidence: Confidence {
                coverage: Coverage {
                    covered: 1,
                    total: 1,
                },
                clean: true,
            },
            enumerated: vec![smi],
        });
    }

    let clusters = cluster(&mols, CLUSTER_TANIMOTO);
    let group: Vec<Molecule> = if clusters.iter().map(Vec::len).max().unwrap_or(0) >= 2 {
        clusters.into_iter().max_by_key(Vec::len).unwrap()
    } else {
        mols
    };

    let counts: Vec<usize> = group.iter().map(Molecule::atom_count).collect();
    if counts.iter().all(|c| *c == counts[0]) {
        build_positional(&group)
    } else {
        build_repeating(&group)
    }
}

// ---------------------------------------------------------------------------
// Parsing & clustering
// ---------------------------------------------------------------------------

fn parse_list(smiles: &[String]) -> Result<Vec<Molecule>, CxError> {
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
fn cluster(mols: &[Molecule], threshold: f64) -> Vec<Vec<Molecule>> {
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

// ---------------------------------------------------------------------------
// Graph helpers
// ---------------------------------------------------------------------------

type Match = FxHashMap<usize, AtomIdx>;

/// `Molecule` → `QueryMolecule` (element + bond-order). Atom `i` of the query
/// corresponds to atom `i` of the molecule.
fn molecule_to_query(mol: &Molecule) -> QueryMolecule {
    let n = mol.atom_count();
    let atoms: Vec<QueryAtom> = (0..n as u32)
        .map(|i| {
            let a = mol.atom(AtomIdx(i));
            QueryAtom {
                query: AtomQuery::Primitive(AtomPrimitive::AtomicNum(a.element.atomic_number())),
                atom_map: a.atom_map,
            }
        })
        .collect();
    let mut bonds: Vec<QueryBond> = Vec::new();
    let mut adj: Vec<Vec<(usize, usize)>> = vec![Vec::new(); n];
    for (_, be) in mol.bonds() {
        let a = be.atom1.0 as usize;
        let b = be.atom2.0 as usize;
        let qi = bonds.len();
        bonds.push(QueryBond {
            atom1: a,
            atom2: b,
            query: BondQuery::Primitive(bond_to_primitive(be.order)),
        });
        adj[a].push((qi, b));
        adj[b].push((qi, a));
    }
    QueryMolecule { atoms, bonds, adj }
}

const fn bond_to_primitive(order: BondOrder) -> BondPrimitive {
    match order {
        BondOrder::Single => BondPrimitive::Single,
        BondOrder::Double => BondPrimitive::Double,
        BondOrder::Triple => BondPrimitive::Triple,
        BondOrder::Aromatic => BondPrimitive::Aromatic,
        _ => BondPrimitive::Any,
    }
}

/// Build a molecule from the subset of `mol`'s atoms and the bonds between them.
fn subgraph(mol: &Molecule, keep: &[bool]) -> Molecule {
    let n = mol.atom_count();
    let mut b = MoleculeBuilder::new();
    let mut map: Vec<Option<AtomIdx>> = vec![None; n];
    for i in 0..n as u32 {
        if keep[i as usize] {
            map[i as usize] = Some(b.add_atom(mol.atom(AtomIdx(i)).clone()));
        }
    }
    for (_, be) in mol.bonds() {
        let (a, c) = (be.atom1.0 as usize, be.atom2.0 as usize);
        if keep[a] && keep[c] {
            let _ = b.add_bond(map[a].unwrap(), map[c].unwrap(), be.order);
        }
    }
    b.build()
}

/// Connected components of `atoms` (u32 indices) using `mol`'s internal edges.
fn components(atoms: &[u32], mol: &Molecule) -> Vec<Vec<u32>> {
    let set: HashSet<u32> = atoms.iter().copied().collect();
    let mut seen: HashSet<u32> = HashSet::new();
    let mut comps: Vec<Vec<u32>> = Vec::new();
    for &start in atoms {
        if seen.contains(&start) {
            continue;
        }
        let mut comp: Vec<u32> = Vec::new();
        let mut stack = vec![start];
        seen.insert(start);
        while let Some(cur) = stack.pop() {
            comp.push(cur);
            for (nbr, _) in mol.neighbors(AtomIdx(cur)) {
                if set.contains(&nbr.0) && !seen.contains(&nbr.0) {
                    seen.insert(nbr.0);
                    stack.push(nbr.0);
                }
            }
        }
        comps.push(comp);
    }
    comps
}

/// One embedding of a query into `mol`, preferring the one that recovers the
/// most scaffold atoms (fewest unmatched).
fn best_match(q: &QueryMolecule, mol: &Molecule) -> Result<Match, CxError> {
    let hits = find_matches(q, mol);
    if hits.is_empty() {
        return Err(CxError("no match of query into molecule".into()));
    }
    Ok(hits
        .iter()
        .min_by_key(|h| mol.atom_count() - h.len())
        .cloned()
        .unwrap())
}

fn unmatched_count(h: &Match, mol: &Molecule) -> usize {
    mol.atom_count() - h.len()
}

fn matched_mask(h: &Match, mol: &Molecule) -> Vec<bool> {
    let mut m = vec![false; mol.atom_count()];
    for &a in h.values() {
        m[a.0 as usize] = true;
    }
    m
}

fn unmatched_atoms(matched: &[bool]) -> Vec<u32> {
    (0..matched.len() as u32)
        .filter(|i| !matched[*i as usize])
        .collect()
}

// ---------------------------------------------------------------------------
// Positional (m:) construction
// ---------------------------------------------------------------------------

/// A floating fragment ready for serialisation and expansion.
#[derive(Clone)]
struct FloatingDef {
    atoms: Vec<Atom>,
    bonds: Vec<(usize, usize, BondOrder)>,
    attachment: usize, // index into `atoms`, bonded to the scaffold/* target
    split: bool,       // double-m variant
}

/// The attachment target of a floating group.
enum Target {
    /// Attaches to one of the equivalent scaffold positions.
    Variable(Vec<usize>),
    /// Attaches to another group's attachment atom (index into `defs`).
    Fixed(usize),
}

fn build_positional(group: &[Molecule]) -> CxResult_ {
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
            targets.push(Target::Fixed(g1)); // group2 attaches to group1's O
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

// ---------------------------------------------------------------------------
// Repeating (Sg:n:) construction
// ---------------------------------------------------------------------------

fn build_repeating(group: &[Molecule]) -> CxResult_ {
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
fn repeat_pattern(
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
fn unit_multiset(pattern: &[u32], longest: &Molecule, unit_size: usize) -> Vec<u8> {
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
fn locate_repeat_in_scaffold(
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

fn multiset(atoms: &[u32], mol: &Molecule) -> Vec<u8> {
    let mut v: Vec<u8> = atoms
        .iter()
        .map(|&i| mol.atom(AtomIdx(i)).element.atomic_number())
        .collect();
    v.sort_unstable();
    v
}

fn is_internal(atoms: &[u32], mol: &Molecule) -> bool {
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

fn center_dist(atoms: &[u32], n: usize) -> i64 {
    let center = (n as i64 - 1) / 2;
    atoms.iter().map(|&a| a as i64 - center).sum::<i64>().abs()
}

fn gcd_vec(xs: &[usize]) -> usize {
    xs.iter().copied().fold(0, gcd)
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

// ---------------------------------------------------------------------------
// Round-trip enumeration
// ---------------------------------------------------------------------------

fn roundtrip_coverage(enumerated: &[String], group: &[Molecule]) -> (usize, Coverage) {
    let canon: Vec<String> = group.iter().map(canonical_smiles).collect();
    let mut covered = 0;
    for c in &canon {
        if enumerated.iter().any(|e| e == c) {
            covered += 1;
        }
    }
    (
        covered,
        Coverage {
            covered,
            total: group.len(),
        },
    )
}

/// Enumerate every distinct molecule implied by a positional CX-SMILES.
fn enumerate(scaffold: &Molecule, defs: &[FloatingDef], targets: &[Target]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let var: Vec<usize> = targets
        .iter()
        .enumerate()
        .filter(|(_, t)| matches!(t, Target::Variable(_)))
        .map(|(i, _)| i)
        .collect();
    if var.is_empty() {
        out.push(canonical_smiles(&build_one(scaffold, defs, targets, &[])));
        return dedup_sort(out);
    }
    let ranges: Vec<&Vec<usize>> = var
        .iter()
        .map(|&i| match &targets[i] {
            Target::Variable(p) => p,
            Target::Fixed(_) => unreachable!(),
        })
        .collect();
    let mut combo: Vec<usize> = vec![0; var.len()];
    loop {
        out.push(canonical_smiles(&build_one(
            scaffold, defs, targets, &combo,
        )));
        if !next_combo(&mut combo, &ranges) {
            break;
        }
    }
    dedup_sort(out)
}

fn next_combo(combo: &mut [usize], ranges: &[&Vec<usize>]) -> bool {
    for i in (0..combo.len()).rev() {
        combo[i] += 1;
        if combo[i] < ranges[i].len() {
            return true;
        }
        combo[i] = 0;
    }
    false
}

/// Build one concrete molecule for a given variable-position choice.
fn build_one(
    scaffold: &Molecule,
    defs: &[FloatingDef],
    targets: &[Target],
    combo: &[usize],
) -> Molecule {
    let mut b = MoleculeBuilder::new();
    let mut sc: Vec<AtomIdx> = Vec::with_capacity(scaffold.atom_count());
    for (_, atom) in scaffold.atoms() {
        sc.push(b.add_atom(atom.clone()));
    }
    for (_, be) in scaffold.bonds() {
        let _ = b.add_bond(sc[be.atom1.0 as usize], sc[be.atom2.0 as usize], be.order);
    }
    let mut part_attach: Vec<AtomIdx> = Vec::with_capacity(defs.len());
    for def in defs {
        let mut pmap: Vec<AtomIdx> = Vec::with_capacity(def.atoms.len());
        for atom in &def.atoms {
            pmap.push(b.add_atom(atom.clone()));
        }
        for (a1, a2, order) in &def.bonds {
            let _ = b.add_bond(pmap[*a1], pmap[*a2], *order);
        }
        part_attach.push(pmap[def.attachment]);
    }
    let mut var = 0usize;
    for (pi, _def) in defs.iter().enumerate() {
        match &targets[pi] {
            Target::Variable(positions) => {
                let pos = positions[combo[var]];
                var += 1;
                let _ = b.add_bond(part_attach[pi], sc[pos], BondOrder::Single);
            }
            Target::Fixed(other) => {
                let _ = b.add_bond(part_attach[pi], part_attach[*other], BondOrder::Single);
            }
        }
    }
    b.build()
}

/// Enumerate the repeating case: scaffold + (count-1) extra copies of the unit.
fn enumerate_repeating(scaffold: &Molecule, unit: &RepeatUnit) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for n in unit.min..=unit.max {
        out.push(canonical_smiles(&splice_repeat(scaffold, &unit.atoms, n)));
    }
    dedup_sort(out)
}

/// Splice `n` copies of the repeat unit between its two external anchors.
fn splice_repeat(scaffold: &Molecule, repeat_atoms: &[usize], n: usize) -> Molecule {
    if n <= 1 {
        return scaffold.clone();
    }
    let set: HashSet<u32> = repeat_atoms
        .iter()
        .copied()
        .map(|a| a as u32)
        .collect();
    let unit: Vec<u32> = repeat_atoms.iter().map(|&a| a as u32).collect();
    let anchors: Vec<u32> = repeat_atoms
        .iter()
        .flat_map(|&a| {
            scaffold
                .neighbors(AtomIdx(a as u32))
                .filter(|(nb, _)| !set.contains(&nb.0))
                .map(|(nb, _)| nb.0)
        })
        .collect::<std::collections::HashSet<_>>()
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

fn endpoint_for(scaffold: &Molecule, unit: &[u32], anchor: u32) -> u32 {
    let uset: HashSet<u32> = unit.iter().copied().collect();
    for (nbr, _) in scaffold.neighbors(AtomIdx(anchor)) {
        if uset.contains(&nbr.0) {
            return nbr.0;
        }
    }
    unit[0]
}

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

fn dedup_sort(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v.dedup();
    v
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(s: &str) -> Vec<String> {
        s.lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    }

    fn assert_roundtrip(result: &CxResult, group: &[Molecule]) {
        let canon: Vec<String> = group.iter().map(canonical_smiles).collect();
        for c in &canon {
            assert!(
                result.enumerated.iter().any(|e| e == c),
                "input canonical SMILES {c} is NOT covered by round-trip; enumerated={:?}",
                result.enumerated
            );
        }
    }

    #[test]
    fn parse_canonical_write_roundtrip() {
        let m = parse("Clc1ccccc1-c2ccccc2").unwrap();
        assert_eq!(canonical_smiles(&m), "c1ccc(-c2ccccc2)c(c1)Cl");
        assert_eq!(write(&m), "Clc1ccccc1-c2ccccc2");
    }

    #[test]
    fn positional_biphenyl_cl() {
        // Reference (RDKit-indexed): C1=CC=CC=C1C2=...Cl* |m:13:0.2.3|
        let input = "Clc1ccccc1-c2ccccc2\nClc1cccc(-c2ccccc2)c1\nClc1ccc(-c2ccccc2)cc1";
        let mols = parse_list(&lines(input)).unwrap();
        let r = generate_cxsmiles(&lines(input)).unwrap();
        assert_eq!(r.construct, Construct::Positional);
        assert_roundtrip(&r, &mols);
        assert!(r.confidence.clean);
        assert_eq!(r.scaffold_smiles, "c1ccccc1-c2ccccc2");
        assert_eq!(r.floating.len(), 1);
        assert_eq!(r.floating[0].equiv.len(), 3);
        assert!(r.cx_smiles.contains("m:"));
        assert_eq!(r.cx_smiles.matches("m:").count(), 1);
    }

    #[test]
    fn positional_omf_moving() {
        // Reference: OC1=C(O)C=C(O)C=C1.C* |m:10:0.3.6|
        let input = "COc1c(O)cc(O)cc1\nOc1c(OC)cc(O)cc1\nOc1c(O)cc(OC)cc1";
        let mols = parse_list(&lines(input)).unwrap();
        let r = generate_cxsmiles(&lines(input)).unwrap();
        assert_eq!(r.construct, Construct::Positional);
        assert_roundtrip(&r, &mols);
        assert!(r.confidence.clean);
        assert_eq!(r.floating.len(), 1);
        assert_eq!(r.floating[0].equiv.len(), 3);
    }

    #[test]
    fn positional_acetyl_double_m() {
        // Acetate (-OCOCH3) moves on a triol scaffold; emitted as two m: blocks
        // (O* + C(=O)(C)*) — the "double m: block variant".
        let input =
            "CC(=O)Oc1ccccc1-c2ccccc2\nCC(=O)Oc1cccc(-c2ccccc2)c1\nCC(=O)Oc1ccc(-c2ccccc2)cc1";
        let mols = parse_list(&lines(input)).unwrap();
        let r = generate_cxsmiles(&lines(input)).unwrap();
        assert_eq!(r.construct, Construct::Positional);
        assert_roundtrip(&r, &mols);
        assert!(r.confidence.clean);
        assert_eq!(r.floating.len(), 2, "floating={:?}", r.floating);
        let o_star = r.floating.iter().find(|f| !f.split).expect("O* group");
        let acetyl = r
            .floating
            .iter()
            .find(|f| f.split)
            .expect("C(=O)(C)* group");
        assert!(o_star.fragment_smiles.contains('O') && o_star.fragment_smiles.contains('*'));
        assert!(
            acetyl.fragment_smiles.contains('C')
                && acetyl.fragment_smiles.contains('O')
                && acetyl.fragment_smiles.contains('*')
        );
        assert_eq!(o_star.equiv.len(), 3);
        assert_eq!(r.cx_smiles.matches("m:").count(), 2);
    }

    #[test]
    fn repeating_pfas() {
        // Reference: OC(=O)C(F)(F)C(F)F |Sg:n:3,4,5:n:ht|
        let input =
            "OC(=O)C(F)(F)C(F)F\nOC(=O)C(F)(F)C(F)(F)C(F)F\nOC(=O)C(F)(F)C(F)(F)C(F)(F)C(F)F";
        let mols = parse_list(&lines(input)).unwrap();
        let r = generate_cxsmiles(&lines(input)).unwrap();
        assert_eq!(r.construct, Construct::Repeating);
        assert_roundtrip(&r, &mols);
        assert!(r.confidence.clean);
        assert_eq!(r.scaffold_smiles, "OC(=O)C(F)(F)C(F)F");
        assert_eq!(r.cx_smiles, "OC(=O)C(F)(F)C(F)F |Sg:n:3,4,5:n:ht|");
    }

    #[test]
    fn repeating_alkyl() {
        // Reference: CCCCCCC |Sg:n:3:n:ht|
        let input = "CCCCCCC\nCCCCCCCC\nCCCCCCCCC";
        let mols = parse_list(&lines(input)).unwrap();
        let r = generate_cxsmiles(&lines(input)).unwrap();
        assert_eq!(r.construct, Construct::Repeating);
        assert_roundtrip(&r, &mols);
        assert_eq!(r.scaffold_smiles, "CCCCCCC");
        assert!(r.cx_smiles.contains("Sg:n:3:n:ht"));
    }

    #[test]
    fn best_effort_constitutional_isomers() {
        // Six constitutional isomers of C4H10O with no clean shared moving
        // group -> best-effort, sub-100% round-trip coverage.
        let input = "\
CC(C)(C)O
CC(C)CO
CC(O)CC
OCCCC
CCOCC
COCCC";
        let r = generate_cxsmiles(&lines(input)).unwrap();
        assert!(
            r.confidence.coverage.fraction() < 1.0,
            "expected sub-100% coverage, got {:?}; cx={}",
            r.confidence,
            r.cx_smiles
        );
    }
}

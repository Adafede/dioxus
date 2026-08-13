//! Local 2D depiction — no remote rendering service required.
//!
//! Wraps `chematic::depict::{depict_svg, depict_svg_highlighted}`, which compute
//! a 2D layout and emit an inline SVG string. Used by the results panel to draw
//! the shared scaffold and every enumerated candidate.

use chematic::core::{AtomIdx, BondIdx, Molecule};
use chematic::depict::{depict_svg, depict_svg_highlighted};
use chematic::smiles::parse;
use std::collections::HashSet;

/// Render a SMILES string to an inline SVG, parsing it with chematic.
#[must_use]
pub fn render_smiles_svg(smiles: &str) -> String {
    parse(smiles).map_or_else(|_| empty_svg(), |mol| depict_svg(&mol))
}

/// Render a `Molecule` to an inline SVG.
#[must_use]
pub fn render_molecule_svg(mol: &Molecule) -> String {
    depict_svg(mol)
}

/// Render a SMILES string with the given `atom_indices` (0-based, in the
/// molecule's write order) highlighted.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn render_smiles_svg_highlighted(smiles: &str, atom_indices: &[usize]) -> String {
    let Ok(mol) = parse(smiles) else { return empty_svg() };
    let highlight: HashSet<AtomIdx> = atom_indices.iter().map(|&i| AtomIdx(i as u32)).collect();
    let bonds: HashSet<BondIdx> = HashSet::new();
    depict_svg_highlighted(&mol, &highlight, &bonds)
}

fn empty_svg() -> String {
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 220 60"><text x="50%" y="34" text-anchor="middle" fill="#94a3b8" font-size="13" font-family="ui-system,system-ui,sans-serif">could not render</text></svg>"##.to_string()
}

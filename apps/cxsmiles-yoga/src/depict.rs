// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Local 2D depiction — no remote rendering service required.
//!
//! Wraps `chematic::depict::{depict_svg, depict_svg_highlighted}`, which compute
//! a 2D layout and emit an inline SVG string. Used by the results panel to draw
//! the shared scaffold and every enumerated candidate.

use std::collections::HashSet;

use chematic::core::{AtomIdx, BondIdx};
use chematic::depict::{depict_svg, depict_svg_highlighted};
use chematic::smiles::parse;

/// Render a SMILES string to an inline SVG, parsing it with chematic.
#[must_use]
pub fn render_smiles_svg(smiles: &str) -> String {
    render_smiles_svg_highlighted(smiles, &[])
}

/// Render a SMILES string to an inline SVG with `highlight_atoms` (and any
/// bonds between two highlighted atoms) highlighted in the 2D layout.
#[must_use]
pub fn render_smiles_svg_highlighted(smiles: &str, highlight_atoms: &[usize]) -> String {
    parse(smiles).map_or_else(
        |_| empty_svg(),
        |mol| {
            if highlight_atoms.is_empty() {
                return depict_svg(&mol);
            }
            let atoms: HashSet<AtomIdx> = highlight_atoms
                .iter()
                .filter_map(|&i| u32::try_from(i).ok().map(AtomIdx))
                .collect();
            let bonds: HashSet<BondIdx> = mol
                .bonds()
                .filter(|(_, b)| atoms.contains(&b.atom1) && atoms.contains(&b.atom2))
                .map(|(idx, _)| idx)
                .collect();
            depict_svg_highlighted(&mol, &atoms, &bonds)
        },
    )
}

fn empty_svg() -> String {
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 220 60"><text x="50%" y="34" text-anchor="middle" fill="#94a3b8" font-size="13" font-family="ui-system,system-ui,sans-serif">could not render</text></svg>"##.to_string()
}

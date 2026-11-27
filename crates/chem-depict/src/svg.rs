// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Panic-guarded 2D SVG depiction wrappers around `chematic::depict`.

use std::collections::HashSet;
use std::panic::{AssertUnwindSafe, catch_unwind};

use chematic::core::{AtomIdx, BondIdx, Molecule};
use chematic::depict::{depict_svg, depict_svg_highlighted};

use crate::parse::parse_smiles_safe;

pub(crate) const MAX_DEPICT_ATOMS: usize = 512;

/// Standard fallback SVG rendered when depiction fails (`220x60`).
#[must_use]
pub(crate) fn empty_svg() -> String {
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 220 60"><text x="50%" y="34" text-anchor="middle" fill="#94a3b8" font-size="13" font-family="ui-system,system-ui,sans-serif">could not render</text></svg>"##.to_string()
}

/// Compact fallback SVG rendered for small gallery cards (`80x24`).
#[must_use]
pub fn empty_compact_svg() -> String {
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 80 24"><text x="40" y="15" fill="#94a3b8" text-anchor="middle" font-family="system-ui" font-size="10">no structure</text></svg>"##.to_string()
}

fn validate_svg_or(
    candidate: Result<String, Box<dyn std::any::Any + Send>>,
    fallback: fn() -> String,
) -> String {
    candidate
        .ok()
        .filter(|svg| {
            let trimmed = svg.trim();
            trimmed.starts_with("<svg") && trimmed.ends_with("</svg>")
        })
        .unwrap_or_else(fallback)
}

fn render_molecule_guarded(
    molecule: &Molecule,
    fallback: fn() -> String,
    render: impl FnOnce(&Molecule) -> String,
) -> String {
    if molecule.atom_count() == 0 || molecule.atom_count() > MAX_DEPICT_ATOMS {
        return fallback();
    }
    let rendered = catch_unwind(AssertUnwindSafe(|| render(molecule)));
    validate_svg_or(rendered, fallback)
}

/// Renders an already-parsed [`Molecule`] into a compact inline SVG, falling back
/// to [`empty_compact_svg`] on empty/oversized graphs, layout panic, or invalid output.
#[must_use]
pub fn render_molecule_svg_compact(molecule: &Molecule) -> String {
    render_molecule_guarded(molecule, empty_compact_svg, depict_svg)
}

/// Parses `smiles` and renders it to an inline SVG, falling back to `empty_svg`.
#[must_use]
pub fn render_smiles_svg(smiles: &str) -> String {
    render_smiles_svg_highlighted(smiles, &[])
}

/// Parses `smiles` and renders a compact inline SVG, falling back to [`empty_compact_svg`].
#[must_use]
pub fn render_smiles_svg_compact(smiles: &str) -> String {
    let Ok(mol) = parse_smiles_safe(smiles) else {
        return empty_compact_svg();
    };
    render_molecule_svg_compact(&mol)
}

/// Parses `smiles` and renders an inline SVG with `highlight_atoms` (and bonds
/// connecting two highlighted atoms) emphasized, falling back to `empty_svg`.
#[must_use]
pub fn render_smiles_svg_highlighted(smiles: &str, highlight_atoms: &[usize]) -> String {
    let Ok(mol) = parse_smiles_safe(smiles) else {
        return empty_svg();
    };
    render_molecule_guarded(&mol, empty_svg, |mol| {
        if highlight_atoms.is_empty() {
            return depict_svg(mol);
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
        depict_svg_highlighted(mol, &atoms, &bonds)
    })
}

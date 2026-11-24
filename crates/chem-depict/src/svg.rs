// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Panic-guarded 2D SVG depiction wrappers around `chematic::depict`.

use std::collections::HashSet;
use std::panic::{AssertUnwindSafe, catch_unwind};

use chematic::core::{AtomIdx, BondIdx, Molecule};
use chematic::depict::{depict_svg, depict_svg_highlighted};

use crate::parse::parse_smiles_safe;

/// Standard fallback SVG rendered when depiction fails (`220x60`).
#[must_use]
pub(crate) fn empty_svg() -> String {
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 220 60"><text x="50%" y="34" text-anchor="middle" fill="#94a3b8" font-size="13" font-family="ui-system,system-ui,sans-serif">could not render</text></svg>"##.to_string()
}

/// Compact fallback SVG rendered for small gallery cards (`40x20`).
#[must_use]
pub fn empty_compact_svg() -> String {
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40 20"><text x="20" y="12" fill="#94a3b8" text-anchor="middle" font-family="system-ui" font-size="10">no structure</text></svg>"##.to_string()
}

fn validate_svg_or(
    candidate: Result<String, Box<dyn std::any::Any + Send>>,
    fallback: fn() -> String,
) -> String {
    match candidate {
        Ok(svg) if svg.trim_start().starts_with("<svg") => svg,
        _ => fallback(),
    }
}

/// Renders an already-parsed [`Molecule`] into a compact inline SVG, falling back
/// to [`empty_compact_svg`] on layout panic or invalid output.
#[must_use]
pub fn render_molecule_svg_compact(molecule: &Molecule) -> String {
    let rendered = catch_unwind(AssertUnwindSafe(|| depict_svg(molecule)));
    validate_svg_or(rendered, empty_compact_svg)
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
    let rendered = catch_unwind(AssertUnwindSafe(|| {
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
    }));
    validate_svg_or(rendered, empty_svg)
}

// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

// The `tests` tests, extracted from `depict_simple.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `depict_simple` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;

#[test]
fn valid_smiles_renders_local_inline_svg() {
    let svg = render_svg("CCCCCCCCCCCCCCCC(=O)O");
    assert!(
        svg.trim_start().starts_with("<svg"),
        "expected inline SVG: {svg}"
    );
    assert!(svg.contains("</svg>"), "expected closed SVG element: {svg}");
    assert!(
        !svg.contains("simolecule.com"),
        "must not emit remote image URL"
    );
    assert!(
        !svg.contains("<img"),
        "must render inline SVG instead of <img>"
    );
    assert!(
        !svg.contains("no structure"),
        "valid SMILES should depict a structure"
    );
}

#[test]
fn empty_or_invalid_smiles_returns_fallback_svg() {
    for bad in ["", "   ", "not-a-valid-smiles-%%%"] {
        let svg = render_svg(bad);
        assert!(
            svg.contains("no structure"),
            "expected fallback for {bad:?}: {svg}"
        );
        assert!(svg.starts_with("<svg"));
    }
}

#[test]
fn all_example_lipids_depict_without_fallback_or_panic() {
    let text = crate::examples::example_smiles().join("\n");
    let blocks = crate::parser::extract_blocks(&text);
    assert!(!blocks.is_empty(), "expected non-empty example blocks");
    for block in blocks {
        let smi = block.psm_smiles.as_deref().unwrap_or("");
        let svg = render_svg(smi);
        assert!(
            svg.trim_start().starts_with("<svg") && !svg.contains("no structure"),
            "failed to depict example SMILES {smi}: {svg}"
        );
    }
}

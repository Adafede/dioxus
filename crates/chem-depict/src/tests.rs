// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::svg::empty_svg;
use crate::{
    ChemParseError, empty_compact_svg, parse_smiles_safe, render_molecule_svg_compact,
    render_smiles_svg, render_smiles_svg_compact, render_smiles_svg_highlighted,
};

#[test]
fn parse_smiles_safe_handles_valid_empty_and_invalid_inputs() {
    assert!(
        parse_smiles_safe("  CCO  ").is_ok(),
        "trimmed valid SMILES should parse"
    );
    assert!(
        matches!(parse_smiles_safe("   "), Err(ChemParseError::Empty)),
        "blank SMILES should return Empty"
    );
    assert!(
        matches!(
            parse_smiles_safe("C1CC"),
            Err(ChemParseError::InvalidSmiles { .. })
        ),
        "unclosed ring should return InvalidSmiles"
    );
}

#[test]
fn valid_smiles_render_non_fallback_svg() -> Result<(), ChemParseError> {
    let ethanol_svg = render_smiles_svg("CCO");
    assert!(
        ethanol_svg.trim_start().starts_with("<svg"),
        "rendered SVG must start with <svg"
    );
    assert_ne!(
        ethanol_svg,
        empty_svg(),
        "valid SMILES should not return fallback"
    );

    let compact_svg = render_smiles_svg_compact("c1ccccc1");
    assert!(
        compact_svg.trim_start().starts_with("<svg"),
        "compact SVG must start with <svg"
    );
    assert_ne!(
        compact_svg,
        empty_compact_svg(),
        "valid compact SMILES should not return fallback"
    );

    let mol = parse_smiles_safe("CCO")?;
    assert_eq!(
        render_molecule_svg_compact(&mol),
        render_smiles_svg_compact("CCO"),
        "molecule and SMILES compact renderers must agree"
    );
    Ok(())
}

#[test]
fn invalid_smiles_and_highlighted_atoms_behave_safely() {
    assert_eq!(
        render_smiles_svg("not_a_smiles"),
        empty_svg(),
        "invalid SMILES should fall back to empty_svg"
    );
    assert_eq!(
        render_smiles_svg_compact(""),
        empty_compact_svg(),
        "empty compact SMILES should fall back to empty_compact_svg"
    );

    let highlighted = render_smiles_svg_highlighted("CCO", &[0, 1, 999]);
    assert!(
        highlighted.trim_start().starts_with("<svg"),
        "out-of-range highlight index must still yield valid SVG"
    );
    assert_ne!(
        highlighted,
        empty_svg(),
        "highlighted valid SMILES should render"
    );
}

#[test]
fn guards_reject_oversized_and_empty_inputs_and_check_svg_closing_tag() {
    let empty_mol = chematic::core::MoleculeBuilder::new().build();
    assert_eq!(render_molecule_svg_compact(&empty_mol), empty_compact_svg());
    assert!(empty_compact_svg().contains("viewBox=\"0 0 80 24\""));

    let huge_smiles = "C".repeat(16_385);
    assert!(matches!(
        parse_smiles_safe(&huge_smiles),
        Err(ChemParseError::InvalidSmiles { .. })
    ));
    assert_eq!(render_smiles_svg(&huge_smiles), empty_svg());

    let over_atoms = "C".repeat(513);
    assert_eq!(render_smiles_svg(&over_atoms), empty_svg());
    assert_eq!(render_smiles_svg_compact(&over_atoms), empty_compact_svg());
    assert!(render_smiles_svg("CCO").trim().ends_with("</svg>"));
}

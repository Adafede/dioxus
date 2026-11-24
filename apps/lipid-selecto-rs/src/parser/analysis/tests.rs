// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

// The `tests` tests, extracted from `analysis.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `analysis` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::super::parsing::extract_blocks;
use super::*;

const EXAMPLE_MGF: &str = "\
BEGIN IONS SMILES=CCCCCCCCCCCCCCCC(=O)O PEPMASS=256.2
CHARGE=1-
TITLE=palmitic_acid
END IONS
BEGIN IONS SMILES=CC1=C(C(=CC=C1)S(=O)(=O)O)C(=O)O
CHARGE=2-
FORMULA=C9H7O4S
TITLE=non_lipid_example
END IONS
BEGIN IONS
PEPMASS=500.0
TITLE=missing_annotation
END IONS
";

#[test]
fn extracts_and_preserves_block_text() {
    let blocks = extract_blocks(EXAMPLE_MGF);
    assert_eq!(blocks.len(), 3);
    assert_eq!(blocks[0].index, 1);
    assert_eq!(
        blocks[0].psm_smiles.as_deref(),
        Some("CCCCCCCCCCCCCCCC(=O)O")
    );
    assert_eq!(blocks[0].title.as_deref(), Some("palmitic_acid"));
    assert!(blocks[0].raw.starts_with("BEGIN IONS"));
    assert!(blocks[0].raw.contains("END IONS"));
}

#[test]
fn detects_lipids_and_keeps_raw_subset() {
    let (blocks, _summary) = analyze(EXAMPLE_MGF);
    assert!(blocks[0].is_lipid());
    assert!(!blocks[1].is_lipid());
    assert!(!blocks[2].is_lipid());

    let analysis = build_analysis(blocks, 16);
    assert!(
        analysis
            .filtered_mgf
            .contains("BEGIN IONS SMILES=CCCCCCCCCCCCCCCC(=O)O")
    );
    assert!(!analysis.filtered_mgf.contains("non_lipid_example"));
    assert!(!analysis.filtered_mgf.contains("missing_annotation"));
}

#[test]
fn summary_counts_lipids() {
    let (blocks, _summary) = analyze(EXAMPLE_MGF);
    let summary = summarize(&blocks);
    assert_eq!(summary.total_items, 3);
    assert_eq!(summary.lipid_items, 1);
    assert_eq!(summary.skipped, 1);
}

#[test]
fn analysis_builds_gallery_and_filtered_mgf() {
    let (blocks, _summary) = analyze(EXAMPLE_MGF);
    let analysis = build_analysis(blocks, 16);
    assert_eq!(analysis.summary.lipid_items, 1);
    assert_eq!(analysis.gallery.len(), 1);
    assert!(!analysis.all_classes.is_empty());
    assert!(analysis.filtered_mgf.contains("palmitic_acid"));
    assert!(!analysis.filtered_mgf.contains("non_lipid_example"));
}

#[test]
fn gallery_items_have_class_matches() {
    let (blocks, _) = analyze(EXAMPLE_MGF);
    let analysis = build_analysis(blocks, 16);

    assert_eq!(analysis.gallery.len(), 1);
    let item = &analysis.gallery[0];

    // All gallery items should have class_matches computed
    assert!(!item.class_matches.is_empty());

    // At least one class should match (since it's a lipid)
    let has_match = item.class_matches.values().any(|&m| m);
    assert!(has_match);
}

#[test]
fn chemical_classes_have_all_required_fields() {
    let classes = ChemicalClass::defaults();

    for class in classes {
        assert!(!class.name.is_empty(), "Class name should not be empty");
        assert!(!class.smarts.is_empty(), "Class SMARTS should not be empty");
        assert!(!class.color.is_empty(), "Class color should not be empty");
        // Colors should start with # or be valid CSS
        assert!(class.color.starts_with('#'), "Color should be hex code");
    }
}

#[test]
fn a_stray_end_ions_outside_any_block_is_not_a_smiles_row() {
    // "END IONS" outside a block is malformed MGF; the letters N, O and S
    // sit inside the SMILES-alphabet heuristic, so without a guard the flat
    // fallback forges a block out of the line. It must be dropped instead,
    // matching how the mgf-precursor-erro-rs parser treats lines that are
    // not inside a block.
    let blocks = extract_blocks("CCCO\nEND IONS\nCCO\n");
    assert_eq!(blocks.len(), 2, "only the two real rows survive");
    assert_eq!(blocks[0].index, 1);
    assert_eq!(blocks[1].index, 2);
    assert_eq!(
        blocks[1].psm_smiles.as_deref(),
        Some("CCO"),
        "the line after the orphan is unchanged"
    );
}

#[test]
fn cyclic_lipids_and_smiles_titles_with_spaces_are_supported() {
    let input = "\
CC(C)CCCC(C)C1CCC2C1(CCC3C2CC=C4C3(CCC(C4)O)C)C\tCholesterol standard
Colneleic acid\tOC(=O)CCCCCC/C=C/O/C=C/C=C\\CCCCC
";
    let (blocks, summary) = analyze(input);
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].title.as_deref(), Some("Cholesterol standard"));
    assert_eq!(blocks[1].title.as_deref(), Some("Colneleic acid"));
    assert!(
        blocks[0].is_lipid(),
        "Cholesterol should be classified as a lipid"
    );
    assert!(
        blocks[1].is_lipid(),
        "Colneleic acid should be classified as a lipid"
    );
    assert_eq!(
        blocks[0]
            .classification
            .as_ref()
            .map(|c| c.class.lipidmaps_category()),
        Some("Sterol Lipids [ST]")
    );
    assert_eq!(summary.lipid_items, 2);
}

#[test]
fn example_smiles_and_asset_file_parse_and_classify() {
    let example_text = crate::examples::example_smiles().join("\n");
    let (blocks, summary) = analyze(&example_text);
    assert_eq!(blocks.len(), 204);
    assert_eq!(summary.total_items, 204);
    assert_eq!(summary.lipid_items, 195);
    assert_eq!(summary.unclassified, 9);

    let asset_text = include_str!("../../../assets/example_lipids.smi");
    let (asset_blocks, asset_summary) = analyze(asset_text);
    assert_eq!(asset_blocks.len(), 204);
    assert_eq!(asset_summary.lipid_items, 195);
    assert_eq!(asset_summary.unclassified, 9);
}

#[test]
fn formula_only_mgf_blocks_classify_compute_mass_and_filter() {
    let mgf = "\
BEGIN IONS
TITLE=fa_only
FORMULA=C16H32O2
END IONS
BEGIN IONS
TITLE=gl_only
FORMULA=C39H74O6
END IONS
BEGIN IONS
TITLE=gp_only
FORMULA=C20H38O8P
END IONS
";
    let (blocks, summary) = analyze(mgf);
    assert_eq!(blocks.len(), 3);
    assert!(blocks[0].is_lipid(), "FA formula block must count as lipid");
    assert!(blocks[1].is_lipid(), "GL formula block must count as lipid");
    assert!(blocks[2].is_lipid(), "GP formula block must count as lipid");
    assert!(
        (blocks[0].exact_mass - 256.240_23).abs() < 0.01,
        "expected C16H32O2 monoisotopic mass ~256.24, got {}",
        blocks[0].exact_mass
    );
    assert!(blocks[1].exact_mass > 600.0);
    assert!(blocks[2].exact_mass > 400.0);
    assert_eq!(summary.lipid_items, 3);
    assert_eq!(summary.unclassified, 0);
    assert_eq!(
        blocks[0]
            .gallery_item_matches
            .as_ref()
            .and_then(|m| m.get("Other Fatty Acyls [FA00]").copied()),
        Some(true)
    );
    assert_eq!(
        blocks[1]
            .gallery_item_matches
            .as_ref()
            .and_then(|m| m.get("Other Glycerolipids [GL00]").copied()),
        Some(true)
    );
    assert_eq!(
        blocks[2]
            .gallery_item_matches
            .as_ref()
            .and_then(|m| m.get("Other Glycerophospholipids [GP00]").copied()),
        Some(true)
    );
    let filtered = build_filtered_mgf(&blocks);
    assert!(filtered.contains("TITLE=fa_only"));
    assert!(filtered.contains("TITLE=gl_only"));
    assert!(filtered.contains("TITLE=gp_only"));
}

#[test]
fn space_delimited_smiles_lines_parse_smiles_and_title() {
    let input = "\
CCCCCCCCCCCCCCCC(=O)O palmitic_acid
CCCCCCCCCCCCCCCC(=O)O\tPalmitic acid standard
";
    let (blocks, summary) = analyze(input);
    assert_eq!(blocks.len(), 2);
    assert_eq!(
        blocks[0].psm_smiles.as_deref(),
        Some("CCCCCCCCCCCCCCCC(=O)O")
    );
    assert_eq!(blocks[0].title.as_deref(), Some("palmitic_acid"));
    assert!(blocks[0].is_lipid());
    assert_eq!(
        blocks[1].psm_smiles.as_deref(),
        Some("CCCCCCCCCCCCCCCC(=O)O")
    );
    assert_eq!(blocks[1].title.as_deref(), Some("Palmitic acid standard"));
    assert!(blocks[1].is_lipid());
    assert_eq!(summary.lipid_items, 2);
}

#[test]
fn valid_smiles_renders_local_inline_svg() {
    let svg = chem_depict::render_smiles_svg_compact("CCCCCCCCCCCCCCCC(=O)O");
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
        let svg = chem_depict::render_smiles_svg_compact(bad);
        assert!(
            svg.contains("no structure"),
            "expected fallback for {bad:?}: {svg}"
        );
        assert!(svg.starts_with("<svg"), "fallback must be <svg>: {svg}");
    }
}

#[test]
fn all_example_lipids_depict_without_fallback_or_panic() {
    let text = crate::examples::example_smiles().join("\n");
    let blocks = extract_blocks(&text);
    assert!(!blocks.is_empty(), "expected non-empty example blocks");
    for block in blocks {
        let smi = block.psm_smiles.as_deref().unwrap_or("");
        let svg = chem_depict::render_smiles_svg_compact(smi);
        assert!(
            svg.trim_start().starts_with("<svg") && !svg.contains("no structure"),
            "failed to depict example SMILES {smi}: {svg}"
        );
    }
}

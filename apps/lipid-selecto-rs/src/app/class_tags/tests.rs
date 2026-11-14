// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

use super::*;
use crate::chemical_class::lmsd_all;
use crate::parser::{build_analysis, extract_blocks};

#[test]
fn class_tags_preserve_canonical_lmsd_order_and_inject_mgf_comments() {
    let mgf = "\
BEGIN IONS SMILES=CCCCCCCCCCCCCCCC(=O)O PEPMASS=256.2
TITLE=palmitic_acid
100.0 50.0
END IONS
";
    let blocks = extract_blocks(mgf);
    let analysis = build_analysis(blocks, 16);
    let classes = lmsd_all();

    let gallery_tags = class_tags_from_gallery(&analysis.gallery[0], &classes);
    assert_eq!(gallery_tags[0].0, "CATEGORY");
    assert!(gallery_tags[0].1.contains("Fatty Acyls [FA]"));
    assert_eq!(gallery_tags[1].0, "MAIN_CLASS");
    assert_ne!(gallery_tags[1].1, "");

    let block_tags = class_tags_from_block(&analysis.blocks[0], &classes);
    assert_eq!(block_tags[0].0, "CATEGORY");
    assert_eq!(block_tags[1].0, "MAIN_CLASS");
    assert_eq!(
        block_tags[1].1,
        analysis.gallery[0]
            .primary_class_name
            .clone()
            .unwrap_or_default()
    );

    let tagged = insert_class_comment(&analysis.blocks[0].raw, &block_tags);
    assert!(tagged.starts_with(
        "BEGIN IONS SMILES=CCCCCCCCCCCCCCCC(=O)O PEPMASS=256.2\nCOMMENT=LIPID_MAPS_CATEGORY="
    ));

    let via_helper =
        crate::app::gallery::block_class_tags(&analysis.blocks[0], &analysis.gallery, &classes);
    assert_eq!(via_helper, gallery_tags);
}

#[test]
fn build_smiles_from_blocks_preserves_smiles_first_and_keeps_untitled_rows() {
    use crate::app::gallery::{build_smiles_from_blocks, passes_item_filters};

    let smi_input = "CCCCCCCCCCCCCCCC(=O)O\tpalmitic_acid\nCCCCCCCCCCCCCCCCCC(=O)O\n";
    let blocks = extract_blocks(smi_input);
    let analysis = build_analysis(blocks, 16);
    let refs: Vec<_> = analysis.blocks.iter().collect();
    let exported = build_smiles_from_blocks(&refs, &analysis.gallery, &analysis.all_classes);

    let lines: Vec<&str> = exported.lines().collect();
    assert_eq!(lines.len(), 2, "both titled and untitled rows must be kept");
    assert!(
        lines[0].starts_with("CCCCCCCCCCCCCCCC(=O)O\tpalmitic_acid\tFatty Acyls [FA]\t"),
        "expected SMILES followed by title, got: {}",
        lines[0]
    );
    assert!(
        lines[1].starts_with("CCCCCCCCCCCCCCCCCC(=O)O\t-\tFatty Acyls [FA]\t"),
        "expected untitled row to use '-' placeholder, got: {}",
        lines[1]
    );

    // Empty class selection must reject items consistently across UI and exports
    assert!(!passes_item_filters(
        Some(&analysis.gallery[0].class_matches),
        analysis.gallery[0].exact_mass,
        analysis.gallery[0].precursor_mz,
        analysis.gallery[0].adduct.as_deref(),
        &[],
        0.0,
        5000.0,
        0.0,
        5000.0,
        "",
    ));
}

#[test]
fn insert_class_comment_preserves_leading_whitespace_before_begin_ions() {
    let raw = "  BEGIN IONS\nTITLE=test\nEND IONS\n";
    let tags = vec![("CATEGORY".to_string(), "Fatty Acyls [FA]".to_string())];
    let tagged = insert_class_comment(raw, &tags);
    assert_eq!(
        tagged,
        "  BEGIN IONS\nCOMMENT=LIPID_MAPS_CATEGORY=Fatty Acyls [FA]\nTITLE=test\nEND IONS\n"
    );
}

#[test]
fn export_and_adduct_options_cover_blocks_beyond_gallery_cap() {
    use crate::app::analysis::collect_adduct_options;
    use crate::app::download::build_filtered_smiles_content;

    let mut mgf = String::new();
    for i in 1..=120 {
        let adduct = if i == 105 { "[M+NH4]+" } else { "[M+H]+" };
        mgf.push_str("BEGIN IONS\n");
        let _ = std::fmt::Write::write_fmt(
            &mut mgf,
            format_args!(
                "TITLE=FA_{i}\nSMILES=CCCCCCCCCCCCCCCC(=O)O\nADDUCT={adduct}\nPEPMASS=274.274\nEND IONS\n"
            ),
        );
    }
    let blocks = extract_blocks(&mgf);
    let analysis = build_analysis(blocks, 96);
    assert_eq!(analysis.gallery.len(), 96);
    assert_eq!(analysis.blocks.len(), 120);

    let adducts = collect_adduct_options(&analysis.blocks);
    assert!(
        adducts.iter().any(|a| a == "[M+NH4]+"),
        "adduct on block #105 must appear even with gallery capped at 96: {adducts:?}"
    );

    let selected: Vec<String> = analysis
        .all_classes
        .iter()
        .map(|c| c.name.clone())
        .collect();
    let exported = build_filtered_smiles_content(
        &analysis.blocks,
        &analysis.gallery,
        &analysis.all_classes,
        0.0,
        5000.0,
        0.0,
        5000.0,
        "",
        &selected,
    );
    assert_eq!(exported.lines().count(), 120);
}

#[test]
fn block_extractor_respects_known_and_autodetected_formats() {
    use crate::format::LipidFormat;
    use crate::parser::BlockExtractor;

    let mgf_with_header = "\
COM=instrument header outside block
MASS=Monoisotopic
BEGIN IONS
TITLE=lip1
SMILES=CCCCCCCCCCCCCCCC(=O)O
END IONS
stray_line_between_blocks
";
    let mut mgf_ext = BlockExtractor::with_format(Some(LipidFormat::Mgf));
    for line in mgf_with_header.lines() {
        mgf_ext.push_line(line);
    }
    assert_eq!(mgf_ext.detected_format(), Some(LipidFormat::Mgf));
    let mgf_blocks = mgf_ext.finish();
    assert_eq!(mgf_blocks.len(), 1);
    assert_eq!(mgf_blocks[0].title.as_deref(), Some("lip1"));

    let mut auto_mgf = BlockExtractor::with_format(None);
    for line in mgf_with_header.lines() {
        auto_mgf.push_line(line);
    }
    assert_eq!(auto_mgf.detected_format(), Some(LipidFormat::Mgf));
    assert_eq!(auto_mgf.finish().len(), 1);

    let mut auto_smi = BlockExtractor::with_format(None);
    for line in ["CCCCCCCCCCCCCCCC(=O)O\tpalmitic", "C18H36O2_ignored\t"] {
        auto_smi.push_line(line);
    }
    assert_eq!(auto_smi.detected_format(), Some(LipidFormat::Smiles));
    assert_eq!(auto_smi.finish().len(), 2);
}

#[test]
fn parse_filter_bound_handles_empty_numbers_and_non_finite_text() {
    use crate::app::components::parse_filter_bound;

    assert_eq!(parse_filter_bound("", 0.0), Some(0.0));
    assert_eq!(parse_filter_bound("   ", 5000.0), Some(5000.0));
    assert_eq!(parse_filter_bound(" 123.5 ", 0.0), Some(123.5));
    assert_eq!(parse_filter_bound("NaN", 0.0), None);
    assert_eq!(parse_filter_bound("inf", 5000.0), None);
    assert_eq!(parse_filter_bound("abc", 0.0), None);
}

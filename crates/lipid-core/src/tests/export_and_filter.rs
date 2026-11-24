// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

use std::fmt::Write as _;

use crate::export::{
    ClassTags, block_class_tags, class_tags_from_block, class_tags_from_gallery,
    insert_class_comment,
};
use crate::filter::group_classes_by_family;
use crate::parser::analysis::build_analysis;
use crate::parser::extractor::extract_blocks;
use crate::{
    BlockExtractor, ClassMatchSet, FilterCriteria, LipidFamily, LipidFormat,
    build_filtered_mgf_content, build_filtered_smiles_content, collect_adduct_options,
    lmsd_classes, passes_item_filters,
};

fn build_120_lipid_analysis() -> crate::Analysis {
    let mut mgf = String::new();
    for i in 1..=120 {
        let adduct = if i == 105 { "[M+NH4]+" } else { "[M+H]+" };
        mgf.push_str("BEGIN IONS\n");
        let _ = write!(
            mgf,
            "TITLE=FA_{i}\nSMILES=CCCCCCCCCCCCCCCC(=O)O\nADDUCT={adduct}\nPEPMASS=274.274\nEND IONS\n"
        );
    }
    build_analysis(extract_blocks(&mgf), 96)
}

#[test]
fn class_tags_preserve_canonical_lmsd_order_and_inject_mgf_comments() {
    let mgf = "\
BEGIN IONS SMILES=CCCCCCCCCCCCCCCC(=O)O PEPMASS=256.2
TITLE=palmitic_acid
100.0 50.0
END IONS
";
    let analysis = build_analysis(extract_blocks(mgf), 16);
    let classes = lmsd_classes();

    let gallery_tags = class_tags_from_gallery(&analysis.gallery[0], classes);
    assert!(gallery_tags.category.contains("Fatty Acyls [FA]"));
    assert_ne!(gallery_tags.main_class, "");

    let block_tags = class_tags_from_block(&analysis.blocks[0], classes);
    assert_eq!(block_tags.category, gallery_tags.category);
    assert_eq!(
        block_tags.main_class,
        analysis.gallery[0]
            .primary_class_name
            .clone()
            .unwrap_or_default()
    );

    let tagged = insert_class_comment(&analysis.blocks[0].raw, &block_tags);
    assert!(tagged.starts_with(
        "BEGIN IONS SMILES=CCCCCCCCCCCCCCCC(=O)O PEPMASS=256.2\nCOMMENT=LIPID_MAPS_CATEGORY="
    ));
    assert_eq!(
        block_class_tags(&analysis.blocks[0], &analysis.gallery, classes),
        gallery_tags
    );
}

#[test]
fn passes_item_filters_checks_classes_mass_precursor_and_adduct_boundaries() {
    let mask = ClassMatchSet::from_indices([0]);
    let other_mask = ClassMatchSet::from_indices([1]);
    let base = FilterCriteria {
        selected_classes: mask,
        mz_min: 100.0,
        mz_max: 300.0,
        precursor_min: 200.0,
        precursor_max: 400.0,
        adduct: "[M+H]+",
    };

    assert!(passes_item_filters(
        mask,
        100.0,
        Some(200.0),
        Some("[M+H]+"),
        &base
    ));
    assert!(passes_item_filters(
        mask,
        300.0,
        None,
        Some("[M+H]+"),
        &base
    ));
    assert!(!passes_item_filters(
        other_mask,
        200.0,
        Some(250.0),
        Some("[M+H]+"),
        &base
    ));
    assert!(!passes_item_filters(
        mask,
        99.9,
        Some(250.0),
        Some("[M+H]+"),
        &base
    ));
    assert!(!passes_item_filters(
        mask,
        300.1,
        Some(250.0),
        Some("[M+H]+"),
        &base
    ));
    assert!(!passes_item_filters(
        mask,
        200.0,
        Some(199.9),
        Some("[M+H]+"),
        &base
    ));
    assert!(!passes_item_filters(
        mask,
        200.0,
        Some(400.1),
        Some("[M+H]+"),
        &base
    ));
    assert!(!passes_item_filters(
        mask,
        200.0,
        Some(250.0),
        Some("[M+Na]+"),
        &base
    ));
    assert!(!passes_item_filters(
        mask,
        200.0,
        Some(250.0),
        Some("[M+H]+"),
        &FilterCriteria {
            selected_classes: ClassMatchSet::empty(),
            ..base
        }
    ));
}

#[test]
fn build_filtered_mgf_and_smiles_apply_filters_and_preserve_untitled_rows() {
    let smi_input = "CCCCCCCCCCCCCCCC(=O)O\tpalmitic_acid\nCCCCCCCCCCCCCCCCCC(=O)O\n";
    let analysis = build_analysis(extract_blocks(smi_input), 16);
    let classes = lmsd_classes();
    let all_mask = ClassMatchSet::all(classes.len());
    let criteria = FilterCriteria {
        selected_classes: all_mask,
        mz_min: 0.0,
        mz_max: 5000.0,
        precursor_min: 0.0,
        precursor_max: 5000.0,
        adduct: "",
    };
    let exported =
        build_filtered_smiles_content(&analysis.blocks, &analysis.gallery, classes, &criteria);
    let lines: Vec<&str> = exported.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("CCCCCCCCCCCCCCCC(=O)O\tpalmitic_acid\tFatty Acyls [FA]\t"));
    assert!(lines[1].starts_with("CCCCCCCCCCCCCCCCCC(=O)O\t-\tFatty Acyls [FA]\t"));

    let empty = build_filtered_mgf_content(
        &analysis.blocks,
        &analysis.gallery,
        classes,
        &FilterCriteria {
            selected_classes: ClassMatchSet::empty(),
            ..criteria
        },
    );
    assert_eq!(empty, "");
}

#[test]
fn group_classes_and_collect_adduct_options_order_deterministically() {
    let classes = lmsd_classes();
    let grouped = group_classes_by_family(classes);
    let fams: Vec<LipidFamily> = grouped.iter().map(|(f, _)| *f).collect();
    assert_eq!(fams, LipidFamily::ALL);
    assert_eq!(grouped.iter().map(|(_, c)| c.len()).sum::<usize>(), 86);

    let mgf = "\
BEGIN IONS SMILES=CCCCCCCCCCCCCCCC(=O)O ADDUCT=[M-H]-
END IONS
BEGIN IONS SMILES=CCCCCCCCCCCCCCCC(=O)O ADDUCT=[M+H]+
END IONS
BEGIN IONS SMILES=CCCCCCCCCCCCCCCC(=O)O ADDUCT=[M+NH4]+
END IONS
BEGIN IONS SMILES=CCO ADDUCT=[M+K]+
END IONS
";
    let (blocks, _) = crate::parser::analysis::analyze(mgf);
    assert_eq!(
        collect_adduct_options(&blocks),
        vec!["[M+H]+", "[M+NH4]+", "[M-H]-"]
    );
}

#[test]
fn export_and_adduct_options_stay_consistent_past_gallery_cap() {
    let analysis = build_120_lipid_analysis();
    assert_eq!(analysis.gallery.len(), 96);
    assert_eq!(analysis.blocks.len(), 120);
    assert!(
        collect_adduct_options(&analysis.blocks)
            .iter()
            .any(|a| a == "[M+NH4]+")
    );

    let block_1 = &analysis.blocks[0];
    let block_105 = &analysis.blocks[104];
    let tags_1 = block_class_tags(block_1, &analysis.gallery, analysis.all_classes);
    let tags_105 = block_class_tags(block_105, &analysis.gallery, analysis.all_classes);
    // Conservative parity check: category stays identical past gallery cap; fallback keeps primary class.
    assert_eq!(tags_105.category, tags_1.category);
    assert_eq!(tags_105.main_class, "Fatty Acids and Conjugates [FA01]");

    let all_criteria = FilterCriteria {
        selected_classes: ClassMatchSet::all(analysis.all_classes.len()),
        mz_min: 0.0,
        mz_max: 5000.0,
        precursor_min: 0.0,
        precursor_max: 5000.0,
        adduct: "",
    };
    let smi = build_filtered_smiles_content(
        &analysis.blocks,
        &analysis.gallery,
        analysis.all_classes,
        &all_criteria,
    );
    assert_eq!(smi.lines().count(), 120);

    let nh4_mgf = build_filtered_mgf_content(
        &analysis.blocks,
        &analysis.gallery,
        analysis.all_classes,
        &FilterCriteria {
            adduct: "[M+NH4]+",
            ..all_criteria
        },
    );
    assert_eq!(nh4_mgf.matches("BEGIN IONS").count(), 1);
    assert!(nh4_mgf.contains("TITLE=FA_105"));
    assert!(nh4_mgf.contains("COMMENT=LIPID_MAPS_MAIN_CLASS="));
}

#[test]
fn insert_class_comment_is_case_insensitive_and_preserves_leading_ws() {
    let tags = ClassTags {
        category: "Fatty Acyls [FA]".to_string(),
        main_class: "-".to_string(),
        sub_class: "-",
    };
    let raw = "  begin ions\nTITLE=test\nend ions\n";
    let tagged = insert_class_comment(raw, &tags);
    assert_eq!(
        tagged,
        "  begin ions\nCOMMENT=LIPID_MAPS_CATEGORY=Fatty Acyls [FA]\n\
         COMMENT=LIPID_MAPS_MAIN_CLASS=-\nCOMMENT=LIPID_MAPS_SUB_CLASS=-\nTITLE=test\nend ions\n"
    );
}

#[test]
fn block_extractor_respects_known_and_autodetected_formats() {
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
    assert_eq!(mgf_ext.finish()[0].title.as_deref(), Some("lip1"));

    let mut auto_smi = BlockExtractor::with_format(None);
    for line in ["CCCCCCCCCCCCCCCC(=O)O\tpalmitic", "C18H36O2_ignored\t"] {
        auto_smi.push_line(line);
    }
    assert_eq!(auto_smi.detected_format(), Some(LipidFormat::Smiles));
    assert_eq!(auto_smi.finish().len(), 2);
}

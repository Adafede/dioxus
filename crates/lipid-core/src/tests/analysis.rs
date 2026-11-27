// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

use super::{EXAMPLE_MGF, analyze, build_analysis, extract_blocks};
use crate::parser::analysis::summarize;
use crate::{
    BlockExtractor, ClassMatchSet, FilterCriteria, LipidFamily, LipidFormat,
    build_filtered_mgf_content, example_smiles, lmsd_classes,
};

#[test]
fn extracts_analyzes_and_summarizes_mgf() {
    let blocks = extract_blocks(EXAMPLE_MGF);
    assert_eq!(blocks.len(), 3);
    assert_eq!(blocks[0].index, 1);
    assert_eq!(
        blocks[0].psm_smiles.as_deref(),
        Some("CCCCCCCCCCCCCCCC(=O)O")
    );
    assert_eq!(blocks[0].title.as_deref(), Some("palmitic_acid"));
    assert!(blocks[0].raw.starts_with("BEGIN IONS") && blocks[0].raw.contains("END IONS"));

    let (analyzed, summary) = analyze(EXAMPLE_MGF);
    assert!(analyzed[0].is_lipid());
    assert!(!analyzed[1].is_lipid());
    assert!(!analyzed[2].is_lipid());
    assert_eq!(
        (summary.total_items, summary.lipid_items, summary.skipped),
        (3, 1, 1)
    );

    let full = build_analysis(analyzed, 16);
    assert_eq!(full.gallery.len(), 1);
    assert!(!full.gallery[0].class_matches.is_empty());

    let all_mask = ClassMatchSet::all(full.all_classes.len());
    let criteria = FilterCriteria {
        selected_classes: all_mask,
        mz_min: 0.0,
        mz_max: 5000.0,
        precursor_min: 0.0,
        precursor_max: 5000.0,
        adduct: "",
    };
    let filtered =
        build_filtered_mgf_content(&full.blocks, &full.gallery, full.all_classes, &criteria);
    assert!(filtered.contains("palmitic_acid"));
    assert!(!filtered.contains("non_lipid_example"));
    assert!(!filtered.contains("missing_annotation"));
}

#[test]
fn mgf_edge_cases_and_stray_end_ions() {
    let stray = extract_blocks("CCCO\nEND IONS\nCCO\n");
    assert_eq!(stray.len(), 2);
    assert_eq!(stray[0].index, 1);
    assert_eq!(stray[1].psm_smiles.as_deref(), Some("CCO"));

    // Unclosed BEGIN IONS flushed by finish() + lowercase IONMODE + PRECURSOR_MZ + inline PEPMASS
    let edge = "PRE_COMMENT=ignored\nBEGIN IONS\nTITLE=edge1\nIONMODE=Positive\nPRECURSOR_MZ=300.5\nPEPMASS=299.1 1000\nFORMULA=C16H32O2\n";
    let mut ext = BlockExtractor::with_format(None);
    for line in edge.lines() {
        ext.push_line(line);
    }
    assert_eq!(ext.detected_format(), Some(LipidFormat::Mgf));
    let parsed = ext.finish();
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].title.as_deref(), Some("edge1"));
    assert_eq!(parsed[0].ion_mode.as_deref(), Some("positive"));
    assert_eq!(parsed[0].precursor_mz, Some(299.1));
}

#[test]
fn cyclic_lipids_and_space_or_tab_titles() {
    let input = "\
CC(C)CCCC(C)C1CCC2C1(CCC3C2CC=C4C3(CCC(C4)O)C)C\tCholesterol standard
Colneleic acid\tOC(=O)CCCCCC/C=C/O/C=C/C=C\\CCCCC
CCCCCCCCCCCCCCCC(=O)O palmitic_acid
";
    let (blocks, summary) = analyze(input);
    assert_eq!(blocks.len(), 3);
    assert_eq!(blocks[0].title.as_deref(), Some("Cholesterol standard"));
    assert_eq!(blocks[1].title.as_deref(), Some("Colneleic acid"));
    assert_eq!(blocks[2].title.as_deref(), Some("palmitic_acid"));
    assert_eq!(
        blocks[0].classification.as_ref().map(|c| c.family),
        Some(LipidFamily::SterolLipids)
    );
    assert_eq!(summary.lipid_items, 3);
}

#[test]
fn example_smiles_and_asset_file_parse_classify_and_depict() {
    let example_text = example_smiles().join("\n");
    let (blocks, summary) = analyze(&example_text);
    assert_eq!(blocks.len(), 204);
    assert_eq!(summary.total_items, 204);
    assert_eq!(summary.lipid_items, 195);
    assert_eq!(summary.unclassified, 9);

    for block in &blocks {
        let smi = block.psm_smiles.as_deref().unwrap_or("");
        let svg = chem_depict::render_smiles_svg_compact(smi);
        assert!(
            svg.trim_start().starts_with("<svg") && !svg.contains("no structure"),
            "failed to depict example SMILES {smi}: {svg}"
        );
    }
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
    assert!(blocks[0].is_lipid() && blocks[1].is_lipid() && blocks[2].is_lipid());
    assert!((blocks[0].exact_mass - 256.240_23).abs() < 0.01);
    assert!(blocks[1].exact_mass > 600.0 && blocks[2].exact_mass > 400.0);
    assert_eq!((summary.lipid_items, summary.unclassified), (3, 0));

    let classes = lmsd_classes();
    for (idx, name) in [
        (0, "Other Fatty Acyls [FA00]"),
        (1, "Other Glycerolipids [GL00]"),
        (2, "Other Glycerophospholipids [GP00]"),
    ] {
        let cls = classes.iter().find(|c| c.name == name).unwrap();
        assert!(
            blocks[idx].class_matches().contains(cls.index),
            "missing {name}"
        );
    }

    let analysis = build_analysis(blocks, 16);
    assert_eq!(summarize(&analysis.blocks).lipid_items, 3);
    let criteria = FilterCriteria {
        selected_classes: ClassMatchSet::all(classes.len()),
        mz_min: 0.0,
        mz_max: 5000.0,
        precursor_min: 0.0,
        precursor_max: 5000.0,
        adduct: "",
    };
    let filtered =
        build_filtered_mgf_content(&analysis.blocks, &analysis.gallery, classes, &criteria);
    assert!(
        filtered.contains("TITLE=fa_only")
            && filtered.contains("TITLE=gl_only")
            && filtered.contains("TITLE=gp_only")
    );
}

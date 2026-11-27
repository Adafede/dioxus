// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

mod analysis;
mod chemical_class;
mod export_and_filter;
mod format;
mod lipids;

use crate::parser::analysis::{Summary, summarize};
use crate::{
    Analysis, BlockExtractor, SpectrumBlock, build_analysis_from_classified, classify_blocks,
    lmsd_classes,
};

pub(super) fn extract_blocks(content: &str) -> Vec<SpectrumBlock> {
    let mut extractor = BlockExtractor::default();
    for line in content.lines() {
        extractor.push_line(line);
    }
    extractor.finish()
}

pub(super) fn build_analysis(mut blocks: Vec<SpectrumBlock>, gallery_limit: usize) -> Analysis {
    let all_classes = lmsd_classes();
    classify_blocks(&mut blocks, all_classes);
    build_analysis_from_classified(blocks, gallery_limit, all_classes)
}

pub(super) fn analyze(content: &str) -> (Vec<SpectrumBlock>, Summary) {
    let mut blocks = extract_blocks(content);
    classify_blocks(&mut blocks, lmsd_classes());
    let summary = summarize(&blocks);
    (blocks, summary)
}

pub(super) const EXAMPLE_MGF: &str = "\
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

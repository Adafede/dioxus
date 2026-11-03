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
}

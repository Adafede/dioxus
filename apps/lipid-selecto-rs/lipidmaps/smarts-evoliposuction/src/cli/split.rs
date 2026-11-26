// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

//! LMSD TSV parsing, negative/positive capping, and manifest splitting step.

use std::fs;
use std::path::Path;

use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use smarts_evoliposuction::{
    CliError, Level, ManifestRow, SplitConfig, SplitResult, parse_lmsd_tsv, split_dataset,
    write_manifest,
};

pub(crate) fn cap_and_shuffle(smiles: &[String], cap: usize, rng: &mut StdRng) -> Vec<String> {
    let mut items = smiles.to_vec();
    items.shuffle(rng);
    items.truncate(cap);
    items
}

pub(crate) fn split_step(
    input: &Path,
    output_dir: &Path,
    smiles_cap: usize,
    max_negatives: usize,
) -> Result<(), CliError> {
    let manifest_path = output_dir.join("manifest.csv");
    if manifest_path.exists() {
        eprintln!(
            "[SKIP] Manifest already exists: {}",
            manifest_path.display()
        );
        return Ok(());
    }

    eprintln!(" Splitting dataset: {}", input.display());
    let tsv_text = fs::read_to_string(input)?;
    let dataset = parse_lmsd_tsv(&tsv_text)?;
    eprintln!("[OK] Loaded {} lipids", dataset.rows.len());

    let config = SplitConfig {
        max_negatives,
        ..SplitConfig::default()
    };
    let result = split_dataset(&dataset, &config);
    log_split_summary(&result);

    fs::create_dir_all(output_dir)?;
    let manifest_rows = write_split_files(output_dir, &result, smiles_cap, max_negatives)?;
    write_manifest(&manifest_path, &manifest_rows)?;
    eprintln!(
        "[OK] Manifest & {} SMILES files written to {}",
        manifest_rows.len(),
        output_dir.display()
    );
    Ok(())
}

fn log_split_summary(result: &SplitResult) {
    let count_level = |level| result.classes.iter().filter(|s| s.level == level).count();
    eprintln!(
        "[OK] Split into {} classes ({} category, {} main, {} sub)",
        result.classes.len(),
        count_level(Level::Category),
        count_level(Level::MainClass),
        count_level(Level::Subclass),
    );
}

fn write_split_files(
    output_dir: &Path,
    result: &SplitResult,
    smiles_cap: usize,
    max_negatives: usize,
) -> Result<Vec<ManifestRow>, CliError> {
    let mut manifest_rows = Vec::with_capacity(result.classes.len());
    let mut rng = StdRng::seed_from_u64(42);

    for split in &result.classes {
        let positive = cap_and_shuffle(&split.positive, smiles_cap, &mut rng);
        let negative = cap_and_shuffle(&split.negative, max_negatives, &mut rng);

        let pos_path = output_dir.join(format!("{}_positive.smiles", split.slug));
        let neg_path = output_dir.join(format!("{}_negative.smiles", split.slug));
        fs::write(&pos_path, positive.join("\n"))?;
        fs::write(&neg_path, negative.join("\n"))?;

        manifest_rows.push(ManifestRow {
            level: split.level.to_string(),
            label: split.label.clone(),
            category: split.category.clone(),
            main_class: split.main_class.clone(),
            subclass: split.subclass.clone(),
            slug: split.slug.clone(),
            positive_path: pos_path.to_string_lossy().to_string(),
            negative_path: neg_path.to_string_lossy().to_string(),
            positive_count: positive.len().to_string(),
            negative_count: negative.len().to_string(),
        });
    }

    Ok(manifest_rows)
}

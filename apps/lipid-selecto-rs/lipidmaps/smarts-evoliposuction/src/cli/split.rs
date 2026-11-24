// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

//! LMSD TSV parsing, negative/positive capping, and manifest splitting step.

use std::fs;
use std::path::Path;

use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use smarts_evoliposuction::{
    CliError, Dataset, DatasetRow, Level, ManifestRow, SplitConfig, SplitResult, split_dataset,
    write_manifest,
};

struct TsvColumns {
    smiles: usize,
    category: Option<usize>,
    main_class: usize,
    subclass: usize,
}

impl TsvColumns {
    fn from_header(header_line: &str) -> Result<Self, CliError> {
        let header: Vec<&str> = header_line.split('\t').collect();
        let find = |name| header.iter().position(|&col| col == name);
        Ok(Self {
            smiles: find("SMILES").ok_or(CliError::MissingColumn("SMILES"))?,
            category: find("CATEGORY"),
            main_class: find("MAIN_CLASS").ok_or(CliError::MissingColumn("MAIN_CLASS"))?,
            subclass: find("SUB_CLASS").ok_or(CliError::MissingColumn("SUB_CLASS"))?,
        })
    }

    fn parse_row(&self, line: &str) -> Option<DatasetRow> {
        let cols: Vec<&str> = line.split('\t').collect();
        let smiles = cols.get(self.smiles)?.trim();
        let main_class = cols.get(self.main_class)?.trim();
        let subclass = cols.get(self.subclass)?.trim();
        if smiles.is_empty() {
            return None;
        }
        let category = self
            .category
            .and_then(|idx| cols.get(idx))
            .map_or("", |val| val.trim());
        Some(DatasetRow {
            smiles: smiles.to_string(),
            category: category.to_string(),
            main_class: main_class.to_string(),
            subclass: subclass.to_string(),
        })
    }
}

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
    let manifest_rows = write_split_files(output_dir, &result, smiles_cap)?;
    write_manifest(&manifest_path, &manifest_rows)?;
    eprintln!(
        "[OK] Manifest & {} SMILES files written to {}",
        manifest_rows.len(),
        output_dir.display()
    );
    Ok(())
}

fn parse_lmsd_tsv(tsv_text: &str) -> Result<Dataset, CliError> {
    let mut lines = tsv_text.lines();
    let Some(header_line) = lines.next() else {
        return Err(CliError::EmptyInput);
    };
    let columns = TsvColumns::from_header(header_line)?;
    let rows = lines.filter_map(|line| columns.parse_row(line)).collect();
    Ok(Dataset { rows })
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
) -> Result<Vec<ManifestRow>, CliError> {
    let mut manifest_rows = Vec::with_capacity(result.classes.len());
    let mut rng = StdRng::seed_from_u64(42);

    for split in &result.classes {
        let positive = cap_and_shuffle(&split.positive, smiles_cap, &mut rng);
        let negative = cap_and_shuffle(&split.negative, smiles_cap, &mut rng);

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

// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! CSV parsing and positive/negative SMILES splitting for `smarts-evolution`.

use std::collections::{HashMap, HashSet};

#[cfg(test)]
use csv::ReaderBuilder;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

mod slug;
#[cfg(test)]
mod tests;
mod types;

use self::slug::{category_of_main, category_rank, dedupe_slug, slugify, target_negative_count};

pub use self::types::{
    ClassSplit, Dataset, DatasetRow, Level, SkippedClass, SplitConfig, SplitResult,
    SubclassNegatives,
};
#[cfg(test)]
use self::types::{ColumnNames, SplitError};

#[cfg(test)]
fn parse_csv(csv_text: &str, columns: &ColumnNames) -> Result<Dataset, SplitError> {
    let mut reader = ReaderBuilder::new().from_reader(csv_text.as_bytes());
    let headers = reader.headers()?.clone();

    let find = |name: &str| headers.iter().position(|h| h == name);
    let smiles_idx = find(&columns.smiles);
    let cat_idx = find(&columns.category);
    let main_idx = find(&columns.main_class);
    let sub_idx = find(&columns.subclass).or_else(|| {
        (columns.subclass == "SUB_CLASS")
            .then(|| find("SUBCLASS"))
            .flatten()
    });

    let (smiles_idx, main_idx, sub_idx) = require_columns(columns, smiles_idx, main_idx, sub_idx)?;

    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record?;
        let smiles = record.get(smiles_idx).unwrap_or("").trim();
        let main_class = record.get(main_idx).unwrap_or("").trim();
        if smiles.is_empty() || main_class.is_empty() {
            continue;
        }
        rows.push(DatasetRow {
            smiles: smiles.to_string(),
            category: cat_idx
                .map_or("", |i| record.get(i).unwrap_or(""))
                .trim()
                .to_string(),
            main_class: main_class.to_string(),
            subclass: record.get(sub_idx).unwrap_or("").trim().to_string(),
        });
    }

    Ok(Dataset { rows })
}

#[cfg(test)]
fn require_columns(
    columns: &ColumnNames,
    smiles_idx: Option<usize>,
    main_idx: Option<usize>,
    sub_idx: Option<usize>,
) -> Result<(usize, usize, usize), SplitError> {
    let mut missing = Vec::new();
    if smiles_idx.is_none() {
        missing.push(columns.smiles.clone());
    }
    if main_idx.is_none() {
        missing.push(columns.main_class.clone());
    }
    if sub_idx.is_none() {
        missing.push(columns.subclass.clone());
    }
    match (smiles_idx, main_idx, sub_idx) {
        (Some(s), Some(m), Some(sub)) => Ok((s, m, sub)),
        _ => Err(SplitError::MissingColumns(missing)),
    }
}

struct SplitSpec<'a> {
    level: Level,
    label: String,
    owner_key: String,
    category: &'a str,
    main_class: &'a str,
    subclass: &'a str,
    slug_base: String,
    positives: Vec<&'a str>,
    neg_pool_source: &'a [&'a str],
}

struct SplitEmitter<'a> {
    config: &'a SplitConfig,
    slug_owner: HashMap<String, String>,
    rng: StdRng,
    classes: Vec<ClassSplit>,
    skipped: Vec<SkippedClass>,
}

impl<'a> SplitEmitter<'a> {
    fn new(config: &'a SplitConfig) -> Self {
        Self {
            config,
            slug_owner: HashMap::new(),
            rng: StdRng::seed_from_u64(config.seed),
            classes: Vec::new(),
            skipped: Vec::new(),
        }
    }

    fn emit(&mut self, specs: Vec<SplitSpec<'_>>) {
        for spec in specs {
            if spec.positives.len() < self.config.min_positive {
                let label = if spec.level == Level::Subclass {
                    spec.owner_key
                } else {
                    spec.label
                };
                self.skipped.push(SkippedClass {
                    level: spec.level,
                    label,
                    count: spec.positives.len(),
                });
                continue;
            }
            let slug = dedupe_slug(spec.slug_base, &spec.owner_key, &mut self.slug_owner);
            let pos_set: HashSet<&str> = spec.positives.iter().copied().collect();
            let neg_pool: Vec<&str> = spec
                .neg_pool_source
                .iter()
                .copied()
                .filter(|s| !pos_set.contains(s))
                .collect();
            let target = target_negative_count(spec.positives.len(), self.config);
            let negative = sample_strs(&neg_pool, target, &mut self.rng);
            let positive = spec.positives.into_iter().map(str::to_string).collect();

            self.classes.push(ClassSplit {
                level: spec.level,
                label: spec.label,
                category: spec.category.to_string(),
                main_class: spec.main_class.to_string(),
                subclass: spec.subclass.to_string(),
                slug,
                positive,
                negative,
            });
        }
    }
}

/// Build deterministic `CATEGORY`, `MAIN_CLASS`, and `SUB_CLASS` splits without intermediate string clones.
#[must_use]
pub fn split_dataset(dataset: &Dataset, config: &SplitConfig) -> SplitResult {
    let (all_smiles, by_cat, by_main, by_sub) = index_dataset(dataset);
    let mut emitter = SplitEmitter::new(config);

    let mut cat_keys: Vec<&str> = by_cat.keys().copied().collect();
    cat_keys.sort_by_key(|c| (category_rank(c), *c));
    emitter.emit(
        cat_keys
            .into_iter()
            .map(|cat| SplitSpec {
                level: Level::Category,
                label: cat.to_string(),
                owner_key: cat.to_string(),
                category: cat,
                main_class: "",
                subclass: "",
                slug_base: slugify(cat),
                positives: by_cat.get(cat).cloned().unwrap_or_default(),
                neg_pool_source: &all_smiles,
            })
            .collect(),
    );

    let mut main_keys: Vec<&str> = by_main.keys().copied().collect();
    main_keys.sort_by(|a, b| {
        category_rank(category_of_main(a))
            .cmp(&category_rank(category_of_main(b)))
            .then_with(|| a.cmp(b))
    });
    emitter.emit(
        main_keys
            .into_iter()
            .map(|main| SplitSpec {
                level: Level::MainClass,
                label: main.to_string(),
                owner_key: main.to_string(),
                category: "",
                main_class: main,
                subclass: "",
                slug_base: slugify(main),
                positives: by_main.get(main).cloned().unwrap_or_default(),
                neg_pool_source: &all_smiles,
            })
            .collect(),
    );

    let mut sub_keys: Vec<(&str, &str)> = by_sub.keys().copied().collect();
    sub_keys.sort_unstable();
    emitter.emit(
        sub_keys
            .into_iter()
            .map(|(main, sub)| {
                let source = match config.subclass_negatives {
                    SubclassNegatives::Siblings => by_main.get(main).map_or(&[][..], Vec::as_slice),
                    SubclassNegatives::Global => &all_smiles,
                };
                SplitSpec {
                    level: Level::Subclass,
                    label: sub.to_string(),
                    owner_key: format!("{main}/{sub}"),
                    category: "",
                    main_class: main,
                    subclass: sub,
                    slug_base: format!("{}_{}", slugify(main), slugify(sub)),
                    positives: by_sub.get(&(main, sub)).cloned().unwrap_or_default(),
                    neg_pool_source: source,
                }
            })
            .collect(),
    );

    emitter
        .classes
        .sort_by(|a, b| (a.level as u8, &a.slug).cmp(&(b.level as u8, &b.slug)));
    SplitResult {
        classes: emitter.classes,
        skipped: emitter.skipped,
    }
}

type SplitIndex<'a> = (
    Vec<&'a str>,
    HashMap<&'a str, Vec<&'a str>>,
    HashMap<&'a str, Vec<&'a str>>,
    HashMap<(&'a str, &'a str), Vec<&'a str>>,
);

fn index_dataset(dataset: &Dataset) -> SplitIndex<'_> {
    let mut all_smiles: Vec<&str> = Vec::new();
    let mut seen: HashSet<&str> = HashSet::new();
    let mut by_cat: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut by_main: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut by_sub: HashMap<(&str, &str), Vec<&str>> = HashMap::new();

    for row in &dataset.rows {
        let smi = row.smiles.as_str();
        if seen.insert(smi) {
            all_smiles.push(smi);
        }
        if !row.category.is_empty() {
            by_cat.entry(row.category.as_str()).or_default().push(smi);
        }
        by_main
            .entry(row.main_class.as_str())
            .or_default()
            .push(smi);
        if !row.subclass.is_empty() {
            by_sub
                .entry((row.main_class.as_str(), row.subclass.as_str()))
                .or_default()
                .push(smi);
        }
    }
    (all_smiles, by_cat, by_main, by_sub)
}

fn sample_strs(pool: &[&str], k: usize, rng: &mut StdRng) -> Vec<String> {
    let mut remaining = pool.to_vec();
    let count = k.min(remaining.len());
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let j = rng.random_range(0..remaining.len());
        out.push(remaining.swap_remove(j).to_string());
    }
    out
}

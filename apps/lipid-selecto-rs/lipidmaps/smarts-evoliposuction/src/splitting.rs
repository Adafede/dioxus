//! CSV parsing and positive/negative SMILES splitting for smarts-evolution.
//!
//! Pure Rust (no smarts-evolution dependency) — safe to compile on any target.

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;

use csv::ReaderBuilder;
use rand::RngExt;
use rand::SeedableRng;
use rand::rngs::StdRng;

/// Column names to read from the input CSV. Defaults match the common
/// `SMILES,MAIN_CLASS,SUBCLASS` header.
#[derive(Debug, Clone)]
pub struct ColumnNames {
    pub smiles: String,
    pub category: String,
    pub main_class: String,
    pub subclass: String,
}

impl Default for ColumnNames {
    fn default() -> Self {
        Self {
            smiles: "SMILES".to_string(),
            category: "CATEGORY".to_string(),
            main_class: "MAIN_CLASS".to_string(),
            subclass: "SUBCLASS".to_string(),
        }
    }
}

/// Where subclass negatives are drawn from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SubclassNegatives {
    /// Only from *other subclasses within the same `MAIN_CLASS`* — harder,
    /// more chemically meaningful negatives (the default).
    #[default]
    Siblings,
    /// From the whole dataset minus the subclass itself.
    Global,
}

/// Balancing / sampling knobs for [`split_dataset`].
#[derive(Debug, Clone)]
pub struct SplitConfig {
    /// Classes with fewer than this many positive SMILES are skipped.
    pub min_positive: usize,
    /// Target negative count = `min(neg_ratio * positive_count, max_negatives)`.
    pub neg_ratio: f64,
    /// Hard cap on negatives per class, regardless of `neg_ratio`.
    pub max_negatives: usize,
    /// Where subclass negatives come from.
    pub subclass_negatives: SubclassNegatives,
    /// RNG seed — same seed + same input always gives the same split.
    pub seed: u64,
}

impl Default for SplitConfig {
    fn default() -> Self {
        Self {
            min_positive: 20,
            neg_ratio: 1.0,
            max_negatives: 5000,
            subclass_negatives: SubclassNegatives::default(),
            seed: 42,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Level {
    Category,
    MainClass,
    Subclass,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Category => write!(f, "category"),
            Self::MainClass => write!(f, "main_class"),
            Self::Subclass => write!(f, "subclass"),
        }
    }
}

/// One row of the input dataset after parsing.
#[derive(Debug, Clone)]
pub struct DatasetRow {
    pub smiles: String,
    pub category: String,
    pub main_class: String,
    pub subclass: String,
}

/// A parsed, in-memory dataset. Build with [`parse_csv`].
#[derive(Debug, Clone, Default)]
pub struct Dataset {
    pub rows: Vec<DatasetRow>,
}

#[derive(Debug)]
pub enum SplitError {
    Csv(csv::Error),
    MissingColumns(Vec<String>),
}

impl fmt::Display for SplitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Csv(e) => write!(f, "CSV error: {e}"),
            Self::MissingColumns(cols) => {
                write!(f, "missing column(s) in CSV header: {}", cols.join(", "))
            }
        }
    }
}

impl std::error::Error for SplitError {}

impl From<csv::Error> for SplitError {
    fn from(e: csv::Error) -> Self {
        Self::Csv(e)
    }
}

/// Parse CSV text (already in memory) into a [`Dataset`].
///
/// Rows with empty SMILES or `MAIN_CLASS` are dropped.
///
/// # Errors
///
/// Returns an error if the CSV cannot be parsed or required columns are missing.
///
/// # Panics
///
/// Panics if `headers()` returns `None` when column indices are found.
pub fn parse_csv(csv_text: &str, columns: &ColumnNames) -> Result<Dataset, SplitError> {
    let mut reader = ReaderBuilder::new().from_reader(csv_text.as_bytes());
    let headers = reader.headers()?.clone();

    let find = |name: &str| headers.iter().position(|h| h == name);
    let smiles_idx = find(&columns.smiles);
    let cat_idx = find(&columns.category);
    let main_idx = find(&columns.main_class);
    let sub_idx = find(&columns.subclass);

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
    if !missing.is_empty() {
        return Err(SplitError::MissingColumns(missing));
    }
    let (smiles_idx, cat_idx, main_idx, sub_idx) = (
        smiles_idx.unwrap(),
        cat_idx,
        main_idx.unwrap(),
        sub_idx.unwrap(),
    );

    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record?;
        let smiles = record.get(smiles_idx).unwrap_or("").trim();
        let category = cat_idx.map_or("", |i| record.get(i).unwrap_or("")).trim();
        let main_class = record.get(main_idx).unwrap_or("").trim();
        let subclass = record.get(sub_idx).unwrap_or("").trim();
        if smiles.is_empty() || main_class.is_empty() {
            continue;
        }
        rows.push(DatasetRow {
            smiles: smiles.to_string(),
            category: category.to_string(),
            main_class: main_class.to_string(),
            subclass: subclass.to_string(),
        });
    }

    Ok(Dataset { rows })
}

/// The positive/negative SMILES pair for one CATEGORY, `MAIN_CLASS` or SUBCLASS label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassSplit {
    pub level: Level,
    pub label: String,
    pub category: String,
    pub main_class: String,
    pub subclass: String,
    pub slug: String,
    pub positive: Vec<String>,
    pub negative: Vec<String>,
}

impl ClassSplit {
    #[must_use]
    pub const fn positive_count(&self) -> usize {
        self.positive.len()
    }

    #[must_use]
    pub const fn negative_count(&self) -> usize {
        self.negative.len()
    }
}

/// A class dropped from the split for having too few members.
#[derive(Debug, Clone)]
pub struct SkippedClass {
    pub level: Level,
    pub label: String,
    pub count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct SplitResult {
    pub classes: Vec<ClassSplit>,
    pub skipped: Vec<SkippedClass>,
}

/// LIPID MAPS category (family) rank order — used for deterministic sorting.
/// The CATEGORY column in the LMSD TSV contains these full names.
static LIPID_MAPS_CATEGORY_RANK: [(&str, usize); 8] = [
    ("Fatty Acyls", 0),
    ("Glycerolipids", 1),
    ("Glycerophospholipids", 2),
    ("Sphingolipids", 3),
    ("Sterol Lipids", 4),
    ("Prenol Lipids", 5),
    ("Saccharolipids", 6),
    ("Polyketides", 7),
];

/// Look up the rank of a LIPID MAPS category name.
fn category_rank(category: &str) -> usize {
    LIPID_MAPS_CATEGORY_RANK
        .iter()
        .find(|(name, _)| *name == category)
        .map_or(99, |(_, rank)| *rank)
}

/// Build every CATEGORY, `MAIN_CLASS` and SUBCLASS positive/negative pair from
/// `dataset` according to `config`. Deterministic for a given
/// `(dataset, config)`.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn split_dataset(dataset: &Dataset, config: &SplitConfig) -> SplitResult {
    let mut rng = StdRng::seed_from_u64(config.seed);

    let mut all_smiles: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for row in &dataset.rows {
        if seen.insert(row.smiles.clone()) {
            all_smiles.push(row.smiles.clone());
        }
    }

    let mut by_cat: HashMap<String, Vec<String>> = HashMap::new();
    let mut by_main: HashMap<String, Vec<String>> = HashMap::new();
    let mut by_sub: HashMap<(String, String), Vec<String>> = HashMap::new();
    for row in &dataset.rows {
        if !row.category.is_empty() {
            by_cat
                .entry(row.category.clone())
                .or_default()
                .push(row.smiles.clone());
        }
        by_main
            .entry(row.main_class.clone())
            .or_default()
            .push(row.smiles.clone());
        if !row.subclass.is_empty() {
            by_sub
                .entry((row.main_class.clone(), row.subclass.clone()))
                .or_default()
                .push(row.smiles.clone());
        }
    }

    let mut classes = Vec::new();
    let mut skipped = Vec::new();
    let mut slug_owner: HashMap<String, String> = HashMap::new();

    // --- CATEGORY pairs, sorted by LIPID MAPS family rank then name ---
    let mut cat_labels: Vec<String> = by_cat.keys().cloned().collect();
    cat_labels.sort_by_key(|c| (category_rank(c), c.clone()));
    for category in cat_labels {
        let positives = by_cat.get(&category).cloned().unwrap_or_default();
        if positives.len() < config.min_positive {
            skipped.push(SkippedClass {
                level: Level::Category,
                label: category.clone(),
                count: positives.len(),
            });
            continue;
        }
        let slug = dedupe_slug(slugify(&category), &category, &mut slug_owner);

        let pos_set: HashSet<&String> = positives.iter().collect();
        let neg_pool: Vec<String> = all_smiles
            .iter()
            .filter(|s| !pos_set.contains(s))
            .cloned()
            .collect();
        let target = target_negative_count(positives.len(), config);
        let negative = sample(&neg_pool, target, &mut rng);

        classes.push(ClassSplit {
            level: Level::Category,
            label: category.clone(),
            category: category.clone(),
            main_class: String::new(),
            subclass: String::new(),
            slug,
            positive: positives,
            negative,
        });
    }

    // --- MAIN_CLASS pairs, sorted by category rank then name ---
    let mut main_labels: Vec<String> = by_main.keys().cloned().collect();
    main_labels.sort_by(|a, b| {
        // Group main classes by their category (first 2 chars → family rank)
        let (cat_a, cat_b) = (category_of_main(a), category_of_main(b));
        category_rank(cat_a)
            .cmp(&category_rank(cat_b))
            .then_with(|| a.cmp(b))
    });
    for main in main_labels {
        let positives = by_main.get(&main).cloned().unwrap_or_default();
        if positives.len() < config.min_positive {
            skipped.push(SkippedClass {
                level: Level::MainClass,
                label: main.clone(),
                count: positives.len(),
            });
            continue;
        }
        let slug = dedupe_slug(slugify(&main), &main, &mut slug_owner);

        let pos_set: HashSet<&String> = positives.iter().collect();
        let neg_pool: Vec<String> = all_smiles
            .iter()
            .filter(|s| !pos_set.contains(s))
            .cloned()
            .collect();
        let target = target_negative_count(positives.len(), config);
        let negative = sample(&neg_pool, target, &mut rng);

        classes.push(ClassSplit {
            level: Level::MainClass,
            label: main.clone(),
            category: String::new(),
            main_class: main,
            subclass: String::new(),
            slug,
            positive: positives,
            negative,
        });
    }

    // --- SUBCLASS pairs, sorted for determinism ---
    let mut sub_labels: Vec<(String, String)> = by_sub.keys().cloned().collect();
    sub_labels.sort();
    for (main, sub) in sub_labels {
        let positives = by_sub
            .get(&(main.clone(), sub.clone()))
            .cloned()
            .unwrap_or_default();
        if positives.len() < config.min_positive {
            skipped.push(SkippedClass {
                level: Level::Subclass,
                label: format!("{main}/{sub}"),
                count: positives.len(),
            });
            continue;
        }
        let slug_base = format!("{}_{}", slugify(&main), slugify(&sub));
        let slug = dedupe_slug(slug_base, &format!("{main}/{sub}"), &mut slug_owner);

        let pos_set: HashSet<&String> = positives.iter().collect();
        let neg_pool: Vec<String> = match config.subclass_negatives {
            SubclassNegatives::Siblings => by_main
                .get(&main)
                .into_iter()
                .flatten()
                .filter(|s| !pos_set.contains(s))
                .cloned()
                .collect(),
            SubclassNegatives::Global => all_smiles
                .iter()
                .filter(|s| !pos_set.contains(s))
                .cloned()
                .collect(),
        };
        let target = target_negative_count(positives.len(), config);
        let negative = sample(&neg_pool, target, &mut rng);

        classes.push(ClassSplit {
            level: Level::Subclass,
            label: sub.clone(),
            category: String::new(),
            main_class: main,
            subclass: sub,
            slug,
            positive: positives,
            negative,
        });
    }

    // Sort: categories first, then main classes, then subclasses
    classes.sort_by_key(|c| (c.level as u8, c.slug.clone()));

    SplitResult { classes, skipped }
}

/// Derive the LIPID MAPS category abbreviation from a `MAIN_CLASS` label
/// (e.g. `FA01` → `Fatty Acyls`). Returns the full category name.
fn category_of_main(main_class: &str) -> &'static str {
    let abbr = if main_class.len() >= 2 {
        &main_class[..2]
    } else {
        main_class
    };
    match abbr {
        "FA" => "Fatty Acyls",
        "GL" => "Glycerolipids",
        "GP" => "Glycerophospholipids",
        "SP" => "Sphingolipids",
        "ST" => "Sterol Lipids",
        "PR" => "Prenol Lipids",
        "SL" => "Saccharolipids",
        "PK" => "Polyketides",
        _ => "Other",
    }
}

fn target_negative_count(positive_count: usize, config: &SplitConfig) -> usize {
    // neg_ratio is a non-negative ratio and positive_count is non-negative,
    // so the rounded result is always non-negative.
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let scaled = (f64::from(u32::try_from(positive_count).unwrap_or(u32::MAX)) * config.neg_ratio)
        .round()
        .max(0.0) as usize;
    scaled.min(config.max_negatives)
}

/// Sample `k` items from `pool` without replacement (partial Fisher-Yates),
/// or the whole pool if it has `k` or fewer items.
fn sample(pool: &[String], k: usize, rng: &mut StdRng) -> Vec<String> {
    if k >= pool.len() {
        return pool.to_vec();
    }
    let mut indices: Vec<usize> = (0..pool.len()).collect();
    for i in 0..k {
        let j = i + rng.random_range(0..(pool.len() - i));
        indices.swap(i, j);
    }
    indices[..k].iter().map(|&i| pool[i].clone()).collect()
}

/// Lowercase, non-alphanumeric runs collapsed to `_`, trimmed, capped at
/// `maxlen` chars (with a short hash suffix on truncation).
fn slugify(label: &str) -> String {
    slugify_with_len(label, 60)
}

fn slugify_with_len(label: &str, maxlen: usize) -> String {
    let mut slug = String::with_capacity(label.len());
    let mut last_was_sep = false;
    for ch in label.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_sep = false;
        } else if !last_was_sep {
            slug.push('_');
            last_was_sep = true;
        }
    }
    let slug = slug.trim_matches('_').to_string();
    let slug = if slug.is_empty() {
        "unlabeled".to_string()
    } else {
        slug
    };

    if slug.len() > maxlen {
        let hash = short_hash(label);
        let keep = maxlen.saturating_sub(hash.len() + 1);
        format!("{}_{}", &slug[..keep.min(slug.len())], hash)
    } else {
        slug
    }
}

/// FNV-1a hash, rendered as 8 hex chars, for slug-collision suffixes.
fn short_hash(input: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{:08x}", (hash & 0xffff_ffff) as u32)
}

/// Ensure `slug` is unique within this split run.
fn dedupe_slug(slug: String, original_label: &str, owner: &mut HashMap<String, String>) -> String {
    if let Some(existing_owner) = owner.get(&slug) {
        if existing_owner == original_label {
            return slug;
        }
        let mut i = 2;
        loop {
            let candidate = format!("{slug}_{i}");
            if !owner.contains_key(&candidate) {
                owner.insert(candidate.clone(), original_label.to_string());
                return candidate;
            }
            i += 1;
        }
    }
    owner.insert(slug.clone(), original_label.to_string());
    slug
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_csv_basic() {
        let csv = "SMILES,CATEGORY,MAIN_CLASS,SUBCLASS\nC(C)C,Fatty Acyls,FA,FA01\nCCO,Glycerolipids,GL,\n";
        let dataset = parse_csv(csv, &ColumnNames::default()).unwrap();
        assert_eq!(dataset.rows.len(), 2);
        assert_eq!(dataset.rows[0].smiles, "C(C)C");
        assert_eq!(dataset.rows[0].category, "Fatty Acyls");
        assert_eq!(dataset.rows[0].main_class, "FA");
        assert_eq!(dataset.rows[0].subclass, "FA01");
    }

    #[test]
    fn parse_csv_skips_empty_smiles() {
        let csv = "SMILES,CATEGORY,MAIN_CLASS,SUBCLASS\n,,FA,\n,,GL,\nCCO,,GL,\n";
        let dataset = parse_csv(csv, &ColumnNames::default()).unwrap();
        assert_eq!(dataset.rows.len(), 1);
        assert_eq!(dataset.rows[0].smiles, "CCO");
    }

    #[test]
    fn parse_csv_missing_column() {
        let csv = "SMILES,MAIN_CLASS\nC(C)C,FA\n";
        let result = parse_csv(csv, &ColumnNames::default());
        assert!(result.is_err());
        match result {
            Err(SplitError::MissingColumns(cols)) => assert_eq!(cols, vec!["SUBCLASS"]),
            _ => panic!("expected MissingColumns error"),
        }
    }

    #[test]
    fn split_dataset_with_categories() {
        use std::fmt::Write;
        let csv = "SMILES,CATEGORY,MAIN_CLASS,SUBCLASS\n";
        let mut csv = csv.to_string();
        for i in 0..50 {
            writeln!(csv, "C{i}({i})C,Fatty Acyls,FA01,FA01").unwrap();
        }
        for i in 0..50 {
            writeln!(csv, "C{i}O,Glycerolipids,GL01,").unwrap();
        }

        let dataset = parse_csv(&csv, &ColumnNames::default()).unwrap();
        let config = SplitConfig {
            min_positive: 20,
            neg_ratio: 1.0,
            max_negatives: 100,
            ..Default::default()
        };
        let result = split_dataset(&dataset, &config);
        // 2 categories + 2 main classes + 1 subclass (FA01/FA01)
        assert_eq!(result.classes.len(), 5);
        assert_eq!(result.skipped.len(), 0);

        let cat_count = result
            .classes
            .iter()
            .filter(|c| c.level == Level::Category)
            .count();
        let main_count = result
            .classes
            .iter()
            .filter(|c| c.level == Level::MainClass)
            .count();
        assert_eq!(cat_count, 2);
        assert_eq!(main_count, 2);

        let fa_cat = result
            .classes
            .iter()
            .find(|c| c.level == Level::Category && c.label == "Fatty Acyls");
        assert!(fa_cat.is_some());
        let fa = fa_cat.unwrap();
        assert_eq!(fa.category, "Fatty Acyls");
        assert_eq!(fa.positive_count(), 50);
        assert_eq!(fa.negative_count(), 50);
    }

    #[test]
    fn split_dataset_skips_small_classes() {
        let mut csv = "SMILES,CATEGORY,MAIN_CLASS,SUBCLASS\nC1,Fatty Acyls,FAAA,\n".to_string();
        for _ in 0..10 {
            csv.push_str("C1,Fatty Acyls,FAAA,\n");
        }
        let dataset = parse_csv(&csv, &ColumnNames::default()).unwrap();
        let config = SplitConfig {
            min_positive: 20,
            ..Default::default()
        };
        let result = split_dataset(&dataset, &config);
        assert_eq!(result.classes.len(), 0);
        assert_eq!(result.skipped.len(), 2); // category + main_class both skipped
    }

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Fatty Acyls [FA01]"), "fatty_acyls_fa01");
        assert_eq!(slugify("  Spaced  "), "spaced");
        assert_eq!(slugify("!!!"), "unlabeled");
    }

    #[test]
    fn slugify_truncates_long_labels() {
        let long = "a".repeat(100);
        let slug = slugify(&long);
        assert!(slug.len() <= 60);
        assert!(slug.starts_with('a'));
    }

    #[test]
    fn target_negative_count_capped() {
        let config = SplitConfig {
            neg_ratio: 2.0,
            max_negatives: 10,
            ..Default::default()
        };
        assert_eq!(target_negative_count(100, &config), 10); // capped
        assert_eq!(target_negative_count(3, &config), 6); // 3 * 2 = 6
    }

    #[test]
    fn dedupe_slug_adds_suffix() {
        let mut owner = HashMap::new();
        let s1 = dedupe_slug("test".to_string(), "a", &mut owner);
        let s2 = dedupe_slug("test".to_string(), "b", &mut owner);
        assert_eq!(s1, "test");
        assert_eq!(s2, "test_2");
    }

    #[test]
    fn parse_csv_custom_columns() {
        let csv = "MOL,CLASS,SUBCLASS\nC(C)C,FA,FA01\n";
        let cols = ColumnNames {
            smiles: "MOL".to_string(),
            category: "CATEGORY".to_string(),
            main_class: "CLASS".to_string(),
            subclass: "SUBCLASS".to_string(),
        };
        let dataset = parse_csv(csv, &cols).unwrap();
        assert_eq!(dataset.rows.len(), 1);
        assert_eq!(dataset.rows[0].smiles, "C(C)C");
    }

    #[test]
    fn split_dataset_subclass_siblings() {
        // Two subclasses in same main class
        let csv = "SMILES,MAIN_CLASS,SUBCLASS\n";
        let mut csv = csv.to_string();
        for _ in 0..25 {
            csv.push_str("CCCC,GP,SUB1\n");
        }
        for _ in 0..25 {
            csv.push_str("CCCO,GP,SUB2\n");
        }

        let dataset = parse_csv(&csv, &ColumnNames::default()).unwrap();
        let config = SplitConfig {
            min_positive: 20,
            neg_ratio: 1.0,
            max_negatives: 100,
            subclass_negatives: SubclassNegatives::Siblings,
            ..Default::default()
        };
        let result = split_dataset(&dataset, &config);
        let sub1 = result
            .classes
            .iter()
            .find(|c| c.level == Level::Subclass && c.label == "SUB1");
        assert!(sub1.is_some());
        let sub1 = sub1.unwrap();
        // Positives: 25, Negatives: all from SUB2 (25) minus any duplicates
        assert_eq!(sub1.positive_count(), 25);
        assert!(sub1.negative_count() > 0);
    }
}

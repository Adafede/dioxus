// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

#[expect(
    clippy::unwrap_used,
    reason = "tests unwrap controlled fixtures to fail fast"
)]
#[expect(
    clippy::indexing_slicing,
    reason = "tests assert exact length before indexing fixtures"
)]
mod cases {
    use super::super::slug::{
        category_of_main, dedupe_slug, slugify, slugify_with_len, target_negative_count,
    };
    use super::super::*;
    use std::collections::HashMap;

    #[test]
    fn parse_lmsd_tsv_basic_and_skips_empty_smiles() {
        let tsv = "SMILES\tCATEGORY\tMAIN_CLASS\tSUB_CLASS\n\
                   C(C)C\tFatty Acyls\tFA\tFA01\n\
                   \t\tFA\t\n\
                   CCO\tGlycerolipids\tGL\t\n";
        let dataset = parse_lmsd_tsv(tsv).unwrap();
        assert_eq!(dataset.rows.len(), 2);
        assert_eq!(dataset.rows[0].smiles, "C(C)C");
        assert_eq!(dataset.rows[0].category, "Fatty Acyls");
        assert_eq!(dataset.rows[0].main_class, "FA");
        assert_eq!(dataset.rows[0].subclass, "FA01");
        assert_eq!(dataset.rows[1].smiles, "CCO");
    }

    #[test]
    fn split_dataset_with_categories() {
        use std::fmt::Write;
        let mut tsv = "SMILES\tCATEGORY\tMAIN_CLASS\tSUB_CLASS\n".to_string();
        for i in 0..50 {
            writeln!(tsv, "C{i}({i})C\tFatty Acyls\tFA01\tFA01").unwrap();
        }
        for i in 0..50 {
            writeln!(tsv, "C{i}O\tGlycerolipids\tGL01\t").unwrap();
        }

        let dataset = parse_lmsd_tsv(&tsv).unwrap();
        let config = SplitConfig {
            min_positive: 20,
            neg_ratio: 1.0,
            max_negatives: 100,
            ..Default::default()
        };
        let result = split_dataset(&dataset, &config);
        assert_eq!(result.classes.len(), 5);
        assert_eq!(result.skipped.len(), 0);

        let fa = result
            .classes
            .iter()
            .find(|c| c.level == Level::Category && c.label == "Fatty Acyls")
            .unwrap();
        assert_eq!(fa.positive_count(), 50);
        assert_eq!(fa.negative_count(), 50);
    }

    #[test]
    fn split_dataset_skips_small_classes() {
        let mut tsv =
            "SMILES\tCATEGORY\tMAIN_CLASS\tSUB_CLASS\nC1\tFatty Acyls\tFAAA\t\n".to_string();
        for _ in 0..10 {
            tsv.push_str("C1\tFatty Acyls\tFAAA\t\n");
        }
        let dataset = parse_lmsd_tsv(&tsv).unwrap();
        let config = SplitConfig {
            min_positive: 20,
            ..Default::default()
        };
        let result = split_dataset(&dataset, &config);
        assert_eq!(result.classes.len(), 0);
        assert_eq!(result.skipped.len(), 2);
    }

    #[test]
    fn slugify_basic_and_long_truncation() {
        assert_eq!(slugify("Fatty Acyls [FA01]"), "fatty_acyls_fa01");
        assert_eq!(slugify("  Spaced  "), "spaced");
        assert_eq!(slugify("!!!"), "unlabeled");
        let slug = slugify(&"a".repeat(100));
        assert!(slug.len() <= 60 && slug.starts_with('a'));
    }

    #[test]
    fn target_negative_count_and_dedupe_slug() {
        let config = SplitConfig {
            neg_ratio: 2.0,
            max_negatives: 10,
            ..Default::default()
        };
        assert_eq!(target_negative_count(100, &config), 10);
        assert_eq!(target_negative_count(3, &config), 6);

        let mut owner = HashMap::new();
        assert_eq!(dedupe_slug("test".to_string(), "a", &mut owner), "test");
        assert_eq!(dedupe_slug("test".to_string(), "b", &mut owner), "test_2");
    }

    #[test]
    fn split_dataset_subclass_siblings() {
        use std::fmt::Write as _;
        let mut tsv = "SMILES\tMAIN_CLASS\tSUB_CLASS\n".to_string();
        for i in 1..=25 {
            let _ = writeln!(tsv, "{}\tGP\tSUB1", "C".repeat(i));
            let _ = writeln!(tsv, "{}O\tGP\tSUB2", "C".repeat(i));
        }

        let dataset = parse_lmsd_tsv(&tsv).unwrap();
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
            .find(|c| c.level == Level::Subclass && c.label == "SUB1")
            .unwrap();
        assert_eq!(sub1.positive_count(), 25);
        assert!(sub1.negative_count() > 0);
    }

    #[test]
    fn category_and_slugify_handle_multibyte_utf8_safely() {
        assert_eq!(category_of_main("FA01"), "Fatty Acyls");
        assert_eq!(
            category_of_main("Fatty Acids and Conjugates [FA01]"),
            "Fatty Acyls"
        );
        assert_eq!(category_rank("Fatty Acyls [FA]"), 0);
        assert_eq!(category_of_main("Aµ01"), "Other");
        assert_eq!(category_of_main("µA01"), "Other");
        assert_eq!(category_of_main(""), "Other");

        let unicode_label = format!("Fatty Acyls {} {}", "α".repeat(30), "b".repeat(80));
        let truncated = slugify_with_len(&unicode_label, 30);
        assert!(
            truncated.len() <= 30,
            "truncated exceeded maxlen: {truncated}"
        );

        let short_cap = slugify_with_len("very_long_lipid_class_name", 5);
        assert!(short_cap.starts_with('_'), "unexpected slug: {short_cap}");
    }

    #[test]
    fn split_dataset_deduplicates_duplicate_smiles_within_class() {
        use std::fmt::Write as _;
        let mut tsv = "SMILES\tCATEGORY\tMAIN_CLASS\tSUB_CLASS\n".to_string();
        for _ in 0..30 {
            tsv.push_str("CCCC\tFatty Acyls [FA]\tFatty Acids and Conjugates [FA01]\t\n");
        }
        for i in 0..25 {
            let _ = writeln!(
                tsv,
                "C{}O\tGlycerolipids [GL]\tMonoradylglycerols [GL01]\t",
                "C".repeat(i + 1)
            );
        }
        let dataset = crate::parse_lmsd_tsv(&tsv).unwrap();
        let result = split_dataset(&dataset, &SplitConfig::default());
        assert!(
            result
                .classes
                .iter()
                .all(|c| c.main_class != "Fatty Acids and Conjugates [FA01]"),
            "30 duplicates of 1 SMILES must not meet min_positive=20"
        );
        assert!(
            result
                .classes
                .iter()
                .any(|c| c.category == "Glycerolipids [GL]" && c.positive_count() == 25)
        );
    }

    #[test]
    fn parse_lmsd_tsv_handles_empty_missing_columns_and_optional_category() {
        assert!(matches!(
            crate::parse_lmsd_tsv(""),
            Err(crate::CliError::EmptyInput)
        ));
        assert!(matches!(
            crate::parse_lmsd_tsv("SMILES\tCATEGORY\nCCO\tFA\n"),
            Err(crate::CliError::MissingColumn("MAIN_CLASS"))
        ));
        let tsv = "SMILES\tMAIN_CLASS\tSUB_CLASS\n  CCO  \t FA01 \t \n \t FA01 \t \n CCC \t \t \n";
        let ds = crate::parse_lmsd_tsv(tsv).unwrap();
        assert_eq!(ds.rows.len(), 1);
        assert_eq!(ds.rows[0].smiles, "CCO");
        assert_eq!(ds.rows[0].category, "");
        assert_eq!(ds.rows[0].main_class, "FA01");
    }
}

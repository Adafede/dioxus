// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

#[expect(
    clippy::unwrap_used,
    reason = "tests unwrap controlled fixtures to fail fast"
)]
#[expect(
    clippy::indexing_slicing,
    reason = "tests assert exact length before indexing fixtures"
)]
#[expect(
    clippy::panic,
    reason = "panic is used as explicit mismatch failure in tests"
)]
mod cases {
    use super::super::slug::{
        category_of_main, dedupe_slug, slugify, slugify_with_len, target_negative_count,
    };
    use super::super::*;
    use std::collections::HashMap;

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
            Err(SplitError::MissingColumns(cols)) => assert_eq!(cols, vec!["SUB_CLASS"]),
            _ => panic!("expected MissingColumns error"),
        }
    }

    #[test]
    fn split_dataset_with_categories() {
        use std::fmt::Write;
        let mut csv = "SMILES,CATEGORY,MAIN_CLASS,SUBCLASS\n".to_string();
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
        use std::fmt::Write as _;
        let mut csv = "SMILES,MAIN_CLASS,SUB_CLASS\n".to_string();
        for i in 1..=25 {
            let _ = writeln!(csv, "{},GP,SUB1", "C".repeat(i));
            let _ = writeln!(csv, "{}O,GP,SUB2", "C".repeat(i));
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

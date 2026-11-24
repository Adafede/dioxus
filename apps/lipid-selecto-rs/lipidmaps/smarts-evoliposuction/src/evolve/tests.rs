// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

#[expect(
    clippy::unwrap_used,
    reason = "tests unwrap controlled fixtures to fail fast"
)]
mod cases {
    use super::super::observer::truncate_str;
    use super::super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, tempdir};

    #[test]
    fn read_smiles_lines_skips_empty() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "CCO").unwrap();
        writeln!(tmp).unwrap();
        writeln!(tmp, "C(C)C").unwrap();
        tmp.flush().unwrap();

        let lines = read_smiles_lines(tmp.path()).unwrap();
        assert_eq!(lines, vec!["CCO".to_string(), "C(C)C".to_string()]);
    }

    #[test]
    fn read_smiles_lines_missing_file_returns_err() {
        let result = read_smiles_lines(std::path::Path::new("/nonexistent/file.smiles"));
        assert!(result.is_err());
    }

    #[test]
    fn read_smiles_lines_propagates_utf8_read_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("invalid.smiles");
        std::fs::write(&path, b"CCO\n\xff\xfe\n").unwrap();
        let result = read_smiles_lines(&path);
        assert!(
            result.is_err(),
            "non-UTF-8 lines must return Err instead of silently stopping"
        );
    }

    #[test]
    fn prepare_all_counts_failures() {
        let lines = vec![
            "CCO".to_string(),
            "NOT_A_SMILES".to_string(),
            "C(C)C".to_string(),
        ];
        let (prepared, failed) = prepare_all(&lines);
        assert_eq!(prepared.len(), 2);
        assert_eq!(failed, 1);
    }

    #[test]
    fn truncate_str_respects_utf8_boundaries() {
        assert_eq!(truncate_str("αβγ", 0), "");
        assert_eq!(truncate_str("αβγ", 1), "");
        assert_eq!(truncate_str("αβγ", 2), "α");
        assert_eq!(truncate_str("αβγ", 3), "α");
        assert_eq!(truncate_str("αβγ", 4), "αβ");
        assert_eq!(truncate_str("αβγ", 100), "αβγ");
    }

    #[test]
    fn error_variants_yield_no_task_result() {
        let counts = ParseCounts::default();
        let empty = EvolutionResult::EmptyAfterParse(counts);
        let read_err = EvolutionResult::ReadError(counts);
        let evo_err = EvolutionResult::EvolutionError("boom".to_owned(), counts);
        assert!(empty.as_task_result().is_none());
        assert!(read_err.as_task_result().is_none());
        assert!(evo_err.as_task_result().is_none());
    }

    #[test]
    fn cmp_priority_sorts_records_with_nan_deterministically() {
        let make = |smarts: &str, mcc: f64, cov: f64, generation: u64| TestedSmartsRecord {
            smarts: smarts.to_owned(),
            mcc,
            smarts_len: smarts.len(),
            coverage_score: cov,
            limit_exceeded: false,
            generation,
        };

        let mut records = vec![
            make("g_late", 0.7, 0.5, 5),
            make("nan_cov", 0.7, f64::NAN, 1),
            make("nan_mcc", f64::NAN, 0.9, 0),
            make("best", 0.9, 0.2, 9),
            make("g_early", 0.7, 0.5, 1),
            make("cov_high", 0.7, 0.8, 4),
        ];
        for i in 0..18_u32 {
            let mcc = if i % 5 == 0 {
                f64::NAN
            } else {
                f64::from(i).mul_add(0.01, 0.1)
            };
            records.push(make(&format!("filler_{i}"), mcc, 0.1, u64::from(i)));
        }

        records.sort_by(TestedSmartsRecord::cmp_priority);
        let top_six: Vec<&str> = records.iter().take(6).map(|r| r.smarts.as_str()).collect();
        assert_eq!(
            top_six,
            vec![
                "nan_mcc",
                "filler_0",
                "filler_5",
                "filler_10",
                "filler_15",
                "best"
            ]
        );
    }
}

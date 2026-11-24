// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

use std::path::PathBuf;

use clap::Parser;
use rand::SeedableRng;
use rand::rngs::StdRng;
use smarts_evoliposuction::{
    EvolutionResult, ManifestRow, ParseCounts, already_done, open_results_writer,
};
use tempfile::tempdir;

use super::cli::ordering::order_rows_for_evolution;
use super::cli::report::process_evolution_result;
use super::cli::split::cap_and_shuffle;
use super::cli::{Args, Command};

#[test]
fn split_command_defaults_to_lmsd_sdf_tsv() {
    let args = Args::try_parse_from(["smarts-evoliposuction", "split"]).unwrap();
    match args.command {
        Command::Split { input, .. } => {
            assert_eq!(input, PathBuf::from("LMSD.sdf.tsv"));
        }
        other => panic!("unexpected command: {other:?}"),
    }
}

#[test]
fn already_done_only_skips_ok_rows() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("smarts_results.csv");
    std::fs::write(
        &out,
        "level,label,category,main_class,subclass,slug,positives_total,negatives_total,positives_filtered,negatives_filtered,smarts,mcc,coverage_score,smarts_len,generations,elapsed_s,status,tested_smarts_count,tested_smarts_path\n\
         main_class,FA01,FA,FA01,,fa01,10,10,10,10,CC,0.9,0.9,2,5,0.1,ok,1,x.csv\n\
         main_class,FA02,FA,FA02,,fa02,10,10,10,10,,,,,,,evolution_error,0,\n\
         main_class,FA03,FA,FA03,,fa03,10,10,0,0,,,,,,,read_error,0,\n\
         main_class,FA04,FA,FA04,,fa04,10,10,0,0,,,,,,,empty_after_parse,0,\n",
    )
    .unwrap();

    let done = already_done(&out);
    assert!(done.contains("fa01"));
    assert!(!done.contains("fa02"));
    assert!(!done.contains("fa03"));
    assert!(!done.contains("fa04"));
}

#[test]
fn open_results_writer_truncates_when_not_resuming_and_appends_on_resume() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("smarts_results.csv");
    std::fs::write(&out, "stale-header\nstale-row\n").unwrap();

    {
        let (mut w, need_headers) = open_results_writer(&out, false).unwrap();
        assert!(need_headers);
        w.write_record(["h1", "h2"]).unwrap();
        w.flush().unwrap();
    }
    let after_fresh = std::fs::read_to_string(&out).unwrap();
    assert_eq!(after_fresh, "h1,h2\n");

    {
        let (mut w, need_headers) = open_results_writer(&out, true).unwrap();
        assert!(!need_headers);
        w.write_record(["v1", "v2"]).unwrap();
        w.flush().unwrap();
    }
    let after_resume = std::fs::read_to_string(&out).unwrap();
    assert_eq!(after_resume, "h1,h2\nv1,v2\n");
}

#[test]
fn order_rows_for_evolution_orders_by_level_then_avg_smiles_len_then_slug() {
    let dir = tempdir().unwrap();
    let mk_row = |level: &str, slug: &str, pos_body: &str, neg_body: &str| {
        let pos = dir.path().join(format!("{slug}_pos.smiles"));
        let neg = dir.path().join(format!("{slug}_neg.smiles"));
        std::fs::write(&pos, pos_body).unwrap();
        std::fs::write(&neg, neg_body).unwrap();
        ManifestRow {
            level: level.to_string(),
            label: slug.to_string(),
            category: String::new(),
            main_class: String::new(),
            subclass: String::new(),
            slug: slug.to_string(),
            positive_path: pos.to_string_lossy().to_string(),
            negative_path: neg.to_string_lossy().to_string(),
            positive_count: "1".to_string(),
            negative_count: "1".to_string(),
        }
    };

    let rows = vec![
        mk_row("subclass", "sub_a", "CC\n", "CCC\n"),
        mk_row("main_class", "main_long", "CCCCCCCCCC\n", "CCCCCCCCCC\n"),
        mk_row("main_class", "main_b_tie", "CC\n", "CC\n"),
        mk_row("main_class", "main_a_tie", "CC\n", "CC\n"),
        mk_row("category", "cat_long", "CCCCCC\n", "CCCCCC\n"),
        mk_row("category", "cat_short", "C\n", "CC\n"),
    ];

    let ordered = order_rows_for_evolution(rows);
    let slugs: Vec<&str> = ordered.iter().map(|r| r.slug.as_str()).collect();
    assert_eq!(
        slugs,
        vec![
            "cat_short",
            "cat_long",
            "main_a_tie",
            "main_b_tie",
            "main_long",
            "sub_a",
        ]
    );
}

#[test]
fn cap_and_shuffle_respects_cap_and_seed() {
    let items: Vec<String> = (0..10).map(|i| format!("C{}", "C".repeat(i))).collect();
    let mut rng_a = StdRng::seed_from_u64(42);
    let mut rng_b = StdRng::seed_from_u64(42);

    let out_a = cap_and_shuffle(&items, 4, &mut rng_a);
    let out_b = cap_and_shuffle(&items, 4, &mut rng_b);
    assert_eq!(out_a.len(), 4);
    assert_eq!(out_a, out_b);
    for item in &out_a {
        assert!(items.contains(item));
    }
}

#[test]
fn process_evolution_result_maps_error_statuses_with_parse_counts() {
    let dir = tempdir().unwrap();
    let row = ManifestRow {
        level: "category".to_string(),
        label: "FA".to_string(),
        category: "Fatty Acyls".to_string(),
        main_class: String::new(),
        subclass: String::new(),
        slug: "fa".to_string(),
        positive_path: String::new(),
        negative_path: String::new(),
        positive_count: "5".to_string(),
        negative_count: "4".to_string(),
    };

    let counts_empty = ParseCounts {
        positive_total: 5,
        negative_total: 4,
        positive_failed: 1,
        negative_failed: 2,
    };
    let empty = process_evolution_result(
        &EvolutionResult::EmptyAfterParse(counts_empty),
        &row,
        dir.path(),
    )
    .unwrap();
    assert_eq!(
        (
            empty.positives_total,
            empty.negatives_total,
            empty.positives_filtered,
            empty.negatives_filtered,
            empty.status,
        ),
        (5, 4, 1, 2, "empty_after_parse")
    );

    let counts_read = ParseCounts {
        positive_total: 5,
        negative_total: 0,
        positive_failed: 0,
        negative_failed: 0,
    };
    let read_err =
        process_evolution_result(&EvolutionResult::ReadError(counts_read), &row, dir.path())
            .unwrap();
    assert_eq!(read_err.status, "read_error");

    let counts_evo = ParseCounts {
        positive_total: 5,
        negative_total: 4,
        positive_failed: 0,
        negative_failed: 0,
    };
    let evo_err = process_evolution_result(
        &EvolutionResult::EvolutionError("boom".to_string(), counts_evo),
        &row,
        dir.path(),
    )
    .unwrap();
    assert_eq!(evo_err.status, "evolution_error");
}

// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

use super::*;
use clap::Parser;
use tempfile::tempdir;

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

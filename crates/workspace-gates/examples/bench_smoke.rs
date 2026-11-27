// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Deterministic release-mode benchmark smoke harness across all core crates.

use std::collections::{BTreeSet, HashMap};
use std::fmt::Write as _;
use std::hint::black_box;
use std::time::Instant;

use futures_executor::block_on;
use mgf_precursor_erro_rs::plotting::{
    render_recalibration_diagnostic_histogram, render_recalibration_diagnostic_ppm,
};
use smellfish_core::csv::{format_csv_rows, parse_csv_rows};
use smellfish_core::model::{Enrichment, EnrichmentOutcome, RdkitInspectResponse, SourceSummary};
use smellfish_core::pipeline::{RowInspection, finalize_pass2, prepare_pass1};
use stream_io::{BlobLines, SliceSource};

fn measure<T>(name: &str, mut f: impl FnMut() -> T) {
    // One warmup pass, then take median of 3 runs.
    drop(black_box(f()));
    let mut times_ms = [0.0f64; 3];
    for slot in &mut times_ms {
        let start = Instant::now();
        drop(black_box(f()));
        *slot = start.elapsed().as_secs_f64() * 1000.0;
    }
    times_ms.sort_by(f64::total_cmp);
    println!("{name:<36} {:>9.3} ms", times_ms[1]);
}

fn build_mgf_corpus(spectra: usize) -> String {
    let mut out = String::with_capacity(spectra * 160);
    for i in 0..spectra {
        let mz = 300.0 + f64::from(u32::try_from(i % 400).unwrap_or(0)) * 0.5;
        let _ = writeln!(
            out,
            "BEGIN IONS\nTITLE=spec_{i}\nPEPMASS={:.5} 5000.0\nFORMULA=C16H32O2\nSMILES=CCCCCCCCCCCCCCCC(=O)O\nADDUCT=[M+H]+\n{:.5} 1000.0\n120.080 500.0\n180.100 250.0\nEND IONS",
            mz + 1.007_276 + 0.002,
            mz + 1.007_276 + 0.005,
        );
    }
    out
}

fn build_json_array(rows: usize) -> String {
    let mut out = String::with_capacity(rows * 96);
    out.push('[');
    for i in 0..rows {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(
            out,
            r#"{{"id":"row_{i}","score":{i},"note":"alpha\\nbeta","nested":{{"k":null,"v":[1,2,3]}}}}"#
        );
    }
    out.push(']');
    out
}

fn bench_stream_and_mgf() {
    let line_corpus = "150.12345 1000.0\n".repeat(500_000);
    measure("stream_io_500k_lines", || {
        block_on(async {
            let mut reader = BlobLines::new(SliceSource::new(line_corpus.as_bytes()), |_, _| {});
            let mut count = 0usize;
            while let Ok(Some(line)) = reader.next_line().await {
                count += line.len();
            }
            count
        })
    });

    let mgf_corpus = build_mgf_corpus(10_000);
    measure("mgf_parse_10k_spectra", || {
        let mut parser = mgf::MgfLineParser::new();
        let mut count = 0usize;
        for line in mgf_corpus.lines() {
            if let Some(b) = parser.push_line(line) {
                count += b.peaks.len();
            }
        }
        count
    });
}

fn bench_json_and_cxsmiles() {
    let json_corpus = build_json_array(50_000);
    measure("json_count_50k_rows", || {
        block_on(json_count_core::scan_source_with_progress(
            SliceSource::new(json_corpus.as_bytes()),
            u64::try_from(json_corpus.len()).unwrap_or(u64::MAX),
            |_, _| {},
        ))
        .map_or(0, |cols| cols.len())
    });

    let pos_inputs: Vec<String> = vec![
        "Clc1ccccc1-c2ccccc2".to_string(),
        "Clc1cccc(-c2ccccc2)c1".to_string(),
        "Clc1ccc(-c2ccccc2)cc1".to_string(),
    ];
    let rep_inputs: Vec<String> = vec![
        "OC(=O)C(F)(F)C(F)F".to_string(),
        "OC(=O)C(F)(F)C(F)(F)C(F)F".to_string(),
        "OC(=O)C(F)(F)C(F)(F)C(F)(F)C(F)F".to_string(),
    ];
    measure("cxsmiles_pos_and_repeat", || {
        let a = cxsmiles_core::generate(&pos_inputs);
        let b = cxsmiles_core::generate(&rep_inputs);
        (a.is_ok(), b.is_ok())
    });
}

fn bench_lipid_and_mgf_analysis() {
    let lipid_text = lipid_core::example_smiles().join("\n");
    let classes = lipid_core::lmsd_classes();
    measure("lipid_core_204_classify_export", || {
        let mut ext = lipid_core::BlockExtractor::default();
        for line in lipid_text.lines() {
            ext.push_line(line);
        }
        let mut blocks = ext.finish();
        lipid_core::classify_blocks(&mut blocks, classes);
        let analysis = lipid_core::build_analysis_from_classified(blocks, 96, classes);
        let criteria = lipid_core::FilterCriteria {
            selected_classes: lipid_core::ClassMatchSet::all(classes.len()),
            mz_min: 0.0,
            mz_max: 5000.0,
            precursor_min: 0.0,
            precursor_max: 5000.0,
            adduct: "",
        };
        let smi = lipid_core::build_filtered_smiles_content(
            &analysis.blocks,
            &analysis.gallery,
            classes,
            &criteria,
        );
        smi.len()
    });

    let mgf_small = build_mgf_corpus(5_000);
    measure("mgf_analysis_5k_ingest_recalibrated", || {
        let mut ingestor = mgf_analysis_core::LineIngestor::default();
        for line in mgf_small.lines() {
            ingestor.feed_line(line);
        }
        let stats = ingestor.finish();
        let model = mgf_analysis_core::CalibrationModel::orbitrap(0.8);
        let diag = mgf_analysis_core::compute_recalibration_diagnostics(&stats, model);
        let recalibrated = mgf_analysis_core::generate_recalibrated_mgf(&mgf_small, model);
        (
            stats.spectra,
            diag.map(|d| d.total_count),
            recalibrated.len(),
        )
    });
}

fn bench_smellfish_and_plotting() {
    let mut csv_in = String::from("name,smiles\n");
    for i in 0..200 {
        let _ = writeln!(csv_in, "mol_{i},CN1C=NC2=C1C(=O)N(C(=O)N2C)C");
    }
    let parsed_rows = parse_csv_rows(&csv_in).unwrap_or_default();
    let sample_svg = "<svg xmlns=\"http://www.w3.org/2000/svg\"><path d=\"M0 0\"/></svg>";
    measure("smellfish_200_pipeline_csv_svg", || {
        let inspections: Vec<RowInspection> = parsed_rows
            .iter()
            .cloned()
            .map(|raw| RowInspection {
                raw,
                outcome: Ok(RdkitInspectResponse {
                    inchikey: Some("RYYVLZVUVIJVGH-UHFFFAOYSA-N".to_string()),
                    svg: Some(sample_svg.to_string()),
                    np_score: Some(2.1),
                    np_confidence: Some(0.85),
                    ..RdkitInspectResponse::default()
                }),
            })
            .collect();
        let batch = prepare_pass1(inspections);
        let mut lotus = HashMap::new();
        lotus.insert(
            "RYYVLZVUVIJVGH".to_string(),
            SourceSummary {
                taxa: BTreeSet::from(["Coffea arabica".to_string()]),
                compounds: BTreeSet::from(["Q60235".to_string()]),
                cids: BTreeSet::new(),
                compounds_with_taxa: BTreeSet::from(["Q60235".to_string()]),
            },
        );
        let outcome = finalize_pass2(
            batch,
            EnrichmentOutcome {
                enrichment: Enrichment {
                    lotus,
                    pubchem: HashMap::new(),
                },
                endpoints: Vec::new(),
                warnings: Vec::new(),
            },
        );
        let csv_out = format_csv_rows(&outcome.rows);
        (outcome.rows.len(), csv_out.len())
    });

    let before_ppm: Vec<f64> = (0..5_000)
        .map(|i| (f64::from(i % 100) - 50.0) * 0.2)
        .collect();
    let after_ppm: Vec<f64> = (0..5_000)
        .map(|i| (f64::from(i % 100) - 50.0) * 0.05)
        .collect();
    measure("mgf_plotting_diag_and_hist", || {
        let p = render_recalibration_diagnostic_ppm(&before_ppm, &after_ppm).unwrap_or_default();
        let h = render_recalibration_diagnostic_histogram(&before_ppm, &after_ppm, 24)
            .unwrap_or_default();
        p.len() + h.len()
    });
}

fn main() {
    bench_stream_and_mgf();
    bench_json_and_cxsmiles();
    bench_lipid_and_mgf_analysis();
    bench_smellfish_and_plotting();
}

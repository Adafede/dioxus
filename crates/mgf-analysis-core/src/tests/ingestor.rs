// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

use std::collections::HashMap;

use mgf::{MgfBlock, MgfHeader};

use crate::ingestor::{ingest_lines, record_block};
use crate::{MAX_RAW_RETENTION_BYTES, PrecursorStats, WarningDetail, append_retained_line};

#[test]
fn keeps_full_precision_when_computing_error() {
    let block = MgfBlock {
        header: MgfHeader {
            pepmass: Some(100.1234),
            latest_precursor_mz: Some(100.1234),
            exact_mass: Some(100.123_456_789),
            charge: Some("1".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    let mut smiles_cache = HashMap::new();
    let mut formula_cache = HashMap::new();
    let mut metrics = PrecursorStats::default();
    record_block(&mut metrics, &block, &mut smiles_cache, &mut formula_cache);

    assert_eq!(metrics.spectra, 1);
    let expected_error = 100.1234_f64 - 100.123_456_789_f64;
    assert!((metrics.abs_error_da_mean - expected_error.abs()).abs() < 1e-12);
}

#[test]
fn pepmass_with_intensity_and_missing_are_handled_properly() {
    let scored = ingest_lines(
        "BEGIN IONS\nPEPMASS=400.0 1234.5\nEXACTMASS=400.0\nEND IONS".lines(),
        |_| {},
    );
    assert_eq!(scored.spectra, 1);
    assert_eq!(scored.observed_precursor_mean, 400.0);

    let non_numeric = ingest_lines("BEGIN IONS\nPEPMASS=unknown\nEND IONS".lines(), |_| {});
    assert_eq!(non_numeric.total_spectra, 0);
}

#[test]
fn a_leading_byte_order_mark_does_not_wipe_out_the_scan() {
    let stats = ingest_lines(
        "\u{feff}BEGIN IONS\nPEPMASS=500.0\nEND IONS".lines(),
        |_| {},
    );
    assert_eq!(stats.total_spectra, 1);
}

#[test]
fn lines_outside_a_block_are_ignored_and_only_terminated_blocks_count() {
    let stats = ingest_lines(
        "COM=some header\nPEPMASS=1.0\nBEGIN IONS\nPEPMASS=500.0\nEND IONS\nBEGIN IONS\nPEPMASS=600.0\n".lines(),
        |_| {},
    );
    assert_eq!(stats.total_spectra, 1);
}

#[test]
fn a_crlf_document_is_scored_the_same_as_a_lf_one() {
    let lf = ingest_lines("BEGIN IONS\nPEPMASS=500.0\nEND IONS".lines(), |_| {});
    let crlf = ingest_lines(
        "BEGIN IONS\r\nPEPMASS=500.0\r\nEND IONS\r\n".lines(),
        |_| {},
    );
    assert_eq!(crlf.total_spectra, 1);
    assert_eq!(lf.total_spectra, crlf.total_spectra);
}

#[test]
fn duplicate_unparsed_smiles_and_unrecognized_adducts_accumulate_in_place() {
    let input = "\
BEGIN IONS
PEPMASS=500.0
SMILES=not_a_smiles
FORMULA=C6H12O6_bad
END IONS
BEGIN IONS
PEPMASS=500.0
SMILES=not_a_smiles
END IONS
BEGIN IONS
PEPMASS=500.0
EXACTMASS=500.0
ADDUCT=[M+Bogus]+
END IONS
BEGIN IONS
PEPMASS=500.0
EXACTMASS=500.0
ADDUCT=[M+Bogus]+
END IONS";
    let stats = ingest_lines(input.lines(), |_| {});
    assert_eq!(stats.total_spectra, 4);
    assert_eq!(stats.skipped_spectra, 4);
    assert_eq!(stats.unparsed_smiles, 2);
    assert_eq!(
        stats.unparsed_smiles_warnings.get("not_a_smiles"),
        Some(&WarningDetail {
            count: 2,
            formula: Some("C6H12O6_bad".to_string()),
        })
    );
    assert_eq!(stats.unrecognized_adducts.get("[M+Bogus]+"), Some(&2));
}

#[test]
fn append_retained_line_caps_buffer_and_clears_when_limit_exceeded() {
    assert_eq!(MAX_RAW_RETENTION_BYTES, 64 * 1024 * 1024);
    let mut buf = String::new();
    let mut capped = false;
    append_retained_line(&mut buf, &mut capped, "BEGIN IONS", 32);
    append_retained_line(&mut buf, &mut capped, "TITLE=ok", 32);
    assert!(!capped);
    assert_eq!(buf, "BEGIN IONS\nTITLE=ok\n");

    append_retained_line(&mut buf, &mut capped, "1234567890123456", 32);
    assert!(capped);
    assert_eq!(buf, "");

    append_retained_line(&mut buf, &mut capped, "after_cap", 32);
    assert!(capped);
    assert_eq!(buf, "");
}

#[test]
fn lowercase_mgf_delimiters_and_keys_are_parsed() {
    let input = "\
begin ions
pepmass=501.007276
exactmass=500.0
charge=1+
adduct=[M+H]+
120.5 100.0
end ions";
    let stats = ingest_lines(input.lines(), |_| {});
    assert_eq!(stats.total_spectra, 1);
    assert_eq!(stats.spectra, 1);
}

#[test]
fn non_positive_expected_mass_and_missing_structure_do_not_poison_stats() {
    let input = "\
BEGIN IONS
PEPMASS=500.0
EXACTMASS=0.0
END IONS
BEGIN IONS
PEPMASS=500.0
END IONS";
    let stats = ingest_lines(input.lines(), |_| {});
    assert_eq!(stats.total_spectra, 2);
    assert_eq!(stats.skipped_spectra, 2);
    assert_eq!(stats.spectra, 0);
    assert_eq!(stats.unparsed_smiles, 0);
    assert!(stats.abs_error_ppm_mean.is_finite());
}

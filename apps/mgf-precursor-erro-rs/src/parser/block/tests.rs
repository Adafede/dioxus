// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

// The `tests` tests, extracted from `block.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `block` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;

#[test]
fn keeps_full_precision_when_computing_error() {
    let state = BlockParseState {
        observed_precursor_raw: Some("100.1234".to_string()),
        observed_precursor: Some(100.1234),
        reference_mass: Some(100.123_456_789),
        charge: Some("1".to_string()),
        ..Default::default()
    };

    let mut smiles_cache = HashMap::new();
    let mut formula_cache = HashMap::new();
    let mut metrics = PrecursorStats::default();

    record_block_state(&mut metrics, &state, &mut smiles_cache, &mut formula_cache);

    assert_eq!(metrics.spectra, 1);
    let expected_error = 100.1234_f64 - 100.123_456_789_f64;
    assert!((metrics.abs_error_da_mean - expected_error.abs()).abs() < 1e-12);
}

/// `PEPMASS`'s second field is the precursor intensity, so
/// `PEPMASS=400.0 1234.5` is well-formed MGF. Parsing the whole remainder as
/// one float failed, `observed_precursor` stayed `None`, and `record_block_state`
/// dropped the block ungently — not counted as skipped, no warning, and the
/// Summary panel showed fewer spectra than the file contained.
#[test]
fn pepmass_with_intensity_is_scored() {
    let mut state = BlockParseState::default();

    state.consume_line("PEPMASS=400.0 1234.5");

    assert_eq!(
        state.observed_precursor,
        Some(400.0),
        "the two-field PEPMASS lost its leading m/z, so the spectrum was dropped"
    );
    assert_eq!(
        state.observed_precursor_raw.as_deref(),
        Some("400.0 1234.5"),
        "the raw field should keep the intensity for display"
    );
}

#[test]
fn pepmass_without_intensity_still_parses() {
    let mut state = BlockParseState::default();

    state.consume_line("PEPMASS=500.0");

    assert_eq!(state.observed_precursor, Some(500.0));
}

#[test]
fn pepmass_with_a_nonnumeric_value_is_left_unset() {
    let mut state = BlockParseState::default();

    state.consume_line("PEPMASS=unknown");

    assert_eq!(
        state.observed_precursor, None,
        "a non-numeric PEPMASS must not fabricate a precursor"
    );
}

/// A UTF-8 BOM is the first byte of files written by Windows tooling and is
/// category Cf, so `trim` leaves it on. `"\u{feff}BEGIN IONS"` never matched the
/// literal, so `current_is_in_block` stayed false for the rest of the file and
/// every spectrum was discarded with no error and a Summary panel reading `0`.
#[test]
fn a_leading_byte_order_mark_does_not_wipe_out_the_scan() {
    let stats = ingest_lines(
        "\u{feff}BEGIN IONS\nPEPMASS=500.0\nEND IONS".lines(),
        |_| {},
    );

    assert_eq!(
        stats.total_spectra, 1,
        "a BOM at byte 0 silently discarded every spectrum with no error"
    );
}

/// The wasm entry point used to be the only place this state machine existed,
/// so a caller could not check any of the block-boundary rules without a
/// browser. These lock the two that decide whether a spectrum counts.
#[test]
fn lines_outside_a_block_are_ignored_and_only_terminated_blocks_count() {
    let stats = ingest_lines(
        "COM=some header\nPEPMASS=1.0\nBEGIN IONS\nPEPMASS=500.0\nEND IONS\nBEGIN IONS\nPEPMASS=600.0\n".lines(),
        |_| {},
    );

    assert_eq!(
        stats.total_spectra, 1,
        "an unterminated block was scored, or a header line was"
    );
}

#[test]
fn a_crlf_document_is_scored_the_same_as_a_lf_one() {
    let lf = ingest_lines("BEGIN IONS\nPEPMASS=500.0\nEND IONS".lines(), |_| {});
    let crlf = ingest_lines(
        "BEGIN IONS\r\nPEPMASS=500.0\r\nEND IONS\r\n".lines(),
        |_| {},
    );

    assert_eq!(
        crlf.total_spectra, 1,
        "CRLF terminators defeated block detection"
    );
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
    assert_eq!(buf, "", "oversized retention buffer must be cleared");

    append_retained_line(&mut buf, &mut capped, "after_cap", 32);
    assert!(capped);
    assert_eq!(buf, "");
}

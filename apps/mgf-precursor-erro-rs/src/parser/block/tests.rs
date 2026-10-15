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
        reference_mass_source: Some("EXACTMASS".to_string()),
        charge: Some("1".to_string()),
        ..Default::default()
    };

    let mut smiles_cache = HashMap::new();
    let mut formula_cache = HashMap::new();
    let mut logged_failures = HashSet::new();
    let mut plot_sample = None;

    let metrics = process_block_state(
        &state,
        &mut smiles_cache,
        &mut formula_cache,
        &mut logged_failures,
        &mut plot_sample,
    )
    .expect("metrics should be produced");

    let expected_error = 100.1234_f64 - 100.123_456_789_f64;
    assert!((metrics.sample_abs_error_da - expected_error.abs()).abs() < 1e-12);
}

/// `PEPMASS`'s second field is the precursor intensity, so
/// `PEPMASS=400.0 1234.5` is well-formed MGF. Parsing the whole remainder as
/// one float failed, `observed_precursor` stayed `None`, and `process_block_state`
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

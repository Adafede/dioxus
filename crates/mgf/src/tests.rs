// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::{
    HeaderCase, MgfLineParser, MgfOptions, PeakParseMode, UnterminatedBlockPolicy,
    extract_pepmass_from_line, is_ions_marker, parse_leading_float, parse_mgf_lines,
    parse_peak_line,
};

#[test]
fn strips_bom_and_parses_case_insensitive_headers() -> Result<(), &'static str> {
    let mgf =
        "\u{feff}begin ions\r\npepmass=400.5 999.0\r\nsmiles=CCO\r\n100.2 50.0\r\nend ions\r\n";
    let blocks = parse_mgf_lines(mgf.lines(), MgfOptions::standard());
    let [block] = blocks.as_slice() else {
        return Err("expected one BOM-prefixed block");
    };
    assert_eq!(
        block.header.effective_precursor_mz(),
        Some(400.5),
        "PEPMASS should parse after BOM"
    );
    assert_eq!(
        block.header.smiles.as_deref(),
        Some("CCO"),
        "SMILES should parse case-insensitively"
    );
    assert_eq!(block.peaks.len(), 1, "one peak expected");
    Ok(())
}

#[test]
fn inline_begin_headers_and_unterminated_flush_work() -> Result<(), &'static str> {
    let mgf = "BEGIN IONS SMILES=CCCC PEPMASS=250.1\nTITLE=alkane\n57.0 100.0\n";
    let blocks = parse_mgf_lines(mgf.lines(), MgfOptions::lipid_selection());
    let [block] = blocks.as_slice() else {
        return Err("expected one flushed block");
    };
    assert_eq!(
        block.header.smiles.as_deref(),
        Some("CCCC"),
        "inline SMILES should parse"
    );
    assert_eq!(
        block.header.title.as_deref(),
        Some("alkane"),
        "TITLE should parse"
    );
    assert_eq!(
        block.header.effective_precursor_mz(),
        Some(250.1),
        "inline PEPMASS should parse"
    );
    assert_eq!(block.raw_lines.len(), 3, "raw lines should be retained");
    Ok(())
}

#[test]
fn unterminated_drop_discards_trailing_block() {
    let mgf = "BEGIN IONS\nPEPMASS=100.0\n";
    let blocks = parse_mgf_lines(mgf.lines(), MgfOptions::standard());
    assert!(
        blocks.is_empty(),
        "standard preset must drop unclosed blocks"
    );
}

#[test]
fn case_sensitive_preset_ignores_lowercase_keys() -> Result<(), &'static str> {
    let mgf = "BEGIN IONS\nsmiles=CCO\nPEPMASS=123.4\nEND IONS\n";
    let mut opts = MgfOptions::standard();
    opts.header_case = HeaderCase::CaseSensitive;
    let blocks = parse_mgf_lines(mgf.lines(), opts);
    let [block] = blocks.as_slice() else {
        return Err("expected one block");
    };
    assert_eq!(
        block.header.smiles, None,
        "lowercase key should be ignored when case-sensitive"
    );
    assert_eq!(
        block.header.effective_precursor_mz(),
        Some(123.4),
        "uppercase PEPMASS should still parse"
    );
    Ok(())
}

#[test]
fn peak_modes_enforce_expected_constraints() {
    assert!(
        parse_peak_line("100.0 20.0", PeakParseMode::RequireMzAndIntensity).is_some(),
        "mz + intensity should parse"
    );
    assert!(
        parse_peak_line("100.0", PeakParseMode::RequireMzAndIntensity).is_none(),
        "missing intensity should be rejected"
    );
    assert!(
        parse_peak_line("0.0 20.0", PeakParseMode::RequirePositiveMz).is_none(),
        "non-positive mz should be rejected"
    );
    assert!(
        parse_peak_line("100.0 bad", PeakParseMode::RequirePositiveMz).is_some(),
        "RequirePositiveMz ignores non-numeric second field"
    );
    assert!(
        parse_peak_line("PEPMASS=100.0", PeakParseMode::RequirePositiveMz).is_none(),
        "header lines must not parse as peaks"
    );
}

#[test]
fn line_helpers_match_expected_inputs() {
    assert_eq!(
        parse_leading_float("  42.5  1000  "),
        Some(42.5),
        "leading float should ignore intensity"
    );
    assert_eq!(
        extract_pepmass_from_line("PRECURSOR_MZ=301.25"),
        Some(301.25),
        "PRECURSOR_MZ should parse"
    );
    assert_eq!(
        extract_pepmass_from_line("TITLE=abc"),
        None,
        "non-pepmass key should return None"
    );
    assert!(
        is_ions_marker("  begin ions SMILES=C  ", "BEGIN IONS"),
        "word-boundary marker should match"
    );
    assert!(
        !is_ions_marker("BEGINNING", "BEGIN IONS"),
        "prefix without word boundary must not match"
    );
}

#[test]
fn deterministic_property_crlf_and_lf_produce_identical_blocks() {
    for n in 1..=6u32 {
        let mut lf = String::new();
        let mut crlf = String::new();
        for i in 0..n {
            let offset = f64::from(i);
            let block = format!(
                "BEGIN IONS\nTITLE=spec_{i}\nPEPMASS={:.3} 100.0\nSMILES=C{}O\n{:.2} 50.0\nEND IONS\n",
                100.0 + offset,
                "C".repeat(i as usize),
                50.0 + offset
            );
            lf.push_str(&block);
            crlf.push_str(&block.replace('\n', "\r\n"));
        }
        let a = parse_mgf_lines(lf.lines(), MgfOptions::standard());
        let b = parse_mgf_lines(crlf.lines(), MgfOptions::standard());
        assert_eq!(a, b, "CRLF and LF must parse identically for n={n}");
        assert_eq!(a.len(), n as usize, "block count must match n={n}");
    }
}

#[test]
fn streaming_push_and_unterminated_policy_roundtrip() -> Result<(), &'static str> {
    let mut parser = MgfLineParser::new(MgfOptions {
        unterminated_block: UnterminatedBlockPolicy::Flush,
        ..MgfOptions::standard()
    });
    assert!(
        parser.push_line("BEGIN IONS").is_none(),
        "BEGIN IONS starts block"
    );
    assert!(
        parser.push_line("EXACTMASS=120.0").is_none(),
        "EXACTMASS stays inside block"
    );
    assert!(
        parser.push_line("MOLECULEMASS=119.0").is_none(),
        "MOLECULEMASS stays inside block"
    );
    let Some(flushed) = parser.finish() else {
        return Err("expected flushed block");
    };
    assert_eq!(
        flushed.header.reference_mass(),
        Some(120.0),
        "EXACTMASS should take precedence over MOLECULEMASS"
    );
    Ok(())
}

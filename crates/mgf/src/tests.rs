// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::{
    MgfLineParser, extract_pepmass_from_line, is_ions_marker, parse_leading_float,
    parser::{parse_mgf_lines, parse_peak_line},
};

#[test]
fn strips_bom_and_parses_case_insensitive_headers() -> Result<(), &'static str> {
    let mgf =
        "\u{feff}begin ions\r\npepmass=400.5 999.0\r\nsmiles=CCO\r\n100.2 50.0\r\nend ions\r\n";
    let blocks = parse_mgf_lines(mgf.lines());
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
fn unterminated_drop_discards_trailing_block() {
    let mgf = "BEGIN IONS\nPEPMASS=100.0\n";
    let blocks = parse_mgf_lines(mgf.lines());
    assert!(blocks.is_empty(), "parser must drop unclosed blocks");
}

#[test]
fn peak_validation_enforces_positive_mz_and_ignores_headers() {
    assert!(
        parse_peak_line("100.0 20.0").is_some(),
        "mz + intensity should parse"
    );
    assert!(
        parse_peak_line("100.0").is_none(),
        "single-token peak line without whitespace should be rejected"
    );
    assert!(
        parse_peak_line("0.0 20.0").is_none(),
        "non-positive mz should be rejected"
    );
    assert_eq!(
        parse_peak_line("100.0 bad").and_then(|p| p.intensity),
        None,
        "non-numeric second field yields None intensity"
    );
    assert!(
        parse_peak_line("NaN 20.0").is_none(),
        "NaN mz must be rejected"
    );
    assert!(
        parse_peak_line("inf 20.0").is_none(),
        "infinite mz must be rejected"
    );
    assert_eq!(
        parse_peak_line("100.0 NaN").and_then(|p| p.intensity),
        None,
        "non-finite intensity must yield None"
    );
    assert!(
        parse_peak_line("PEPMASS=100.0").is_none(),
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
        let a = parse_mgf_lines(lf.lines());
        let b = parse_mgf_lines(crlf.lines());
        assert_eq!(a, b, "CRLF and LF must parse identically for n={n}");
        assert_eq!(a.len(), n as usize, "block count must match n={n}");
    }
}

#[test]
fn exactmass_takes_precedence_over_moleculemass() -> Result<(), &'static str> {
    let mut parser = MgfLineParser::new();
    assert!(parser.push_line("BEGIN IONS").is_none());
    assert!(parser.push_line("EXACTMASS=120.0").is_none());
    assert!(parser.push_line("MOLECULEMASS=119.0").is_none());
    let Some(block) = parser.push_line("END IONS") else {
        return Err("expected closed block");
    };
    assert_eq!(
        block.header.reference_mass(),
        Some(120.0),
        "EXACTMASS should take precedence over MOLECULEMASS"
    );
    Ok(())
}

#[test]
fn non_finite_headers_and_malformed_precursor_do_not_clobber_valid_values()
-> Result<(), &'static str> {
    let mgf = "\
BEGIN IONS
PEPMASS=400.25 1000.0
PRECURSOR_MZ=invalid
EXACTMASS=NaN
MOLECULEMASS=inf
END IONS
";
    let blocks = parse_mgf_lines(mgf.lines());
    let [block] = blocks.as_slice() else {
        return Err("expected one block");
    };
    assert_eq!(
        block.header.effective_precursor_mz(),
        Some(400.25),
        "malformed PRECURSOR_MZ must not overwrite valid PEPMASS"
    );
    assert_eq!(
        block.header.reference_mass(),
        None,
        "non-finite EXACTMASS and MOLECULEMASS must be rejected"
    );
    assert_eq!(parse_leading_float("NaN"), None, "NaN must be rejected");
    assert_eq!(parse_leading_float("inf"), None, "Inf must be rejected");
    Ok(())
}

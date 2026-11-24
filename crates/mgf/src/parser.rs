// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Streaming MGF line parser implementation.

use crate::types::{
    HeaderCase, MgfBlock, MgfHeader, MgfOptions, MgfPeak, PeakParseMode, UnterminatedBlockPolicy,
};

/// Parses the first whitespace-separated float from an MGF header or peak field.
#[must_use]
pub fn parse_leading_float(text: &str) -> Option<f64> {
    text.split_whitespace().next()?.parse::<f64>().ok()
}

/// Returns `true` when `line` begins with `marker` (`"BEGIN IONS"` or `"END IONS"`)
/// on a word boundary (case-insensitive).
#[must_use]
pub fn is_ions_marker(line: &str, marker: &str) -> bool {
    let trimmed = line.trim();
    trimmed
        .get(..marker.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(marker))
        && trimmed
            .get(marker.len()..)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with(char::is_whitespace))
}

/// Extracts a precursor `m/z` value from a `PEPMASS=` or `PRECURSOR_MZ=` line.
#[must_use]
pub fn extract_pepmass_from_line(line: &str) -> Option<f64> {
    let (key, value) = line.trim().split_once('=')?;
    if key.eq_ignore_ascii_case("PRECURSOR_MZ") || key.eq_ignore_ascii_case("PEPMASS") {
        parse_leading_float(value)
    } else {
        None
    }
}

/// Parses a single MS2 peak line according to `mode`.
#[must_use]
pub fn parse_peak_line(line: &str, mode: PeakParseMode) -> Option<MgfPeak> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.contains('=') {
        return None;
    }
    match mode {
        PeakParseMode::RequirePositiveMz => {
            if !trimmed.contains(char::is_whitespace) {
                return None;
            }
            let mut fields = trimmed.split_whitespace();
            let mz = fields.next()?.parse::<f64>().ok()?;
            if mz <= 0.0 {
                return None;
            }
            let intensity = fields.next().and_then(|s| s.parse::<f64>().ok());
            Some(MgfPeak { mz, intensity })
        }
        PeakParseMode::RequireMzAndIntensity => {
            let mut fields = trimmed.split_whitespace();
            let mz = fields.next()?.parse::<f64>().ok()?;
            let intensity = fields.next()?.parse::<f64>().ok()?;
            Some(MgfPeak {
                mz,
                intensity: Some(intensity),
            })
        }
    }
}

fn key_matches(key: &str, target: &str, case: HeaderCase) -> bool {
    match case {
        HeaderCase::CaseInsensitive => key.eq_ignore_ascii_case(target),
        HeaderCase::CaseSensitive => key == target,
    }
}

fn assign_header_field(header: &mut MgfHeader, key: &str, value: &str, case: HeaderCase) {
    let trimmed = value.trim();
    if key_matches(key, "PEPMASS", case) {
        let parsed = parse_leading_float(value);
        header.pepmass = parsed;
        header.latest_precursor_mz = parsed;
    } else if key_matches(key, "PRECURSOR_MZ", case) {
        let parsed = match case {
            HeaderCase::CaseInsensitive => parse_leading_float(value),
            HeaderCase::CaseSensitive => trimmed.parse::<f64>().ok(),
        };
        header.precursor_mz = parsed;
        header.latest_precursor_mz = parsed;
    } else if key_matches(key, "EXACTMASS", case) {
        if let Ok(parsed) = trimmed.parse::<f64>() {
            header.exact_mass = Some(parsed);
        }
    } else if key_matches(key, "MOLECULEMASS", case) {
        if let Ok(parsed) = trimmed.parse::<f64>() {
            header.molecule_mass = Some(parsed);
        }
    } else if key_matches(key, "CHARGE", case) {
        header.charge = Some(value.to_string());
    } else if key_matches(key, "ADDUCT", case) {
        header.adduct = Some(value.to_string());
    } else if key_matches(key, "IONMODE", case) {
        header.ion_mode = Some(value.to_string());
    } else if key_matches(key, "SMILES", case) {
        header.smiles = Some(trimmed.to_string());
    } else if key_matches(key, "FORMULA", case) {
        header.formula = Some(trimmed.to_string());
    } else if key_matches(key, "TITLE", case)
        || ((key_matches(key, "NAME", case) || key_matches(key, "SCANS", case))
            && header.title.is_none())
    {
        header.title = Some(trimmed.to_string());
    }
}

/// Incremental line-by-line parser producing [`MgfBlock`] records.
#[derive(Debug)]
pub struct MgfLineParser {
    options: MgfOptions,
    line_index: usize,
    block_index: usize,
    current: Option<MgfBlock>,
}

impl MgfLineParser {
    /// Creates a new parser with the supplied [`MgfOptions`].
    #[must_use]
    pub const fn new(options: MgfOptions) -> Self {
        Self {
            options,
            line_index: 0,
            block_index: 0,
            current: None,
        }
    }

    /// Feeds one line into the parser, returning a completed [`MgfBlock`] when
    /// `END IONS` closes the current block.
    pub fn push_line(&mut self, line: &str) -> Option<MgfBlock> {
        let normalized = if self.line_index == 0 && self.options.strip_bom {
            line.trim_start_matches('\u{feff}')
        } else {
            line
        };
        self.line_index += 1;

        let trimmed = normalized.trim();
        if trimmed.is_empty() {
            return None;
        }

        if is_ions_marker(trimmed, "BEGIN IONS") {
            let previous = self.current.take();
            self.block_index += 1;
            let mut block = MgfBlock {
                index: self.block_index,
                ..MgfBlock::default()
            };
            if self.options.inline_begin_headers {
                for token in trimmed.split_whitespace() {
                    if token.eq_ignore_ascii_case("BEGIN") || token.eq_ignore_ascii_case("IONS") {
                        continue;
                    }
                    if let Some((k, v)) = token.split_once('=') {
                        assign_header_field(&mut block.header, k, v, self.options.header_case);
                    }
                }
            }
            if self.options.retain_raw_lines {
                block.raw_lines.push(normalized.to_string());
            }
            self.current = Some(block);
            return previous;
        }

        let block = self.current.as_mut()?;
        if self.options.retain_raw_lines {
            block.raw_lines.push(normalized.to_string());
        }

        if is_ions_marker(trimmed, "END IONS") {
            return self.current.take();
        }

        if let Some((key, value)) = trimmed.split_once('=') {
            assign_header_field(&mut block.header, key, value, self.options.header_case);
        } else if let Some(peak) = parse_peak_line(trimmed, self.options.peak_mode) {
            block.peaks.push(peak);
        }
        None
    }

    /// Finishes parsing, returning any unclosed block when
    /// [`UnterminatedBlockPolicy::Flush`] is configured.
    #[must_use]
    pub fn finish(mut self) -> Option<MgfBlock> {
        match self.options.unterminated_block {
            UnterminatedBlockPolicy::Drop => None,
            UnterminatedBlockPolicy::Flush => self.current.take(),
        }
    }
}

/// Parses an iterator of lines into a `Vec<MgfBlock>` using `options`.
#[must_use]
pub fn parse_mgf_lines<'a>(
    lines: impl IntoIterator<Item = &'a str>,
    options: MgfOptions,
) -> Vec<MgfBlock> {
    let mut parser = MgfLineParser::new(options);
    let mut blocks = Vec::new();
    for line in lines {
        if let Some(block) = parser.push_line(line) {
            blocks.push(block);
        }
    }
    if let Some(trailing) = parser.finish() {
        blocks.push(trailing);
    }
    blocks
}

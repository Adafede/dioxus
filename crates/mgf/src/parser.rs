// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Streaming MGF line parser implementation.

use crate::types::{MgfBlock, MgfHeader, MgfPeak};

/// Parses the first whitespace-separated finite float from an MGF header or peak field.
#[must_use]
pub fn parse_leading_float(text: &str) -> Option<f64> {
    text.split_whitespace()
        .next()?
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
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

/// Parses a single MS2 peak line requiring a finite positive `m/z` followed by whitespace.
#[must_use]
pub(crate) fn parse_peak_line(line: &str) -> Option<MgfPeak> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.contains('=') || !trimmed.contains(char::is_whitespace) {
        return None;
    }
    let mut fields = trimmed.split_whitespace();
    let mz = fields
        .next()?
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && *v > 0.0)?;
    let intensity = fields
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|v| v.is_finite());
    Some(MgfPeak { mz, intensity })
}

fn assign_header_field(header: &mut MgfHeader, key: &str, value: &str) {
    let trimmed = value.trim();
    if key.eq_ignore_ascii_case("PEPMASS") {
        if let Some(parsed) = parse_leading_float(value) {
            header.pepmass = Some(parsed);
            header.latest_precursor_mz = Some(parsed);
        }
    } else if key.eq_ignore_ascii_case("PRECURSOR_MZ") {
        if let Some(parsed) = parse_leading_float(value) {
            header.precursor_mz = Some(parsed);
            header.latest_precursor_mz = Some(parsed);
        }
    } else if key.eq_ignore_ascii_case("EXACTMASS") {
        if let Ok(parsed) = trimmed.parse::<f64>()
            && parsed.is_finite()
        {
            header.exact_mass = Some(parsed);
        }
    } else if key.eq_ignore_ascii_case("MOLECULEMASS") {
        if let Ok(parsed) = trimmed.parse::<f64>()
            && parsed.is_finite()
        {
            header.molecule_mass = Some(parsed);
        }
    } else if key.eq_ignore_ascii_case("CHARGE") {
        header.charge = Some(value.to_string());
    } else if key.eq_ignore_ascii_case("ADDUCT") {
        header.adduct = Some(value.to_string());
    } else if key.eq_ignore_ascii_case("IONMODE") {
        header.ion_mode = Some(value.to_string());
    } else if key.eq_ignore_ascii_case("SMILES") {
        header.smiles = Some(trimmed.to_string());
    } else if key.eq_ignore_ascii_case("FORMULA") {
        header.formula = Some(trimmed.to_string());
    } else if key.eq_ignore_ascii_case("TITLE")
        || ((key.eq_ignore_ascii_case("NAME") || key.eq_ignore_ascii_case("SCANS"))
            && header.title.is_none())
    {
        header.title = Some(trimmed.to_string());
    }
}

/// Incremental line-by-line parser producing [`MgfBlock`] records.
#[derive(Debug, Default)]
pub struct MgfLineParser {
    line_index: usize,
    block_index: usize,
    current: Option<MgfBlock>,
}

impl MgfLineParser {
    /// Creates a new streaming MGF parser.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            line_index: 0,
            block_index: 0,
            current: None,
        }
    }

    /// Feeds one line into the parser, returning a completed [`MgfBlock`] when
    /// `END IONS` closes the current block.
    pub fn push_line(&mut self, line: &str) -> Option<MgfBlock> {
        let normalized = if self.line_index == 0 {
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
            self.current = Some(MgfBlock {
                index: self.block_index,
                ..MgfBlock::default()
            });
            return previous;
        }

        let block = self.current.as_mut()?;

        if is_ions_marker(trimmed, "END IONS") {
            return self.current.take();
        }

        if let Some((key, value)) = trimmed.split_once('=') {
            assign_header_field(&mut block.header, key, value);
        } else if let Some(peak) = parse_peak_line(trimmed) {
            block.peaks.push(peak);
        }
        None
    }
}

/// Parses an iterator of lines into a `Vec<MgfBlock>`.
#[cfg(test)]
#[must_use]
pub(crate) fn parse_mgf_lines<'a>(lines: impl IntoIterator<Item = &'a str>) -> Vec<MgfBlock> {
    let mut parser = MgfLineParser::new();
    let mut blocks = Vec::new();
    for line in lines {
        if let Some(block) = parser.push_line(line) {
            blocks.push(block);
        }
    }
    blocks
}

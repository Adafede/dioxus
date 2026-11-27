// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Streaming MGF and SMILES line state machine producing [`SpectrumBlock`] records.

use chematic::core::Molecule;
use mgf::is_ions_marker;

use super::block::SpectrumBlock;
use crate::format::LipidFormat;

/// Streaming state machine turning MGF or SMILES lines into [`SpectrumBlock`] records.
#[derive(Debug, Default)]
pub struct BlockExtractor {
    blocks: Vec<SpectrumBlock>,
    index: usize,
    current: Option<SpectrumBlock>,
    format: Option<LipidFormat>,
    saw_mgf: bool,
}

impl BlockExtractor {
    /// Construct an extractor optionally constrained to a known [`LipidFormat`].
    #[must_use]
    pub fn with_format(format: Option<LipidFormat>) -> Self {
        Self {
            format,
            ..Self::default()
        }
    }

    /// Resolve detected format after ingesting lines.
    #[must_use]
    pub fn detected_format(&self) -> Option<LipidFormat> {
        self.format
            .or_else(|| (!self.blocks.is_empty()).then_some(LipidFormat::Smiles))
    }

    /// Feed a single line into the extractor.
    pub fn push_line(&mut self, line: &str) {
        if self.format == Some(LipidFormat::Smiles) {
            self.push_smiles_line(line);
            return;
        }
        if is_ions_marker(line, "BEGIN IONS") {
            self.begin_mgf_block(line);
            return;
        }
        if let Some(block) = self.current.as_mut() {
            if is_ions_marker(line, "END IONS") {
                block.raw.push_str(line);
                block.raw.push('\n');
                if let Some(finished) = self.current.take() {
                    self.blocks.push(finished);
                }
                return;
            }
            block.consume_metadata(line);
            block.raw.push_str(line);
            block.raw.push('\n');
        } else if self.format.is_none() && !is_ions_marker(line, "END IONS") {
            self.push_smiles_line(line);
        }
    }

    fn push_smiles_line(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return;
        }
        self.index += 1;
        if let Some(block) = parse_smiles_line(self.index, trimmed) {
            self.blocks.push(block);
        }
    }

    fn begin_mgf_block(&mut self, line: &str) {
        if !self.saw_mgf {
            self.saw_mgf = true;
            if self.format.is_none() {
                self.blocks.clear();
                self.index = 0;
                self.format = Some(LipidFormat::Mgf);
            }
        }
        if let Some(block) = self.current.take() {
            self.blocks.push(block);
        }
        self.index += 1;
        let mut block = SpectrumBlock::new(self.index);
        block.consume_inline_header(line);
        block.raw.push_str(line);
        block.raw.push('\n');
        self.current = Some(block);
    }

    /// Flush any open block and return all parsed records.
    #[must_use]
    pub fn finish(mut self) -> Vec<SpectrumBlock> {
        if let Some(block) = self.current.take() {
            self.blocks.push(block);
        }
        self.blocks
    }
}

fn try_parse_smiles(s: &str) -> Option<Molecule> {
    (!s.contains(char::is_whitespace))
        .then(|| chem_depict::parse_smiles_safe(s).ok())
        .flatten()
}

fn looks_like_id(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

fn parse_smiles_line(index: usize, trimmed: &str) -> Option<SpectrumBlock> {
    let (first, second) = split_smiles_columns(trimmed)?;
    let (smiles_str, id_str, parsed) = second.map_or_else(
        || (first, "", try_parse_smiles(first)),
        |sec| {
            let first_mol = try_parse_smiles(first);
            let sec_mol = try_parse_smiles(sec);
            match (first_mol, sec_mol) {
                (Some(mol), None) => (first, sec, Some(mol)),
                (None, Some(mol)) => (sec, first, Some(mol)),
                (Some(f_mol), Some(s_mol)) => {
                    if looks_like_id(first) && !looks_like_id(sec) {
                        (sec, first, Some(s_mol))
                    } else {
                        (first, sec, Some(f_mol))
                    }
                }
                (None, None) => (first, sec, None),
            }
        },
    );

    let mut block = SpectrumBlock::new(index);
    block.psm_smiles = Some(smiles_str.to_string());
    block.parsed_molecule = parsed;
    if !id_str.is_empty() {
        block.title = Some(id_str.to_string());
    }
    block.raw = format!("{trimmed}\n");
    Some(block)
}

fn split_smiles_columns(trimmed: &str) -> Option<(&str, Option<&str>)> {
    let (first, second) = if let Some((left, right)) = trimmed.split_once('\t') {
        let right_col = right
            .split('\t')
            .next()
            .map(str::trim)
            .filter(|s| !s.is_empty());
        (left.trim(), right_col)
    } else if let Some((left, right)) = trimmed.split_once(char::is_whitespace) {
        (left.trim(), Some(right.trim()).filter(|s| !s.is_empty()))
    } else {
        (trimmed.trim(), None)
    };
    (!first.is_empty()).then_some((first, second))
}

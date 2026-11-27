// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Browser download dispatch surfacing any error back into the UI status signal.

use dioxus::prelude::*;
use lipid_core::{
    Analysis, FilterCriteria, LipidFormat, build_filtered_mgf_content,
    build_filtered_smiles_content,
};

pub(super) fn download_filtered(
    format: LipidFormat,
    analysis: &Analysis,
    criteria: &FilterCriteria<'_>,
    mut status: Signal<String>,
) {
    let ext = format.extension();
    let (content, filename) = match format {
        LipidFormat::Mgf => (
            build_filtered_mgf_content(
                &analysis.blocks,
                &analysis.gallery,
                analysis.all_classes,
                criteria,
            ),
            format!("lipids_filtered.{ext}"),
        ),
        LipidFormat::Smiles => (
            build_filtered_smiles_content(
                &analysis.blocks,
                &analysis.gallery,
                analysis.all_classes,
                criteria,
            ),
            format!("lipids.{ext}"),
        ),
    };
    if let Err(err) = upload::download_text(&content, &filename) {
        status.set(format!("Download failed: {err}"));
    }
}

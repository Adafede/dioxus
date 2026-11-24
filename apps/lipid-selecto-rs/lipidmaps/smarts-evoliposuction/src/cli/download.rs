// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

//! Download and SDF-to-TSV conversion subcommand step.

use std::fs;
use std::path::Path;

use smarts_evoliposuction::{CliError, LIPIDMAPS_LMSD_URL, sdf_zip_to_tsv};

pub(crate) fn download_step(dest: &Path) -> Result<(), CliError> {
    let tsv_path = dest.with_extension("tsv");

    // Skip download if the zip already exists and is non-empty.
    if dest.exists() {
        let metadata = fs::metadata(dest)?;
        if metadata.len() > 0 {
            eprintln!(
                "[SKIP] {} already exists ({} bytes)",
                dest.display(),
                metadata.len()
            );
            convert_if_missing(dest, &tsv_path)?;
            return Ok(());
        }
    }

    eprintln!(" Downloading LIPID MAPS LMSD.sdf.zip from lipidmaps.org...");
    let client = reqwest::blocking::Client::new();
    let resp = client.get(LIPIDMAPS_LMSD_URL).send()?.error_for_status()?;
    let bytes = resp.bytes()?;
    fs::write(dest, &bytes)?;
    eprintln!(
        "[OK] Downloaded {} bytes to {}",
        bytes.len(),
        dest.display()
    );

    convert_and_write(dest, &tsv_path)
}

fn convert_if_missing(zip_path: &Path, tsv_path: &Path) -> Result<(), CliError> {
    if tsv_path.exists() {
        eprintln!("[OK] TSV already exists: {}", tsv_path.display());
        return Ok(());
    }
    eprintln!(" Converting existing SDF to TSV...");
    convert_and_write(zip_path, tsv_path)
}

fn convert_and_write(zip_path: &Path, tsv_path: &Path) -> Result<(), CliError> {
    eprintln!(" Converting SDF to TSV...");
    let (tsv, count) = sdf_zip_to_tsv(zip_path)?;
    fs::write(tsv_path, tsv)?;
    eprintln!("[OK] Converted: {count} records -> {}", tsv_path.display());
    Ok(())
}

// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

//! Download and SDF-to-TSV conversion subcommand step.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use smarts_evoliposuction::{CliError, LIPIDMAPS_LMSD_URL, sdf_zip_to_tsv};

fn temp_sibling(dest: &Path) -> PathBuf {
    let file_name = dest
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("output");
    dest.with_file_name(format!(".{file_name}.part"))
}

fn write_atomic(dest: &Path, bytes: &[u8]) -> Result<(), CliError> {
    let tmp = temp_sibling(dest);
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, dest)?;
    Ok(())
}

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
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(300))
        .build()?;
    let resp = client.get(LIPIDMAPS_LMSD_URL).send()?.error_for_status()?;
    let bytes = resp.bytes()?;
    write_atomic(dest, &bytes)?;
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
    write_atomic(tsv_path, tsv.as_bytes())?;
    eprintln!("[OK] Converted: {count} records -> {}", tsv_path.display());
    Ok(())
}

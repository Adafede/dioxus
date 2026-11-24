// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Manifest CSV reading/writing for the smarts-evoliposuction pipeline.
//!
//! The manifest is the contract between the `split` phase (which writes
//! positive/negative `.smiles` file pairs) and the `evolve` phase (which
//! reads them and runs `smarts-evolution`).
//!
//! Columns: `level, label, category, main_class, subclass, slug, positive_path,
//! negative_path, positive_count, negative_count`.

use std::path::Path;

use serde::{Deserialize, Serialize};

/// One row of the manifest CSV.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ManifestRow {
    pub level: String,
    pub label: String,
    #[serde(default)]
    pub category: String,
    pub main_class: String,
    pub subclass: String,
    pub slug: String,
    pub positive_path: String,
    pub negative_path: String,
    pub positive_count: String,
    pub negative_count: String,
}

/// Read a manifest CSV from `path` into a vector of [`ManifestRow`].
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed as CSV.
pub fn read_manifest(path: &Path) -> Result<Vec<ManifestRow>, csv::Error> {
    csv::Reader::from_path(path)?
        .deserialize::<ManifestRow>()
        .collect()
}

/// Write a manifest CSV to `path`, creating parent dirs if needed.
///
/// # Errors
///
/// Returns an error if the file cannot be written.
pub fn write_manifest(path: &Path, rows: &[ManifestRow]) -> Result<(), csv::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut writer = csv::Writer::from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
#[expect(clippy::unwrap_used)] // tests unwrap fixtures to fail-fast rather than assert on the error
#[expect(clippy::indexing_slicing)] // tests reach into fixture rows and columns by index
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn read_manifest_roundtrip() {
        let content = "level,label,category,main_class,subclass,slug,positive_path,negative_path,positive_count,negative_count\n\
main_class,FA,Fatty Acyls,FA,,fa,fa_pos.smiles,fa_neg.smiles,50,50\n";

        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(content.as_bytes()).unwrap();
        tmp.flush().unwrap();

        let rows = read_manifest(tmp.path()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].level, "main_class");
        assert_eq!(rows[0].label, "FA");
        assert_eq!(rows[0].slug, "fa");
        assert_eq!(rows[0].positive_path, "fa_pos.smiles");
    }

    #[test]
    fn read_manifest_missing_file() {
        let result = read_manifest(std::path::Path::new("/nonexistent/manifest.csv"));
        assert!(result.is_err());
    }

    #[test]
    fn read_manifest_without_category_defaults_empty() {
        // Manifests omitting the `category` column default it to "".
        let content = "level,label,main_class,subclass,slug,positive_path,negative_path,positive_count,negative_count\n\
main_class,FA,FA,,fa,fa_pos.smiles,fa_neg.smiles,50,50\n";
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(content.as_bytes()).unwrap();
        tmp.flush().unwrap();
        let rows = read_manifest(tmp.path()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].category, "");
        assert_eq!(rows[0].slug, "fa");
    }
}

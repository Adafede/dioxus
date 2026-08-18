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
#[allow(clippy::module_name_repetitions)]
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
    #[allow(dead_code)]
    pub positive_count: String,
    #[allow(dead_code)]
    pub negative_count: String,
}

/// Read a manifest CSV from `path` into a vector of [`ManifestRow`].
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed as CSV.
#[allow(clippy::module_name_repetitions)]
pub fn read_manifest(path: &Path) -> Result<Vec<ManifestRow>, csv::Error> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut rows = Vec::new();
    for record in reader.deserialize::<ManifestRow>() {
        rows.push(record?);
    }
    Ok(rows)
}

/// Write a manifest CSV to `path`, creating parent dirs if needed.
///
/// # Errors
///
/// Returns an error if the file cannot be written.
#[allow(clippy::module_name_repetitions)]
pub fn write_manifest(path: &Path, rows: &[ManifestRow]) -> Result<(), csv::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| csv::Error::from(std::io::Error::other(e)))?;
    }
    let file =
        std::fs::File::create(path).map_err(|e| csv::Error::from(std::io::Error::other(e)))?;
    let mut writer = csv::Writer::from_writer(file);
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
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
    fn read_manifest_backward_compatible_no_category() {
        // Older manifests may not have a `category` column — it should default to "".
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

//! Download LMSD.sdf.zip from `LipidMaps` and convert to unique TSV.
//!
//! Uses [`lipidsdl`] for the actual work — this module is a thin adapter
//! that ties the download URL to the SDF→TSV conversion.

use std::path::Path;

use lipidsdl::download::{Error, download_to, noop_progress};
use lipidsdl::sdf::lipidmaps::to_lmsd_tsv;

/// The canonical LMSD download URL.
pub const LIPIDMAPS_LMSD_URL: &str = "https://www.lipidmaps.org/files/?file=LMSD&ext=sdf.zip";

/// Download LMSD.sdf.zip to `dest` (skipped if already present).
///
/// # Errors
///
/// Returns an error if the download fails.
#[allow(clippy::module_name_repetitions)]
pub async fn download_lmsd(dest: &Path) -> Result<(), Error> {
    download_to(LIPIDMAPS_LMSD_URL, dest, noop_progress).await
}

/// Extract and convert the first entry of a zip archive to LMSD-format TSV.
///
/// `zip_path` should point to a `.sdf.zip` file. The function reads the
/// first `.sdf` entry, extracts the 19 LMSD columns, deduplicates by
/// non-empty `SMILES+LM_ID`, and returns the TSV text plus the record count.
///
/// On native targets this uses the [`zip`] crate for extraction.
///
/// # Errors
///
/// Returns an error if the zip file cannot be read or parsed.
#[allow(clippy::module_name_repetitions)]
pub fn sdf_zip_to_tsv(zip_path: &Path) -> Result<(String, usize), SdfConvertError> {
    let data = std::fs::read(zip_path)?;
    sdf_zip_to_tsv_from_bytes(&data)
}

/// Same as [`sdf_zip_to_tsv`] but reads from an in-memory byte buffer.
///
/// # Errors
///
/// Returns an error if the zip data cannot be parsed.
#[allow(clippy::module_name_repetitions)]
pub fn sdf_zip_to_tsv_from_bytes(data: &[u8]) -> Result<(String, usize), SdfConvertError> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(data))?;

    // Find the first .sdf entry in the archive.
    let mut entry_name: Option<String> = None;
    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
        let name = entry.name();
        if name.to_lowercase().ends_with(".sdf") {
            entry_name = Some(name.to_string());
            break;
        }
    }
    let entry_name = entry_name.ok_or(SdfConvertError::NoSdfEntry)?;

    let mut entry = archive.by_name(&entry_name)?;
    let mut sdf_text = String::new();
    std::io::Read::read_to_string(&mut entry, &mut sdf_text)?;

    let (tsv, count) = to_lmsd_tsv(&sdf_text);
    Ok((tsv, count))
}

#[derive(Debug, thiserror::Error)]
pub enum SdfConvertError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("no .sdf entry found in zip archive")]
    NoSdfEntry,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sample_sdf() -> String {
        r"  CDK     2DS

  3  2  0  0  0  0  0  0  0  0999 V2000
    1.0000    0.0000    0.0000 C  0  0  0  0  0  0  0  0  0  0  0  0
    2.0000    0.0000    0.0000 C  0  0  0  0  0  0  0  0  0  0  0  0
  1  2  1  0
M  END
> <LM_ID>
LMFA00000001

> <NAME>
Test FA

> <SMILES>
CC

> <MAIN_CLASS>
Fatty Acyls [FA01]

$$$$
"
        .to_string()
    }

    #[test]
    fn sdf_zip_to_tsv_from_bytes_converts() {
        use std::io::Write;
        let sdf_text = make_sample_sdf();
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buf);
            let options = zip::write::FileOptions::<()>::default()
                .compression_method(zip::CompressionMethod::Stored);
            zip.start_file("LMSD.sdf", options).unwrap();
            zip.write_all(sdf_text.as_bytes()).unwrap();
            zip.finish().unwrap();
        }
        let data = buf.into_inner();
        let (tsv, count) = sdf_zip_to_tsv_from_bytes(&data).unwrap();
        assert_eq!(count, 1);
        assert!(tsv.contains("LMFA00000001\tTest FA"));
    }

    #[test]
    fn sdf_zip_to_tsv_missing_file() {
        let result = sdf_zip_to_tsv(std::path::Path::new("/nonexistent.zip"));
        assert!(matches!(result, Err(SdfConvertError::Io(_))));
    }
}

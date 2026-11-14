// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Async file download utilities (native targets only).
//!
//! Uses `reqwest` + `tokio` to stream a file from a URL to disk, with
//! automatic skip-if-exists and a progress callback.

#![allow(clippy::module_name_repetitions)]

use std::path::Path;

use tokio::io::AsyncWriteExt;

/// Progress callback: receives `(bytes_downloaded, total_bytes)` on each chunk.
pub type ProgressFn = fn(usize, Option<usize>);

/// No-op progress callback — use when you don't care about progress.
pub const fn noop_progress(_: usize, _: Option<usize>) {}

/// Download `url` to `dest`, calling `on_progress` periodically.
///
/// Creates parent directories if they don't exist. If `dest` already exists,
/// the download is skipped (idempotent — safe to retry).
///
/// # Errors
///
/// Returns an error if the HTTP request fails, the response can't be read,
/// or the file can't be written.
pub async fn download_to(url: &str, dest: &Path, on_progress: ProgressFn) -> Result<(), Error> {
    if dest.exists() {
        on_progress(0, None);
        return Ok(());
    }

    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let resp = reqwest::get(url).await?;
    if !resp.status().is_success() {
        return Err(Error::HttpStatus(resp.status().as_u16(), url.to_string()));
    }

    let total = resp
        .content_length()
        .map(|v| usize::try_from(v).unwrap_or(usize::MAX));
    let mut file = tokio::fs::File::create(dest).await?;
    let mut downloaded: usize = 0;

    let mut stream = resp;
    while let Some(chunk) = stream.chunk().await? {
        file.write_all(&chunk).await?;
        downloaded += chunk.len();
        on_progress(downloaded, total);
    }

    file.flush().await?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("HTTP status {0} for {1}")]
    HttpStatus(u16, String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use tempfile::NamedTempFile;

    use super::*;

    static PROGRESS_CALLS: AtomicUsize = AtomicUsize::new(0);
    static LAST_BYTES: AtomicUsize = AtomicUsize::new(0);

    fn record_progress(downloaded: usize, _total: Option<usize>) {
        PROGRESS_CALLS.fetch_add(1, Ordering::Relaxed);
        LAST_BYTES.store(downloaded, Ordering::Relaxed);
    }

    #[tokio::test]
    async fn download_to_skips_http_when_file_already_exists()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut temp = NamedTempFile::new()?;
        std::io::Write::write_all(&mut temp, b"cached-sdf")?;

        PROGRESS_CALLS.store(0, Ordering::Relaxed);
        LAST_BYTES.store(usize::MAX, Ordering::Relaxed);
        noop_progress(0, None);

        download_to(
            "http://127.0.0.1:9/unreachable",
            temp.path(),
            record_progress,
        )
        .await?;

        let contents = std::fs::read(temp.path())?;
        assert_eq!(contents, b"cached-sdf", "existing file must stay intact");
        assert_eq!(
            PROGRESS_CALLS.load(Ordering::Relaxed),
            1,
            "cache hit should still emit initial progress callback"
        );
        assert_eq!(
            LAST_BYTES.load(Ordering::Relaxed),
            0,
            "cache hit reports zero downloaded bytes"
        );
        Ok(())
    }

    #[test]
    fn http_status_error_displays_status_and_url() {
        let err = Error::HttpStatus(404, "https://example.test/data.sdf".to_owned());
        let msg = err.to_string();
        assert!(msg.contains("404"), "status code missing from: {msg}");
        assert!(
            msg.contains("https://example.test/data.sdf"),
            "URL missing from: {msg}"
        );
    }
}

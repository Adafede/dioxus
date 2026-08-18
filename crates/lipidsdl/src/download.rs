//! Async file download utilities (native targets only).
//!
//! Uses `reqwest` + `tokio` to stream a file from a URL to disk, with
//! automatic skip-if-exists and a progress callback.
//!
//! This module silences the `module_name_repetitions` clippy lint because
//! `download_to` and `Error` are clear in context.

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
    use super::*;

    #[test]
    fn noop_progress_does_not_panic() {
        noop_progress(0, None);
    }
}

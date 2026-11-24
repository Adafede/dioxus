// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Browser download helpers — text and SVG content.

use crate::error::UploadError;

#[cfg(target_arch = "wasm32")]
use js_sys::Array;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use web_sys::{Blob, HtmlAnchorElement, Url};

#[cfg(target_arch = "wasm32")]
fn blob_url_from_str(content: &str, mime: &str) -> Result<String, UploadError> {
    let parts = Array::new();
    parts.push(&JsValue::from_str(content));

    let blob = {
        let options = web_sys::BlobPropertyBag::new();
        options.set_type(mime);
        Blob::new_with_str_sequence_and_options(&parts, &options)
            .or_else(|_| Blob::new_with_str_sequence(&parts))
    }
    .map_err(|e| UploadError::DownloadFailed(format!("failed to create blob: {e:?}")))?;

    Url::create_object_url_with_blob(&blob)
        .map_err(|e| UploadError::DownloadFailed(format!("failed to create object URL: {e:?}")))
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn download_with_mime(
    content: &str,
    filename: &str,
    mime: &str,
) -> Result<(), UploadError> {
    let safe_name = sanitize_filename(filename);
    let url = blob_url_from_str(content, mime)?;

    if let Err(err) = click_download_anchor(&url, &safe_name) {
        drop(Url::revoke_object_url(&url));
        return Err(err);
    }

    gloo_timers::callback::Timeout::new(1_000, move || {
        drop(Url::revoke_object_url(&url));
    })
    .forget();
    Ok(())
}

/// Triggers a browser download of `content` as a text file.
///
/// # Errors
/// Returns [`UploadError`] if the download cannot be triggered.
pub fn download_text(content: &str, filename: &str) -> Result<(), UploadError> {
    download_with_mime(content, filename, "text/plain;charset=utf-8")
}

/// Triggers a browser download of SVG markup as an `image/svg+xml` file.
///
/// # Errors
/// Returns [`UploadError`] if the download cannot be triggered.
pub fn download_svg(content: &str, filename: &str) -> Result<(), UploadError> {
    download_with_mime(content, filename, "image/svg+xml;charset=utf-8")
}

#[cfg(target_arch = "wasm32")]
fn click_download_anchor(href: &str, filename: &str) -> Result<(), UploadError> {
    let window = web_sys::window()
        .ok_or_else(|| UploadError::DownloadFailed("no window object".to_string()))?;
    let document = window
        .document()
        .ok_or_else(|| UploadError::DownloadFailed("no document object".to_string()))?;
    let anchor: HtmlAnchorElement = document
        .create_element("a")
        .map_err(|e| UploadError::DownloadFailed(format!("failed to create anchor: {e:?}")))?
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|e| UploadError::DownloadFailed(format!("failed to cast anchor: {e:?}")))?;

    anchor.set_href(href);
    anchor.set_download(filename);
    anchor.set_rel("noopener noreferrer");

    let body = document
        .body()
        .ok_or_else(|| UploadError::DownloadFailed("no document body".to_string()))?;
    body.append_child(&anchor)
        .map_err(|e| UploadError::DownloadFailed(format!("failed to append anchor: {e:?}")))?;
    anchor.click();
    drop(body.remove_child(&anchor));

    Ok(())
}

#[cfg(any(test, target_arch = "wasm32"))]
const MAX_FILENAME_BYTES: usize = 255;

#[cfg(any(test, target_arch = "wasm32"))]
fn is_windows_reserved_stem(stem: &str) -> bool {
    if stem.eq_ignore_ascii_case("CON")
        || stem.eq_ignore_ascii_case("PRN")
        || stem.eq_ignore_ascii_case("AUX")
        || stem.eq_ignore_ascii_case("NUL")
    {
        return true;
    }
    let prefix = stem.get(..3).unwrap_or("");
    let suffix = stem.get(3..).unwrap_or("");
    (prefix.eq_ignore_ascii_case("COM") || prefix.eq_ignore_ascii_case("LPT"))
        && suffix.len() == 1
        && matches!(suffix.as_bytes().first().copied(), Some(b'1'..=b'9'))
}

#[cfg(any(test, target_arch = "wasm32"))]
fn truncate_utf8_filename(name: String, max_bytes: usize) -> String {
    if name.len() <= max_bytes {
        return name;
    }
    if let Some((stem, ext)) = name.rsplit_once('.')
        && !stem.is_empty()
        && !ext.is_empty()
        && ext.len() + 1 < max_bytes
        && ext.len() <= 32
    {
        let cut = stem.floor_char_boundary(max_bytes - ext.len() - 1);
        if cut > 0 {
            let mut out = stem.get(..cut).unwrap_or_default().to_string();
            out.push('.');
            out.push_str(ext);
            return out;
        }
    }
    let cut = name.floor_char_boundary(max_bytes);
    let truncated = name.get(..cut).unwrap_or_default();
    if truncated.is_empty() {
        "download.txt".to_string()
    } else {
        truncated.to_string()
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
#[must_use]
pub(crate) fn sanitize_filename(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.trim().chars() {
        if c.is_control() {
            continue;
        }
        match c {
            '/' | '\\' | '"' | '\'' | ':' | '*' | '?' | '<' | '>' | '|' => out.push('_'),
            _ => out.push(c),
        }
    }
    let cleaned = out.trim_matches('.').trim();
    if cleaned.is_empty() {
        return "download.txt".to_string();
    }
    let stem = cleaned.split('.').next().unwrap_or("");
    let guarded = if is_windows_reserved_stem(stem) {
        format!("_{cleaned}")
    } else {
        cleaned.to_string()
    };
    truncate_utf8_filename(guarded, MAX_FILENAME_BYTES)
}

#[cfg(not(target_arch = "wasm32"))]
#[expect(
    clippy::missing_const_for_fn,
    reason = "matches non-const wasm32 DOM implementation"
)]
pub(crate) fn download_with_mime(
    _content: &str,
    _filename: &str,
    _mime: &str,
) -> Result<(), UploadError> {
    Err(UploadError::BrowserOnly)
}

#[cfg(test)]
mod tests;

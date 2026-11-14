// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Browser download helpers — text and SVG content.

#[cfg(target_arch = "wasm32")]
use js_sys::Array;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::{Blob, HtmlAnchorElement, Url};

/// Creates a `Blob` from a string and returns its object URL.
///
/// # Errors
/// Returns a message if the browser API call fails.
#[cfg(target_arch = "wasm32")]
fn blob_url_from_str(content: &str, mime: &str) -> Result<String, String> {
    let parts = Array::new();
    parts.push(&JsValue::from_str(content));

    let blob = {
        let options = web_sys::BlobPropertyBag::new();
        options.set_type(mime);
        Blob::new_with_str_sequence_and_options(&parts, &options)
            .or_else(|_| Blob::new_with_str_sequence(&parts))
    };
    let blob = blob.map_err(|e| format!("failed to create blob: {e:?}"))?;
    Url::create_object_url_with_blob(&blob)
        .map_err(|e| format!("failed to create object URL: {e:?}"))
}

/// Triggers a browser download of `content` with an explicit MIME type.
///
/// # Arguments
/// - `content`: The text content to download
/// - `filename`: The full filename (including extension) for the download
/// - `mime`: The MIME type (for example, `"image/svg+xml;charset=utf-8"`)
///
/// # Errors
/// Returns a message if the download cannot be triggered.
#[cfg(target_arch = "wasm32")]
pub fn download_with_mime(content: &str, filename: &str, mime: &str) -> Result<(), String> {
    let safe_name = sanitize_filename(filename);
    let url = blob_url_from_str(content, mime)?;

    if let Err(err) = click_download_anchor(&url, &safe_name) {
        drop(Url::revoke_object_url(&url));
        return Err(format!("download failed: {err}"));
    }

    // Defer revocation so the browser has time to initiate the download before
    // the object URL is invalidated.
    gloo_timers::callback::Timeout::new(1_000, move || {
        drop(Url::revoke_object_url(&url));
    })
    .forget();
    Ok(())
}

/// Triggers a browser download of `content` as a text file.
///
/// # Arguments
/// - `content`: The text content to download
/// - `filename`: The full filename (including extension) for the download
///
/// # Errors
/// Returns a message if the download cannot be triggered.
pub fn download_text(content: &str, filename: &str) -> Result<(), String> {
    download_with_mime(content, filename, "text/plain;charset=utf-8")
}

/// Triggers a browser download of SVG markup as an `image/svg+xml` file.
///
/// # Arguments
/// - `content`: The SVG XML markup to download
/// - `filename`: The full filename (including `.svg` extension) for the download
///
/// # Errors
/// Returns a message if the download cannot be triggered.
pub fn download_svg(content: &str, filename: &str) -> Result<(), String> {
    download_with_mime(content, filename, "image/svg+xml;charset=utf-8")
}

#[cfg(target_arch = "wasm32")]
fn click_download_anchor(href: &str, filename: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or("no window object")?;
    let document = window.document().ok_or("no document object")?;
    let anchor: HtmlAnchorElement = document
        .create_element("a")
        .map_err(|e| format!("failed to create anchor: {e:?}"))?
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|e| format!("failed to cast anchor: {e:?}"))?;

    anchor.set_href(href);
    anchor.set_download(filename);
    anchor.set_rel("noopener noreferrer");

    let body = document.body().ok_or("no document body")?;
    body.append_child(&anchor)
        .map_err(|e| format!("failed to append anchor: {e:?}"))?;
    anchor.click();
    // The anchor exists only to carry the click, so it is removed immediately
    // afterwards. `drop` rather than `let _ =` says the removal is the point;
    // the two are the same statement and `clippy::let_underscore_drop` reads
    // the second as an accidental early drop.
    drop(body.remove_child(&anchor));

    Ok(())
}

/// Sanitizes a filename for safe browser download.
///
/// Removes control characters, replaces path separators, quotes, and
/// Windows-reserved characters (`: * ? < > |`) with underscores, and falls
/// back to `"download.txt"` if nothing remains after trimming dots/spaces.
#[must_use]
#[cfg(any(target_arch = "wasm32", test))]
fn sanitize_filename(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.trim().chars() {
        // Runs first, so it is also what decides newlines: they are control
        // characters, so they are dropped here rather than reaching the
        // `_`-substitution below.
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
        "download.txt".to_string()
    } else {
        cleaned.to_string()
    }
}

/// Triggers a browser download with an explicit MIME type (native stub — returns `Err`).
///
/// On non-WASM targets browsers aren't available, so this always returns `Err`.
///
/// # Errors
/// Always returns an error on native targets.
#[cfg(not(target_arch = "wasm32"))]
pub fn download_with_mime(_content: &str, _filename: &str, _mime: &str) -> Result<(), String> {
    Err("Download is only available in the browser".to_string())
}

#[cfg(test)]
mod tests;

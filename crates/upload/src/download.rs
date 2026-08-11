// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Browser download helpers — text content, blobs, and URL-based downloads.
//!
//! Consolidates the per-app download patterns that were previously inlined in
//! `json-count-rs`, `mgf-precursor-erro-rs`, `lipid-selecto-rs`, and
//! `lotus-explore-rs`.

#[cfg(target_arch = "wasm32")]
use gloo_timers::future::TimeoutFuture;
#[cfg(target_arch = "wasm32")]
use js_sys::Array;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::{Blob, HtmlAnchorElement, Url};

/// Default skip-link style for keyboard navigation to main content.
pub const SKIP_LINK_STYLE: &str = "position:absolute;top:-100%;left:0.5rem;z-index:9999;padding:0.5rem 1rem;background:transparent;color:#0b5cab;font-size:0.875rem;font-weight:600;border-radius:0 0 4px 4px;text-decoration:underline;";

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

/// Triggers a browser download of `content` as a text file.
///
/// # Arguments
/// - `content`: The text content to download
/// - `filename`: The full filename (including extension) for the download
///
/// # Errors
/// Returns a message if the download cannot be triggered.
#[cfg(target_arch = "wasm32")]
pub fn download_text(content: &str, filename: &str) -> Result<(), String> {
    let safe_name = sanitize_filename(filename);
    let url = blob_url_from_str(content, "text/plain;charset=utf-8")?;

    click_download_anchor(&url, &safe_name, false)
        .map(|_| ())
        .map_err(|e| format!("download failed: {e}"))
}

/// Downloads content as a blob with a specific extension and MIME type.
///
/// # Arguments
/// - `content`: The text content to download
/// - `filename`: The base filename (without extension)
/// - `extension`: File extension including dot (e.g. `".csv"`, `".json"`)
/// - `mime`: MIME type for the blob
///
/// # Errors
/// Returns a message if the download cannot be triggered.
#[cfg(target_arch = "wasm32")]
pub fn download_text_as_blob(
    content: &str,
    filename: &str,
    extension: &str,
    mime: &str,
) -> Result<(), String> {
    let safe_name = if extension.is_empty() {
        sanitize_filename(filename)
    } else {
        let name = sanitize_filename(filename);
        if name.ends_with(extension) {
            name
        } else {
            format!("{name}{extension}")
        }
    };
    let url = blob_url_from_str(content, mime)?;
    click_download_anchor(&url, &safe_name, false)
        .map(|_| ())
        .map_err(|e| format!("download failed: {e}"))
}

/// Triggers a browser download of a URL (e.g. a QLever export URL or a remote file).
///
/// Opens the URL in a new tab / triggers an anchor click.  Returns `false` if
/// the browser does not support programmatic clicks (extremely rare).
#[cfg(target_arch = "wasm32")]
pub fn download_url(url: &str, filename: &str) -> bool {
    let safe_name = sanitize_filename(filename);
    click_download_anchor(&url, &safe_name, true).unwrap_or_else(|_| {
        web_sys::window()
            .and_then(|w| w.open_with_url(url).ok())
            .is_some()
    })
}

#[cfg(target_arch = "wasm32")]
fn click_download_anchor(href: &str, filename: &str, new_tab: bool) -> Result<bool, String> {
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
    if new_tab {
        anchor.set_target("_blank");
    }

    let body = document.body().ok_or("no document body")?;
    body.append_child(&anchor)
        .map_err(|e| format!("failed to append anchor: {e:?}"))?;
    let _ = body.remove_child(&anchor);

    Ok(true)
}

/// Submits a hidden form to trigger a browser download via POST.
///
/// Used when the URL + query payload is too large for a GET request.
///
/// # Errors
/// Returns a message if the form cannot be created or submitted.
#[cfg(target_arch = "wasm32")]
pub async fn submit_download_form(endpoint: &str, fields: &[(&str, &str)]) -> Result<(), String> {
    let window = web_sys::window().ok_or("no window object")?;
    let document = window.document().ok_or("no document object")?;

    let form = document
        .create_element("form")
        .map_err(|e| format!("failed to create form: {e:?}"))?
        .dyn_into::<web_sys::HtmlFormElement>()
        .map_err(|e| format!("failed to cast form: {e:?}"))?;
    form.set_method("POST");
    form.set_action(endpoint);
    form.set_target("_blank");
    let _ = form.set_attribute("accept-charset", "UTF-8");
    let _ = form.set_attribute("enctype", "application/x-www-form-urlencoded");

    for (name, value) in fields {
        let input = document
            .create_element("input")
            .map_err(|e| format!("failed to create input {name}: {e:?}"))?
            .dyn_into::<web_sys::HtmlInputElement>()
            .map_err(|e| format!("failed to cast input {name}: {e:?}"))?;
        input.set_type("hidden");
        input.set_name(name);
        input.set_value(value);
        form.append_child(&input)
            .map_err(|e| format!("failed to append input {name}: {e:?}"))?;
    }

    let body = document.body().ok_or("no document body")?;
    body.append_child(&form)
        .map_err(|e| format!("failed to append form: {e:?}"))?;
    form.submit()
        .map_err(|e| format!("failed to submit form: {e:?}"))?;
    let _ = body.remove_child(&form);

    // Yield so the form submission takes effect before the caller continues.
    #[allow(clippy::let_underscore_drop)]
    let _ = TimeoutFuture::new(0).await;
    Ok(())
}

/// Sanitizes a filename for safe browser download.
///
/// Removes control characters and replaces path separators and quotes with
/// underscores.  No external crate required.
#[must_use]
pub fn sanitize_filename(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.trim().chars() {
        if c.is_control() {
            continue;
        }
        match c {
            '/' | '\\' | '"' | '\'' | '\n' | '\r' => out.push('_'),
            _ => out.push(c),
        }
    }
    out.trim_matches('.').trim().to_string()
}

// ── Non-WASM stubs ───────────────────────────────────────────────────────────

/// Triggers a browser download of text content (native stub — returns `Err`).
///
/// On non-WASM targets browsers aren't available, so this always returns `Err`.
///
/// # Errors
/// Always returns an error on native targets.
#[cfg(not(target_arch = "wasm32"))]
pub fn download_text(_content: &str, _filename: &str) -> Result<(), String> {
    Err("Download is only available in the browser".to_string())
}

/// Downloads content as a blob (native stub — returns `Err`).
///
/// On non-WASM targets browsers aren't available, so this always returns `Err`.
///
/// # Errors
/// Always returns an error on native targets.
#[cfg(not(target_arch = "wasm32"))]
pub fn download_text_as_blob(
    _content: &str,
    _filename: &str,
    _extension: &str,
    _mime: &str,
) -> Result<(), String> {
    Err("Download is only available in the browser".to_string())
}

/// Triggers a browser download of a URL (native stub — returns `false`).
///
/// On non-WASM targets browsers aren't available, so this always returns `false`.
#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub const fn download_url(_url: &str, _filename: &str) -> bool {
    false
}

/// Submits a hidden form for download (native stub — returns an error).
///
/// On non-WASM targets browsers aren't available, so this always returns `Err`.
///
/// # Errors
/// Always returns an error on native targets.
#[cfg(not(target_arch = "wasm32"))]
pub async fn submit_download_form(_endpoint: &str, _fields: &[(&str, &str)]) -> Result<(), String> {
    Err("Download is only available in the browser".to_string())
}

#[cfg(test)]
mod tests {
    use super::sanitize_filename;

    #[test]
    fn sanitize_removes_path_separators() {
        assert_eq!(sanitize_filename("a/b\\c"), "a_b_c");
    }

    #[test]
    fn sanitize_strips_control_chars() {
        assert_eq!(sanitize_filename("file\x00name"), "filename");
    }

    #[test]
    fn sanitize_strips_leading_dots() {
        assert_eq!(sanitize_filename("...file.txt"), "file.txt");
    }

    #[test]
    fn sanitize_empty_input() {
        assert_eq!(sanitize_filename("   "), "");
        assert_eq!(sanitize_filename("."), "");
    }

    #[test]
    fn download_text_fails_on_native() {
        #[cfg(not(target_arch = "wasm32"))]
        assert!(crate::download_text("test", "test.txt").is_err());
    }
}

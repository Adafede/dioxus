// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Download utilities for WASM applications.
//!
//! Provides functions for triggering file downloads from browser-based apps.

#[cfg(target_arch = "wasm32")]
use js_sys::Array;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::{Blob, HtmlAnchorElement, Url};

/// Default skip link style for keyboard navigation.
pub const SKIP_LINK_STYLE: &str = "position:absolute;top:-100%;left:0.5rem;z-index:9999;padding:0.5rem 1rem;background:transparent;color:#0b5cab;font-size:0.875rem;font-weight:600;border-radius:0 0 4px 4px;text-decoration:underline;";

/// Downloads a text file in the browser using the download attribute.
///
/// # Arguments
/// - `content`: The text content to download
/// - `filename`: The base filename (without extension)
///
/// # Returns
/// - `Ok(())` on success
/// - `Err(message)` if the download fails
#[cfg(target_arch = "wasm32")]
pub fn download_text(content: &str, filename: &str) -> Result<(), String> {
    let safe_name = sanitize_filename(filename);

    let array = Array::new();
    array.push(&JsValue::from(content));

    let blob =
        Blob::new_with_str_sequence(&array).map_err(|e| format!("Failed to create blob: {e:?}"))?;

    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|e| format!("Failed to create object URL: {e:?}"))?;

    let window = web_sys::window().ok_or("No window object")?;
    let document = window.document().ok_or("No document object")?;
    let anchor: HtmlAnchorElement = document
        .create_element("a")
        .map_err(|e| format!("Failed to create anchor element: {:?}", e))?
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|e| format!("Failed to cast to HtmlAnchorElement: {:?}", e))?;

    anchor.set_href(&url);
    anchor.set_download(&safe_name);

    // Add to DOM, click, remove
    if let Some(body) = document.body() {
        let _ = body.append_child(&anchor);
    }
    anchor.click();
    if let Some(body) = document.body() {
        let _ = body.remove_child(&anchor);
    }
    let _ = Url::revoke_object_url(&url);

    Ok(())
}

/// Downloads content as a blob with a specific extension.
///
/// # Arguments
/// - `content`: The text content to download
/// - `filename`: The base filename
/// - `extension`: File extension with dot (e.g., ".csv", ".json")
#[cfg(target_arch = "wasm32")]
pub fn download_text_as_blob(content: &str, filename: &str, extension: &str) -> Result<(), String> {
    let safe_name = if extension.is_empty() {
        sanitize_filename(filename)
    } else {
        let name = sanitize_filename(filename);
        if extension.starts_with('.') && name.ends_with(extension) {
            name
        } else {
            format!("{}{}", name, extension)
        }
    };

    let array = Array::new();
    array.push(&JsValue::from(content));

    let blob =
        Blob::new_with_str_sequence(&array).map_err(|e| format!("Failed to create blob: {e:?}"))?;

    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|e| format!("Failed to create object URL: {e:?}"))?;

    let window = web_sys::window().ok_or("No window object")?;
    let document = window.document().ok_or("No document object")?;
    let anchor: HtmlAnchorElement = document
        .create_element("a")
        .map_err(|e| format!("Failed to create anchor element: {:?}", e))?
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|e| format!("Failed to cast to HtmlAnchorElement: {:?}", e))?;

    anchor.set_href(&url);
    anchor.set_download(&safe_name);
    anchor.set_attribute("style", "display:none").ok();

    if let Some(body) = document.body() {
        let _ = body.append_child(&anchor);
    }
    anchor.click();
    if let Some(body) = document.body() {
        let _ = body.remove_child(&anchor);
    }
    let _ = Url::revoke_object_url(&url);

    Ok(())
}

/// Sanitizes a filename for safe download.
///
/// Removes control characters and replaces path separators with underscores.
#[cfg(target_arch = "wasm32")]
fn sanitize_filename(input: &str) -> String {
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
    out.trim_matches('.').trim().to_string().to_string()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn download_text(_content: &str, _filename: &str) -> Result<(), String> {
    Err("Download is only available in WASM".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn download_text_as_blob(
    _content: &str,
    _filename: &str,
    _extension: &str,
) -> Result<(), String> {
    Err("Download is only available in WASM".to_string())
}

#[cfg(test)]
mod tests {
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn download_text_works() {
        // Test in wasm-bindgen-test environment
        let _ = download_text("test content", "test.txt");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn download_text_fails_on_native() {
        assert!(download_text("test", "test.txt").is_err());
    }
}

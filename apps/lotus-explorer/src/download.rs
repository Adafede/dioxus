//! Shared download helpers for browser/native targets.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

pub fn trigger_download(filename: &str, mime: &str, content_or_url: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if content_or_url.starts_with("http://") || content_or_url.starts_with("https://") {
            trigger_download_url(filename, content_or_url);
            let _ = mime;
            return;
        }

        let Some((window, document)) = window_and_document() else {
            return;
        };

        let parts = js_sys::Array::new();
        parts.push(&wasm_bindgen::JsValue::from_str(content_or_url));

        let blob = {
            let options = web_sys::BlobPropertyBag::new();
            options.set_type(mime);
            web_sys::Blob::new_with_str_sequence_and_options(&parts, &options)
                .or_else(|_| web_sys::Blob::new_with_str_sequence(&parts))
        };
        let Ok(blob) = blob else {
            return;
        };

        let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else {
            return;
        };

        if !click_download_anchor(&document, &url, filename, false) {
            let _ = window.open_with_url(&url);
        }

        let _ = web_sys::Url::revoke_object_url(&url);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (filename, mime, content_or_url);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn trigger_download_url(filename: &str, url: &str) {
    let Some((window, document)) = window_and_document() else {
        return;
    };

    if !click_download_anchor(&document, url, filename, true) {
        let _ = window.open_with_url(url);
    }
}

#[cfg(target_arch = "wasm32")]
fn window_and_document() -> Option<(web_sys::Window, web_sys::Document)> {
    let window = web_sys::window()?;
    let document = window.document()?;
    Some((window, document))
}

#[cfg(target_arch = "wasm32")]
fn click_download_anchor(
    document: &web_sys::Document,
    href: &str,
    filename: &str,
    new_tab: bool,
) -> bool {
    let Some(anchor) = document
        .create_element("a")
        .ok()
        .and_then(|el| el.dyn_into::<web_sys::HtmlAnchorElement>().ok())
    else {
        return false;
    };
    let Some(body_el) = document.body() else {
        return false;
    };

    anchor.set_href(href);
    anchor.set_download(filename);
    anchor.set_rel("noopener noreferrer");
    if new_tab {
        anchor.set_target("_blank");
    }

    let _ = body_el.append_child(&anchor);
    anchor.click();
    let _ = body_el.remove_child(&anchor);
    true
}

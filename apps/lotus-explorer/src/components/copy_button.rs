//! Reusable "copy to clipboard" button.
//!
//! Uses the modern `navigator.clipboard.writeText` API with a silent
//! `document.execCommand('copy')` fallback for older browsers / non-secure
//! contexts. Shows a brief "Copied!" state for user feedback.

use crate::i18n::{Locale, TextKey, t};
use dioxus::prelude::*;

/// A compact button that copies `text` to the system clipboard on click.
///
/// `label` is the resting label shown on the button. We swap it for
/// "Copied!" for ~1.2 seconds after a successful copy.
#[component]
pub fn CopyButton(
    text: String,
    #[props(default = "")] label: &'static str,
    #[props(default = "")] title: &'static str,
    #[props(default = "btn btn-xs copy-btn")] class: &'static str,
    #[props(default = Locale::En)] locale: Locale,
) -> Element {
    let mut copied = use_signal(|| false);
    let label_attr = if label.is_empty() {
        t(locale, TextKey::Copy)
    } else {
        label
    };
    let title_attr = if title.is_empty() {
        t(locale, TextKey::CopyToClipboard).to_string()
    } else {
        title.to_string()
    };

    rsx! {
        button {
            class: "{class}",
            r#type: "button",
            title: "{title_attr}",
            aria_label: "{title_attr}",
            onclick: move |_| {
                let t = text.clone();
                copy_to_clipboard(&t);
                *copied.write() = true;
                spawn(async move {
                    gloo_timer_sleep_ms(1200).await;
                    *copied.write() = false;
                });
            },
            if *copied.read() {
                "{t(locale, TextKey::Copied)}"
            } else {
                {label_attr}
            }
        }
    }
}

//noinspection ALL
/// Portable sleep helper — uses `setTimeout` on wasm and `thread::sleep`
async fn gloo_timer_sleep_ms(ms: u32) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use wasm_bindgen::closure::Closure;
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            let cb = Closure::once_into_js(move || {
                let _ = resolve.call0(&wasm_bindgen::JsValue::NULL);
            });
            if let Some(win) = web_sys::window() {
                let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    ms as i32,
                );
            }
        });
        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    }
}

/// Write `text` to the system clipboard. Tries `navigator.clipboard` first,
/// falls back to a hidden-textarea + `document.execCommand('copy')` for
/// older browsers or non-secure (http://) contexts where `clipboard` is
/// unavailable. Silent on failure.
pub fn copy_to_clipboard(text: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;

        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };

        // Try modern clipboard API via reflection without requiring extra
        // `web-sys` features.
        let window_js = wasm_bindgen::JsValue::from(window.clone());
        let nav = js_sys::Reflect::get(&window_js, &wasm_bindgen::JsValue::from_str("navigator"));
        if let Ok(nav) = nav {
            if let Ok(clipboard) =
                js_sys::Reflect::get(&nav, &wasm_bindgen::JsValue::from_str("clipboard"))
            {
                if let Ok(write_text) =
                    js_sys::Reflect::get(&clipboard, &wasm_bindgen::JsValue::from_str("writeText"))
                {
                    if let Some(func) = write_text.dyn_ref::<js_sys::Function>() {
                        let _ = func.call1(&clipboard, &wasm_bindgen::JsValue::from_str(text));
                        return;
                    }
                }
            }
        }

        // Fallback: hidden textarea + execCommand('copy').
        let area = document
            .create_element("textarea")
            .ok()
            .and_then(|el| el.dyn_into::<web_sys::HtmlTextAreaElement>().ok());
        if let (Some(ta), Some(body)) = (area, document.body()) {
            ta.set_value(text);
            let _ = ta.set_attribute("readonly", "");
            let _ = ta.set_attribute(
                "style",
                "position:fixed;top:0;left:0;opacity:0;pointer-events:none;",
            );
            let _ = body.append_child(&ta);
            ta.select();
            let doc_js = wasm_bindgen::JsValue::from(document.clone());
            if let Ok(exec_cmd) =
                js_sys::Reflect::get(&doc_js, &wasm_bindgen::JsValue::from_str("execCommand"))
            {
                if let Some(func) = exec_cmd.dyn_ref::<js_sys::Function>() {
                    let _ = func.call1(&doc_js, &wasm_bindgen::JsValue::from_str("copy"));
                }
            }
            let _ = body.remove_child(&ta);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = text;
    }
}

//! Reusable "copy to clipboard" button.
//!
//! Uses the modern `navigator.clipboard.writeText` API with a silent
//! `document.execCommand('copy')` fallback for older browsers / non-secure
//! contexts. Shows a brief "Copied!" state for user feedback.

use dioxus::prelude::*;

/// A compact button that copies `text` to the system clipboard on click.
///
/// `label` is the resting label shown on the button. We swap it for
/// "Copied!" for ~1.2 seconds after a successful copy.
#[component]
pub fn CopyButton(
    text: String,
    #[props(default = "Copy")] label: &'static str,
    #[props(default = "")] title: &'static str,
    #[props(default = "btn btn-xs copy-btn")] class: &'static str,
) -> Element {
    let mut copied = use_signal(|| false);
    let title_attr = if title.is_empty() {
        format!("Copy to clipboard")
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
                "Copied!"
            } else {
                {label}
            }
        }
    }
}

/// Portable sleep helper — uses `setTimeout` on wasm and `thread::sleep`
/// on native so the same `spawn(async move { … })` body works everywhere.
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
        // JSON-encode so arbitrary text (including quotes, newlines, unicode)
        // embeds safely in the eval'd snippet.
        let payload = serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string());
        let script = format!(
            r#"(() => {{
  const text = {payload};
  const nav = window.navigator;
  if (nav && nav.clipboard && nav.clipboard.writeText) {{
    nav.clipboard.writeText(text).catch(() => fallback());
    return;
  }}
  fallback();
  function fallback() {{
    try {{
      const ta = document.createElement("textarea");
      ta.value = text;
      ta.setAttribute("readonly", "");
      ta.style.position = "fixed";
      ta.style.top = "0";
      ta.style.left = "0";
      ta.style.opacity = "0";
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      ta.remove();
    }} catch (e) {{ /* give up silently */ }}
  }}
}})();"#
        );
        let _ = js_sys::eval(&script);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = text;
    }
}

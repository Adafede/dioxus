// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Programmatic document head management.
//!
//! Uses [`dioxus::document`] to set `<head>` content from Rust code.
//! All elements are deduplicated and inserted once after first render.
//!
//! # Example
//!
//! ```rust
//! use dioxus::prelude::*;
//! use ui::prelude::DocumentHead;
//!
//! fn app() -> Element {
//!     rsx! {
//!         DocumentHead {
//!             title: "My App".to_string(),
//!             description: Some("A chemistry web app".to_string()),
//!             theme_colors: Some(("#f6f8fb", "#10141b")),
//!             scripts: vec![
//!                 "https://unpkg.com/@rdkit/rdkit/dist/RDKit_minimal.js".to_string(),
//!             ],
//!         }
//!     }
//! }
//!
//! let mut dom = VirtualDom::new(app);
//! dom.rebuild_in_place();
//! ```

#![expect(
    clippy::empty_enums,
    reason = "Dioxus Props derive macro generates an empty helper enum"
)]

use dioxus::document::document;
use dioxus::prelude::*;

fn sync_document_lang(lang: &str) {
    let doc = document();
    doc.eval(format!("document.documentElement.lang = {lang:?};"));
}

/// Properties for [`DocumentHead`].
#[derive(Clone, Debug, Eq, Props, PartialEq)]
#[allow(clippy::module_name_repetitions)]
pub struct DocumentHeadProps {
    /// Page title.
    pub title: String,
    /// Document language for `<html lang="...">`.
    #[props(default = "en".to_string())]
    pub lang: String,
    /// Meta description (also sets `og:description`).
    #[props(default)]
    pub description: Option<String>,
    /// Open Graph type (default: "website").
    #[props(default)]
    pub og_type: Option<String>,
    /// Open Graph URL.
    #[props(default)]
    pub og_url: Option<String>,
    /// Light/dark theme colors for the `theme-color` meta tag.
    #[props(default)]
    pub theme_colors: Option<(&'static str, &'static str)>,
    /// External JS URLs to load via `<script async src="..." crossorigin>`.
    #[props(default)]
    pub scripts: Vec<String>,
    /// Inline JavaScript (e.g. language bootstrap, toast bridge code).
    #[props(default)]
    pub inline_script: Option<String>,
    /// `WebMCP` tool registration config — emits a read-only `<app_id>_capabilities`
    /// tool via the [WebMCP proposal](https://webmachinelearning.github.io/webmcp/).
    #[props(default)]
    pub webmcp: Option<crate::webmcp::WebMcpConfig>,
    /// Open Graph site name.
    #[props(default)]
    pub og_site_name: Option<String>,
    /// JSON-LD structured data.
    #[props(default)]
    pub json_ld: Option<String>,
    /// Canonical URL.
    #[props(default)]
    pub canonical: Option<String>,
}

/// Unified document `<head>` manager.
///
/// Call this once in your app's root component.  All head elements are added
/// lazily via `use_hook` + `document()` after the first render.
#[component]
pub fn DocumentHead(props: DocumentHeadProps) -> Element {
    let DocumentHeadProps {
        title,
        lang,
        description,
        og_type,
        og_url,
        theme_colors,
        scripts,
        inline_script,
        webmcp,
        og_site_name,
        json_ld,
        canonical,
    } = props;
    let lang = lang.trim().to_string();
    let lang_for_hook = lang.clone();
    let lang_for_effect = lang;
    let og_type = og_type.unwrap_or_else(|| "website".to_string());

    use_hook(move || {
        let doc = document();
        doc.set_title(title.clone());
        sync_document_lang(&lang_for_hook);
        let meta = |attr: &'static str, key: &str, content: String| {
            doc.create_head_element(
                "meta",
                &[(attr, key.to_string()), ("content", content)],
                None,
            );
        };

        // <meta name="description"> + og:description
        if let Some(desc) = description {
            meta("name", "description", desc.clone());
            meta("property", "og:description", desc);
        }

        // og:type
        meta("property", "og:type", og_type);

        // og:title
        meta("property", "og:title", title);

        // og:site_name
        if let Some(site_name) = og_site_name {
            meta("property", "og:site_name", site_name);
        }

        // og:url
        if let Some(url) = og_url {
            meta("property", "og:url", url);
        }

        // theme-color (light/dark)
        if let Some((light, dark)) = theme_colors {
            for (color, scheme) in [(light, "light"), (dark, "dark")] {
                doc.create_head_element(
                    "meta",
                    &[
                        ("name", "theme-color".to_string()),
                        ("content", color.to_string()),
                        ("media", format!("(prefers-color-scheme: {scheme})")),
                    ],
                    None,
                );
            }
            meta("name", "color-scheme", "light dark".to_string());
        }

        // canonical <link>
        if let Some(url) = canonical {
            doc.create_head_element(
                "link",
                &[("rel", "canonical".to_string()), ("href", url)],
                None,
            );
        }

        // External scripts.
        //
        // `async` (not `defer`): `defer` is a no-op on dynamically injected
        // scripts — browsers queue deferred scripts to run after the *parser*
        // finishes, but by the time `use_hook` runs the document is already
        // parsed, so the script never executes.  `async` loads and executes as
        // soon as the file arrives; the inline bridge JS polls for
        // `initRDKitModule` so ordering is handled gracefully.
        //
        // `crossorigin="anonymous"`: prevents `nosniff` MIME-type errors on CDN
        // resources that send CORS headers.  Browsers also deduplicate
        // duplicate `async` script `src` URLs naturally.
        for url in scripts {
            doc.create_head_element(
                "script",
                &[
                    ("src", url),
                    ("async", String::new()),
                    ("crossorigin", "anonymous".to_string()),
                ],
                None,
            );
        }

        // No app-wide stylesheet is injected here; an app that wants one supplies
        // its own (smellfish-rs passes its stylesheet through `document::Style`).
        //
        // The skip-link rule is the exception, because `ui` owns both halves of
        // it — `skip_link` and `SKIP_LINK_STYLE` — and a rule that lives with
        // the components it styles is the one an app cannot forget. It was
        // inlined in `json-count-rs`'s rsx, which is the only reason that app's
        // skip link worked and the ones in `index` and `mgf-precursor-erro-rs`
        // did not.
        doc.create_head_element(
            "style",
            &[],
            Some(format!(
                "{}\n{}",
                crate::common::SKIP_LINK_FOCUS_CSS,
                crate::common::ANALYTICS_PIXEL_CSS,
            )),
        );

        // Inline JavaScript — wrapped in an IIFE so `const`/`let` declarations
        // don't leak into the global scope (which causes
        // `SyntaxError: redeclaration` when the component re-renders or
        // hot-reloads in dev mode and the hook runs again).
        if let Some(js) = inline_script {
            let wrapped = format!("(function(){{{js}}})();");
            doc.create_head_element("script", &[], Some(wrapped));
        }

        if let Some(cfg) = webmcp {
            doc.create_head_element("script", &[], Some(crate::webmcp::capabilities_script(cfg)));
            if json_ld.is_none() {
                doc.create_head_element(
                    "script",
                    &[("type", "application/ld+json".to_string())],
                    Some(crate::webmcp::bioschemas_json_ld(cfg)),
                );
            }
        }

        // JSON-LD structured data: escape `</` so caller payloads cannot close `<script>`.
        if let Some(ld) = json_ld {
            doc.create_head_element(
                "script",
                &[("type", "application/ld+json".to_string())],
                Some(ld.replace("</", "<\\/")),
            );
        }
    });

    use_effect(move || sync_document_lang(&lang_for_effect));

    VNode::empty()
}

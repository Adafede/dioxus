// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Programmatic document `<head>` management for lotus-explore-rs.

use dioxus::prelude::*;
use ui::prelude::*;

#[cfg(target_arch = "wasm32")]
use dioxus::document::document as dioxus_document;

/// Build the absolute base URL (origin + pathname) from the browser's
/// `location` so that every canonical / hreflang link is an absolute URL.
#[cfg(target_arch = "wasm32")]
fn base_url() -> String {
    let win = web_sys::window().expect("web_sys::window");
    let loc = win.location();
    let origin = loc.origin().unwrap_or_default();
    let pathname = loc.pathname().unwrap_or_default();
    format!("{origin}{pathname}")
}

#[cfg(not(target_arch = "wasm32"))]
fn base_url() -> String {
    String::new()
}

/// Non-hreflang links that can be statically known at compile time.
const LINKS: &[LinkSpec] = &[
    LinkSpec {
        rel: "apple-touch-icon",
        href: "./apple-touch-icon.png",
        r#type: None,
        media: None,
        crossorigin: None,
        sizes: Some("180x180"),
        hreflang: None,
    },
    LinkSpec {
        rel: "icon",
        href: "./favicon-32x32.png",
        r#type: Some("image/png"),
        media: None,
        crossorigin: None,
        sizes: Some("32x32"),
        hreflang: None,
    },
    LinkSpec {
        rel: "icon",
        href: "./favicon-16x16.png",
        r#type: Some("image/png"),
        media: None,
        crossorigin: None,
        sizes: Some("16x16"),
        hreflang: None,
    },
    LinkSpec {
        rel: "dns-prefetch",
        href: "https://qlever.dev",
        r#type: None,
        media: None,
        crossorigin: None,
        sizes: None,
        hreflang: None,
    },
    LinkSpec {
        rel: "dns-prefetch",
        href: "https://query.wikidata.org",
        r#type: None,
        media: None,
        crossorigin: None,
        sizes: None,
        hreflang: None,
    },
    LinkSpec {
        rel: "dns-prefetch",
        href: "https://unpkg.com",
        r#type: None,
        media: None,
        crossorigin: None,
        sizes: None,
        hreflang: None,
    },
    LinkSpec {
        rel: "dns-prefetch",
        href: "https://www.simolecule.com",
        r#type: None,
        media: None,
        crossorigin: None,
        sizes: None,
        hreflang: None,
    },
    LinkSpec {
        rel: "dns-prefetch",
        href: "https://idsm.elixir-czech.cz",
        r#type: None,
        media: None,
        crossorigin: None,
        sizes: None,
        hreflang: None,
    },
    LinkSpec {
        rel: "manifest",
        href: "./site.webmanifest",
        r#type: Some("application/manifest+json"),
        media: None,
        crossorigin: None,
        sizes: None,
        hreflang: None,
    },
];

const DESCRIPTION: &str = "Explore LOTUS with taxon filters, SMILES/Molfile structure search, and Wikidata curation workflows.";

/// Alternating-language hreflang map: `"en"` → path suffix.
/// English has no suffix because it is the default language.
#[cfg(target_arch = "wasm32")]
const HREF_LANGS: &[(&str, &str)] = &[("en", ""), ("fr", "fr"), ("de", "de"), ("it", "it")];

#[component]
#[allow(clippy::volatile_composites)]
pub fn LotusDocumentHead(lang: String) -> Element {
    let canonical = match lang.as_str() {
        "en" => base_url(),
        other => {
            let base = base_url();
            if base.contains('?') {
                format!("{base}&lang={other}")
            } else {
                format!("{base}?lang={other}")
            }
        }
    };

    // Inject hreflang `<link rel="alternate">` tags with absolute URLs.
    #[cfg(target_arch = "wasm32")]
    use_hook(move || {
        let doc = dioxus_document();
        let base = base_url();
        for (hreflang, suffix) in HREF_LANGS {
            let href = if suffix.is_empty() {
                base.clone()
            } else if base.contains('?') {
                format!("{base}&lang={suffix}")
            } else {
                format!("{base}?lang={suffix}")
            };
            let attrs: Vec<(&str, String)> = vec![
                ("rel", "alternate".to_string()),
                ("href", href),
                ("hreflang", hreflang.to_string()),
            ];
            doc.create_head_element("link", &attrs, None);
        }
    });
    #[cfg(not(target_arch = "wasm32"))]
    let _ = canonical.clone(); // suppress unused warning in tests

    rsx! {
        DocumentHead {
            title: "LOTUS Knowledge Explorer".to_string(),
            lang,
            description: Some(DESCRIPTION.to_string()),
            og_type: Some("website".to_string()),
            og_url: Some(canonical.clone()),
            og_site_name: Some("LOTUS Knowledge Explorer".to_string()),
            theme_colors: Some(("#f6f8fb", "#10141b")),
            canonical: Some(canonical),
        }

        // Externalized CSS stylesheet via asset pipeline
        document::Link { rel: "stylesheet", href: asset!("/public/assets/lotus-explore.css") }

        // Externalized Bootstrap script asset
        document::Script { src: asset!("/public/assets/js/bootstrap.js"), defer: true }

        DocumentLinks { links: LINKS.to_vec() }
    }
}

/// Lazily inject the curation bridge JS files into the document `<head>`.
#[component]
#[allow(clippy::volatile_composites)]
pub fn CurationScripts() -> Element {
    rsx! {
        document::Script { src: asset!("/public/assets/js/curation/rdkit-bridge.js"), defer: true }
        document::Script { src: asset!("/public/assets/js/curation/citation-bridge.js"), defer: true }
    }
}

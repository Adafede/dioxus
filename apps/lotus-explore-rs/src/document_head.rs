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
#[allow(clippy::volatile_composites)]
fn links() -> Vec<LinkSpec> {
    vec![
        // Font optimization - preconnect and load Inter font
        LinkSpec {
            rel: "preconnect",
            href: "https://fonts.googleapis.com".to_string(),
            r#type: None,
            media: None,
            crossorigin: None,
            sizes: None,
            hreflang: None,
        },
        LinkSpec {
            rel: "preconnect",
            href: "https://fonts.gstatic.com".to_string(),
            r#type: None,
            media: None,
            crossorigin: Some("anonymous"),
            sizes: None,
            hreflang: None,
        },
        LinkSpec {
            rel: "stylesheet",
            href:
                "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap"
                    .to_string(),
            r#type: None,
            media: None,
            crossorigin: None,
            sizes: None,
            hreflang: None,
        },
        // Favicon links
        LinkSpec {
            rel: "icon",
            href: asset!("/public/favicon.ico").to_string(),
            r#type: Some("image/x-icon"),
            media: None,
            crossorigin: None,
            sizes: Some("48x48"),
            hreflang: None,
        },
        LinkSpec {
            rel: "apple-touch-icon",
            href: asset!("/public/apple-touch-icon.png").to_string(),
            r#type: None,
            media: None,
            crossorigin: None,
            sizes: Some("180x180"),
            hreflang: None,
        },
        LinkSpec {
            rel: "icon",
            href: asset!("/public/favicon-32x32.png").to_string(),
            r#type: Some("image/png"),
            media: None,
            crossorigin: None,
            sizes: Some("32x32"),
            hreflang: None,
        },
        LinkSpec {
            rel: "icon",
            href: asset!("/public/favicon-16x16.png").to_string(),
            r#type: Some("image/png"),
            media: None,
            crossorigin: None,
            sizes: Some("16x16"),
            hreflang: None,
        },
        // External Third-Party APIs (DNS Prefetching)
        LinkSpec {
            rel: "dns-prefetch",
            href: "https://qlever.dev".to_string(),
            r#type: None,
            media: None,
            crossorigin: None,
            sizes: None,
            hreflang: None,
        },
        LinkSpec {
            rel: "dns-prefetch",
            href: "https://query.wikidata.org".to_string(),
            r#type: None,
            media: None,
            crossorigin: None,
            sizes: None,
            hreflang: None,
        },
        LinkSpec {
            rel: "dns-prefetch",
            href: "https://unpkg.com".to_string(),
            r#type: None,
            media: None,
            crossorigin: None,
            sizes: None,
            hreflang: None,
        },
        LinkSpec {
            rel: "dns-prefetch",
            href: "https://tools-static.wmflabs.org".to_string(),
            r#type: None,
            media: None,
            crossorigin: None,
            sizes: None,
            hreflang: None,
        },
        LinkSpec {
            rel: "dns-prefetch",
            href: "https://www.simolecule.com".to_string(),
            r#type: None,
            media: None,
            crossorigin: None,
            sizes: None,
            hreflang: None,
        },
        LinkSpec {
            rel: "dns-prefetch",
            href: "https://idsm.elixir-czech.cz".to_string(),
            r#type: None,
            media: None,
            crossorigin: None,
            sizes: None,
            hreflang: None,
        },
        // Web Manifest
        LinkSpec {
            rel: "manifest",
            href: asset!("/public/site.webmanifest").to_string(),
            r#type: Some("application/manifest+json"),
            media: None,
            crossorigin: None,
            sizes: None,
            hreflang: None,
        },
    ]
}

const DESCRIPTION: &str = "Explore LOTUS with taxon filters, SMILES/Molfile structure search, and Wikidata curation workflows.";

/// Build `application/ld+json` structured data (schema.org `WebApplication`).
///
/// `serde_json::to_string` is used to safely JSON-encode the free-text
/// description and the URL so the emitted script stays valid even if those
/// values ever contain quotes or control characters.
fn json_ld(canonical: &str) -> String {
    let desc = serde_json::to_string(DESCRIPTION).unwrap_or_else(|_| "\"\"".to_string());
    let url = serde_json::to_string(canonical).unwrap_or_else(|_| "\"\"".to_string());
    format!(
        "{{\"@context\":\"https://schema.org\",\"@type\":\"WebApplication\",\"name\":\"LOTUS Knowledge Explorer\",\"description\":{desc},\"url\":{url},\"applicationCategory\":\"ScienceApplication\",\"operatingSystem\":\"Web\",\"inLanguage\":[\"en\",\"fr\",\"de\",\"it\"],\"offers\":{{\"@type\":\"Offer\",\"price\":\"0\",\"priceCurrency\":\"EUR\"}}}}"
    )
}

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
            json_ld: Some(json_ld(&canonical)),
            canonical: Some(canonical),
        }

        document::Link {
            rel: "stylesheet",
            href: asset!("/public/assets/lotus-explore.css"),
        }
        document::Script { src: asset!("/public/assets/js/bootstrap.js"), defer: true }

        DocumentLinks { links: links() }
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

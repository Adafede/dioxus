// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Programmatic document `<head>` management for lotus-explore-rs.
//!
//! Replaces the static `index.html` with Rust code that sets meta tags,
//! bundled styles, scripts (CDN + inline bridge code), and structured data.

use dioxus::prelude::*;
use ui::prelude::*;

mod inline_script;
mod inline_style;

/// JSON-LD structured data for the LOTUS Knowledge Explorer.
const JSON_LD: &str = r#"{
  "@context": "https://schema.org",
  "@type": "WebApplication",
  "name": "LOTUS Knowledge Explorer",
  "description": "Explore LOTUS natural-product records with taxon filters, SMILES/Molfile structure search, and Wikidata curation workflows.",
  "applicationCategory": "ScienceApplication",
  "operatingSystem": "Any",
  "inLanguage": "en",
  "isAccessibleForFree": true,
  "license": "https://www.gnu.org/licenses/agpl-3.0.html",
  "codeRepository": "https://github.com/Adafede/dioxus",
  "author": {
    "@type": "Organization",
    "name": "Contributors to the dioxus-apps project"
  }
}"#;

/// Link specifications for resource hints and favicons.
const LINKS: &[LinkSpec] = &[
    // Discovery links
    LinkSpec {
        rel: "manifest",
        href: "/site.webmanifest",
        r#type: Some("application/manifest+json"),
        media: None,
        crossorigin: None,
        sizes: None,
        hreflang: None,
    },
    LinkSpec {
        rel: "llmstxt",
        href: "/llms.txt",
        r#type: Some("text/plain"),
        media: None,
        crossorigin: None,
        sizes: None,
        hreflang: None,
    },
    LinkSpec {
        rel: "api-catalog",
        href: "/.well-known/api-catalog.json",
        r#type: Some("application/json"),
        media: None,
        crossorigin: None,
        sizes: Some("64x64"),
        hreflang: None,
    },
    LinkSpec {
        rel: "agent-skills",
        href: "/.well-known/agent-skills.json",
        r#type: Some("application/json"),
        media: None,
        crossorigin: None,
        sizes: None,
        hreflang: None,
    },
    // Resource hints
    LinkSpec {
        rel: "preconnect",
        href: "https://qlever.dev",
        r#type: None,
        media: None,
        crossorigin: Some("anonymous"),
        sizes: None,
        hreflang: None,
    },
    LinkSpec {
        rel: "preconnect",
        href: "https://www.simolecule.com",
        r#type: None,
        media: None,
        crossorigin: Some("anonymous"),
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
        rel: "preconnect",
        href: "https://unpkg.com",
        r#type: None,
        media: None,
        crossorigin: Some("anonymous"),
        sizes: None,
        hreflang: None,
    },
    // Favicon
    LinkSpec {
        rel: "icon",
        href: "favicon.svg",
        r#type: Some("image/svg+xml"),
        media: None,
        crossorigin: None,
        sizes: Some("any"),
        hreflang: None,
    },
];

/// Hreflang alternate links.
const HREFLANGS: &[(&str, &str)] = &[
    ("x-default", ""),
    ("en", "?lang=en"),
    ("fr", "?lang=fr"),
    ("de", "?lang=de"),
    ("it", "?lang=it"),
];

/// Base URL for the app (used in hreflang and canonical links).
const BASE_URL: &str = "https://adafede.github.io/dioxus/lotus-explore-rs/";

/// Renders the complete document `<head>` using `dioxus::document` instead of
/// a static `index.html`.  Includes SEO meta tags, OG tags, JSON-LD, CDN scripts,
/// inline bridge JS, toast CSS, and resource hint `<link>`s.
#[component]
pub fn LotusDocumentHead(lang: String) -> Element {
    let description = "Explore LOTUS natural-product records with taxon filters, SMILES/Molfile structure search, and Wikidata curation workflows.";
    let scripts = vec![
        "https://unpkg.com/@rdkit/rdkit/dist/RDKit_minimal.js".to_string(),
        "https://cdn.jsdelivr.net/npm/citation-js@0.8.2/build/citation.min.js".to_string(),
        // Privacy-respecting analytics (async, no cookies, GDPR-compliant)
        "https://scripts.simpleanalyticscdn.com/latest.js".to_string(),
    ];

    let mut links: Vec<LinkSpec> = LINKS.to_vec();
    // Canonical
    links.push(LinkSpec {
        rel: "canonical",
        href: BASE_URL,
        r#type: None,
        media: None,
        crossorigin: None,
        sizes: None,
        hreflang: None,
    });
    // Hreflang alternates — need owned strings for formatted URLs
    for (lang, suffix) in HREFLANGS {
        links.push(LinkSpec {
            rel: "alternate",
            href: Box::leak(format!("{BASE_URL}{suffix}").into_boxed_str()),
            r#type: None,
            media: None,
            crossorigin: None,
            sizes: None,
            hreflang: Some(*lang),
        });
    }
    let inline_style = format!(
        "{}\n\n{}",
        ui::styles::bundled_lotus_styles(),
        inline_style::build_inline_style()
    );

    rsx! {
        DocumentHead {
            title: "LOTUS Knowledge Explorer".to_string(),
            lang,
            description: Some(description.to_string()),
            og_type: Some("website".to_string()),
            og_url: Some(BASE_URL.to_string()),
            og_site_name: Some("LOTUS Knowledge Explorer".to_string()),
            theme_colors: Some(("#f6f8fb", "#10141b")),
            scripts,
            inline_style: Some(inline_style),
            inline_script: Some(inline_script::build_inline_script()),
            json_ld: Some(JSON_LD.to_string()),
            canonical: Some(BASE_URL.to_string()),
        }

        DocumentLinks { links }
    }
}

/// The toast notification template — placed in the component tree so it
/// renders before the main content, mirroring the original `index.html` body.
#[component]
pub fn ToastTemplate() -> Element {
    rsx! {
        div { id: "dx-toast-template", style: "display:none;visibility:hidden" }
        div {
            id: "__dx-toast",
            class: "dx-toast",
            "aria-hidden": "true",
            div {
                id: "__dx-toast-inner",
                class: "dx-toast-inner",
                style: "right:-1000px;",
                div { class: "dx-toast-level-bar-container",
                    div { id: "__dx-toast-decor", class: "dx-toast-level-bar __info" }
                }
                div { class: "dx-toast-content",
                    div { class: "dx-toast-header",
                        svg { xmlns: "http://www.w3.org/2000/svg", view_box: "0 0 32 32", "preserveAspectRatio": "none", "aria-hidden": "true",
                            path { fill: "#e96020", d: "M22.158 1.783c0 3.077-.851 5.482-2.215 7.377s-3.32 3.557-5.447 5.33-4.425 3.657-6.252 6.195-3.102 5.515-3.102 9.532h4.699c0-3.077.853-5.377 2.217-7.272s3.32-3.557 5.447-5.33 4.425-3.657 6.252-6.195 3.102-5.62 3.102-9.637z" }
                            path { fill: "#2d323b", d: "M9.531 25.927c-.635 0-1.021.515-1.02 1.15s.385 1.151 1.02 1.15H22.47a1.151 1.151 0 1 0 0-2.301zm1.361-4.076c-.608 0-.954.558-.953 1.166s.346 1.035.953 1.035h10.217a1.101 1.101 0 1 0 0-2.201zm0-13.594a1.101 1.101 0 1 0 0 2.201h10.217c.607 0 .953-.598.953-1.205s-.345-.996-.953-.996zM9.531 4.021A1.15 1.15 0 0 0 8.38 5.17a1.15 1.15 0 0 1 1.15 1.15h12.94c.635 0 1.021-.498 1.02-1.133s-.386-1.166-1.02-1.166z" }
                            path { fill: "#00a8d6", d: "M5.142 1.783c0 4.016 1.275 7.099 3.102 9.637s4.125 4.422 6.252 6.195 4.083 3.656 5.447 5.551 2.215 3.974 2.215 7.051h4.701c0-4.016-1.275-7.038-3.102-9.576s-4.125-4.422-6.252-6.195-4.083-3.435-5.447-5.33S9.841 4.86 9.841 1.783z" }
                        }
                    }
                }
                p { id: "__dx-toast-msg", class: "dx-toast-msg", "A non-hot-reloadable change occurred and we must rebuild." }
            }
        }
    }
}

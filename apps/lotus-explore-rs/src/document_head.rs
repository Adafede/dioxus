// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Programmatic document `<head>` management for lotus-explore-rs.
//!
//! Replaces the static `index.html` with Rust code that sets meta tags,
//! scripts (CDN + inline bridge code), and structured data.
//!
//! CSS is inlined directly via `DocumentHead::inline_style` using `include_str!`.
//! This ensures critical styles are delivered in the initial HTML response without
//! external stylesheet network hops or chaining critical requests.

use dioxus::prelude::*;
use ui::prelude::*;

mod inline_script;

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
        rel: "dns-prefetch",
        href: "https://unpkg.com",
        r#type: None,
        media: None,
        crossorigin: None,
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
    ("en", ""),
    ("fr", "?lang=fr"),
    ("de", "?lang=de"),
    ("it", "?lang=it"),
];

/// Base URL for the app (used in hreflang and canonical links).
const BASE_URL: &str = "https://adafede.github.io/dioxus/lotus-explore-rs/";

const CSS_STYLES: &str = concat!(include_str!("../public/assets/lotus-explore.css"),);

/// Renders the complete document `<head>` using `dioxus::document`.
#[component]
pub fn LotusDocumentHead(lang: String) -> Element {
    let description = "Explore LOTUS natural-product records with taxon filters, SMILES/Molfile structure search, and Wikidata curation workflows.";

    let mut links: Vec<LinkSpec> = LINKS.to_vec();
    for (lang_code, suffix) in HREFLANGS {
        links.push(LinkSpec {
            rel: "alternate",
            href: Box::leak(format!("{BASE_URL}{suffix}").into_boxed_str()),
            r#type: None,
            media: None,
            crossorigin: None,
            sizes: None,
            hreflang: Some(*lang_code),
        });
    }

    let canonical = match lang.as_str() {
        "en" => BASE_URL.to_string(),
        other => format!("{BASE_URL}?lang={other}"),
    };

    rsx! {
        DocumentHead {
            title: "LOTUS Knowledge Explorer".to_string(),
            lang,
            description: Some(description.to_string()),
            og_type: Some("website".to_string()),
            og_url: Some(canonical.clone()),
            og_site_name: Some("LOTUS Knowledge Explorer".to_string()),
            theme_colors: Some(("#f6f8fb", "#10141b")),
            inline_script: Some(inline_script::build_core_inline_script()),
            json_ld: Some(JSON_LD.to_string()),
            canonical: Some(canonical),
        }

        // Explicit non-blocking analytics script
        script {
            r#async: true,
            defer: true,
            src: "https://scripts.simpleanalyticscdn.com/latest.js",
        }

        style { "{CSS_STYLES}" }

        DocumentLinks { links }
    }
}

/// Lazily inject the curation bridge JS into the document `<head>`.
#[component]
pub fn CurationScripts() -> Element {
    let inline_script = Some(inline_script::build_curation_inline_script());
    rsx! {
        DocumentScripts { scripts: Vec::<String>::new(), inline_script }
    }
}

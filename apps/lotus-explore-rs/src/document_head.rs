// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Programmatic document `<head>` management for lotus-explore-rs.

use dioxus::prelude::*;
use ui::prelude::*;

mod inline_script;

/// Link specifications for resource hints and favicons.
const LINKS: &[LinkSpec] = &[
    // Resource hints for external SPARQL / API endpoints
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
    // Discovery & Manifest
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
        rel: "icon",
        href: "favicon.svg",
        r#type: Some("image/svg+xml"),
        media: None,
        crossorigin: None,
        sizes: Some("any"),
        hreflang: None,
    },
];

const HREFLANGS: &[(&str, &str)] = &[
    ("en", ""),
    ("fr", "?lang=fr"),
    ("de", "?lang=de"),
    ("it", "?lang=it"),
];

const BASE_URL: &str = "https://adafede.github.io/dioxus/lotus-explore-rs/";
const CSS_STYLES: &str = concat!(include_str!("../public/assets/lotus-explore.css"));

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
            inline_script: Some(inline_script::build_bootstrap_inline_script()),
            canonical: Some(canonical),
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

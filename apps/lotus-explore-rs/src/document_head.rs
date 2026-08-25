// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Programmatic document `<head>` management for lotus-explore-rs.

use dioxus::prelude::*;
use ui::prelude::*;

mod inline_script;

const BASE_URL: &str = "https://adafede.github.io/dioxus/lotus-explore-rs/";

const LINKS: &[LinkSpec] = &[
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
        href: "/favicon.svg",
        r#type: Some("image/svg+xml"),
        media: None,
        crossorigin: None,
        sizes: Some("any"),
        hreflang: None,
    },
    // Pre-calculated alternate hreflang links
    LinkSpec {
        rel: "alternate",
        href: "/",
        r#type: None,
        media: None,
        crossorigin: None,
        sizes: None,
        hreflang: Some("en"),
    },
    LinkSpec {
        rel: "alternate",
        href: "/?lang=fr",
        r#type: None,
        media: None,
        crossorigin: None,
        sizes: None,
        hreflang: Some("fr"),
    },
    LinkSpec {
        rel: "alternate",
        href: "/?lang=de",
        r#type: None,
        media: None,
        crossorigin: None,
        sizes: None,
        hreflang: Some("de"),
    },
    LinkSpec {
        rel: "alternate",
        href: "/?lang=it",
        r#type: None,
        media: None,
        crossorigin: None,
        sizes: None,
        hreflang: Some("it"),
    },
];

const CSS_STYLES: &str = include_str!("../public/assets/lotus-explore.css");
const DESCRIPTION: &str = "Explore LOTUS with taxon filters, SMILES/Molfile structure search, and Wikidata curation workflows.";

#[component]
pub fn LotusDocumentHead(lang: String) -> Element {
    let canonical = match lang.as_str() {
        "en" => BASE_URL.to_string(),
        other => format!("{BASE_URL}?lang={other}"),
    };

    rsx! {
        DocumentHead {
            title: "LOTUS Knowledge Explorer".to_string(),
            lang,
            description: Some(DESCRIPTION.to_string()),
            og_type: Some("website".to_string()),
            og_url: Some(canonical.clone()),
            og_site_name: Some("LOTUS Knowledge Explorer".to_string()),
            theme_colors: Some(("#f6f8fb", "#10141b")),
            inline_script: Some(inline_script::build_bootstrap_inline_script()),
            canonical: Some(canonical),
        }

        style { "{CSS_STYLES}" }
        DocumentLinks { links: LINKS.to_vec() }
    }
}

/// Lazily inject the curation bridge JS into the document `<head>`.
#[component]
pub fn CurationScripts() -> Element {
    rsx! {
        DocumentScripts {
            scripts: Vec::new(),
            inline_script: Some(inline_script::build_curation_inline_script())
        }
    }
}

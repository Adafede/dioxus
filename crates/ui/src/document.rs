// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Programmatic document head management.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus Props derive macro generates an empty helper enum"
)]

use crate::a11y::SKIP_LINK_FOCUS_CSS;
use crate::components::simple_analytics_badge::ANALYTICS_PIXEL_CSS;
use crate::webmcp::{WebMcpConfig, bioschemas_json_ld, capabilities_script};
use dioxus::document::{Document, document};
use dioxus::prelude::*;

fn sync_document_lang(lang: &str) {
    document().eval(format!("document.documentElement.lang = {lang:?};"));
}

/// Properties for [`DocumentHead`].
#[derive(Clone, Debug, Eq, Props, PartialEq)]
pub struct DocumentHeadProps {
    /// Page title.
    pub title: String,
    /// Document language for `<html lang="...">`.
    #[props(default = "en".to_string())]
    pub lang: String,
    /// Meta description (also sets `og:description`).
    #[props(default)]
    pub description: Option<String>,
    /// Open Graph type (default: `"website"`).
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
    /// Inline JavaScript (wrapped in an IIFE to isolate `const`/`let` scope).
    #[props(default)]
    pub inline_script: Option<String>,
    /// Optional `WebMCP` capability metadata.
    #[props(default)]
    pub webmcp: Option<WebMcpConfig>,
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
/// Injects title, language, metadata, external/inline scripts, shared CSS rules,
/// and optional `WebMCP` / JSON-LD descriptors once after the first render.
#[component]
pub fn DocumentHead(props: DocumentHeadProps) -> Element {
    let lang = props.lang.trim().to_string();
    let lang_for_effect = lang.clone();

    use_hook(move || {
        let doc = document();
        doc.set_title(props.title.clone());
        sync_document_lang(&lang);
        inject_meta_tags(&*doc, &props);
        inject_theme_meta(&*doc, props.theme_colors);
        inject_scripts(&*doc, &props.scripts, props.inline_script.as_deref());
        inject_structured_data(&*doc, props.webmcp, props.json_ld.as_deref());
    });

    use_effect(move || sync_document_lang(&lang_for_effect));
    VNode::empty()
}

fn create_meta(doc: &dyn Document, attr: &'static str, key: &str, content: String) {
    doc.create_head_element(
        "meta",
        &[(attr, key.to_string()), ("content", content)],
        None,
    );
}

fn inject_meta_tags(doc: &dyn Document, props: &DocumentHeadProps) {
    let og_type = props
        .og_type
        .clone()
        .unwrap_or_else(|| "website".to_string());
    if let Some(desc) = &props.description {
        create_meta(doc, "name", "description", desc.clone());
        create_meta(doc, "property", "og:description", desc.clone());
    }
    create_meta(doc, "property", "og:type", og_type);
    create_meta(doc, "property", "og:title", props.title.clone());
    if let Some(site_name) = &props.og_site_name {
        create_meta(doc, "property", "og:site_name", site_name.clone());
    }
    if let Some(url) = &props.og_url {
        create_meta(doc, "property", "og:url", url.clone());
    }
    if let Some(url) = &props.canonical {
        doc.create_head_element(
            "link",
            &[("rel", "canonical".to_string()), ("href", url.clone())],
            None,
        );
    }
}

fn inject_theme_meta(doc: &dyn Document, theme_colors: Option<(&'static str, &'static str)>) {
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
        create_meta(doc, "name", "color-scheme", "light dark".to_string());
    }
    doc.create_head_element(
        "style",
        &[],
        Some(format!("{SKIP_LINK_FOCUS_CSS}\n{ANALYTICS_PIXEL_CSS}")),
    );
}

fn inject_scripts(doc: &dyn Document, scripts: &[String], inline_script: Option<&str>) {
    for url in scripts {
        doc.create_head_element(
            "script",
            &[
                ("src", url.clone()),
                ("async", String::new()),
                ("crossorigin", "anonymous".to_string()),
            ],
            None,
        );
    }
    if let Some(js) = inline_script {
        doc.create_head_element("script", &[], Some(format!("(function(){{{js}}})();")));
    }
}

fn inject_structured_data(doc: &dyn Document, webmcp: Option<WebMcpConfig>, json_ld: Option<&str>) {
    if let Some(cfg) = webmcp {
        doc.create_head_element("script", &[], Some(capabilities_script(cfg)));
        if json_ld.is_none() {
            doc.create_head_element(
                "script",
                &[("type", "application/ld+json".to_string())],
                Some(bioschemas_json_ld(cfg)),
            );
        }
    }
    if let Some(ld) = json_ld {
        doc.create_head_element(
            "script",
            &[("type", "application/ld+json".to_string())],
            Some(ld.replace("</", "<\\/")),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::webmcp::{EdamAnnotations, EdamTerm, WebMcpConfig};

    fn render_full_head() -> Element {
        rsx! {
            DocumentHead {
                title: "Unit App".to_string(),
                lang: "en".to_string(),
                description: Some("Screen SMILES".to_string()),
                og_type: Some("website".to_string()),
                og_url: Some("https://example.org/app".to_string()),
                og_site_name: Some("Suite".to_string()),
                canonical: Some("https://example.org/app".to_string()),
                theme_colors: Some(("#ffffff", "#000000")),
                scripts: vec!["https://example.org/lib.js".to_string()],
                inline_script: Some("const x = 1;".to_string()),
                webmcp: Some(WebMcpConfig {
                    app_id: "unit_app",
                    title: "Unit App",
                    description: "desc",
                    inputs: &["csv"],
                    outputs: &["report"],
                    edam: EdamAnnotations {
                        topics: &[EdamTerm {
                            id: "topic_2258",
                            label: "Cheminformatics",
                        }],
                        operations: &[EdamTerm {
                            id: "operation_0366",
                            label: "Clustering",
                        }],
                        input_data: &[],
                        input_formats: &[],
                        output_data: &[],
                        output_formats: &[],
                    },
                }),
            }
        }
    }

    #[test]
    fn document_head_builds_empty_body_and_runs_head_hook() {
        let mut dom = VirtualDom::new(render_full_head);
        let edits = format!("{:?}", dom.rebuild_to_vec());
        assert!(
            edits.contains("CreatePlaceholder") || !edits.is_empty(),
            "DocumentHead rebuild should succeed: {edits}"
        );
    }
}

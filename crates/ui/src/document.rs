//! Programmatic document head management — replaces static `index.html` files.
//!
//! Uses [`dioxus::document`] to set `<head>` content from Rust code.
//! All elements are deduplicated and inserted once after first render.
//!
//! # Example
//!
//! ```rust, ignore
//! use dioxus::prelude::*;
//! use ui::document::DocumentHead;
//!
//! fn App() -> Element {
//!     rsx! {
//!         DocumentHead {
//!             title: "My App".to_string(),
//!             description: "A cool Dioxus app".to_string(),
//!             theme_colors: (Some("#f6f8fb"), Some("#10141b")),
//!             scripts: vec![
//!                 "https://unpkg.com/@rdkit/rdkit/dist/RDKit_minimal.js".to_string(),
//!             ],
//!         }
//!         // ... rest of app
//!     }
//! }
//! ```

use dioxus::document::document;
use dioxus::prelude::*;

/// Properties for [`DocumentHead`].
#[derive(Clone, Props, PartialEq)]
pub struct DocumentHeadProps {
    /// Page title.
    pub title: String,
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
    /// External JS URLs to load via `<script defer src="...">`.
    #[props(default)]
    pub scripts: Vec<String>,
    /// Inline CSS for a `<style>` tag in the head.
    #[props(default)]
    pub inline_style: Option<String>,
    /// Inline JavaScript (e.g. RDKit bridge code).
    #[props(default)]
    pub inline_script: Option<String>,
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
///
/// For `<link>` tags (favicons, preconnect, manifests), use
/// `ui::document::DocumentLinks` alongside this component.
#[component]
pub fn DocumentHead(props: DocumentHeadProps) -> Element {
    let title = props.title.clone();
    let description = props.description.clone();
    let og_type = props
        .og_type
        .clone()
        .unwrap_or_else(|| "website".to_string());
    let og_url = props.og_url.clone();
    let og_site_name = props.og_site_name.clone();
    let theme_colors = props.theme_colors;
    let scripts = props.scripts.clone();
    let inline_style = props.inline_style.clone();
    let inline_script = props.inline_script.clone();
    let json_ld = props.json_ld.clone();
    let canonical = props.canonical.clone();

    use_hook(move || {
        let doc = document();
        doc.set_title(title.clone());

        // <meta name="description"> + og:description
        if let Some(desc) = &description {
            doc.create_head_element(
                "meta",
                &[
                    ("name", "description".to_string()),
                    ("content", desc.clone()),
                ],
                None,
            );
            doc.create_head_element(
                "meta",
                &[
                    ("property", "og:description".to_string()),
                    ("content", desc.clone()),
                ],
                None,
            );
        }

        // og:type
        doc.create_head_element(
            "meta",
            &[("property", "og:type".to_string()), ("content", og_type)],
            None,
        );

        // og:title
        doc.create_head_element(
            "meta",
            &[
                ("property", "og:title".to_string()),
                ("content", title.clone()),
            ],
            None,
        );

        // og:site_name
        if let Some(site_name) = &og_site_name {
            doc.create_head_element(
                "meta",
                &[
                    ("property", "og:site_name".to_string()),
                    ("content", site_name.clone()),
                ],
                None,
            );
        }

        // og:url
        if let Some(url) = &og_url {
            doc.create_head_element(
                "meta",
                &[("property", "og:url".to_string()), ("content", url.clone())],
                None,
            );
        }

        // theme-color (light/dark)
        if let Some((light, dark)) = theme_colors {
            doc.create_head_element(
                "meta",
                &[
                    ("name", "theme-color".to_string()),
                    ("content", light.to_string()),
                    ("media", "(prefers-color-scheme: light)".to_string()),
                ],
                None,
            );
            doc.create_head_element(
                "meta",
                &[
                    ("name", "theme-color".to_string()),
                    ("content", dark.to_string()),
                    ("media", "(prefers-color-scheme: dark)".to_string()),
                ],
                None,
            );
            doc.create_head_element(
                "meta",
                &[
                    ("name", "color-scheme".to_string()),
                    ("content", "light dark".to_string()),
                ],
                None,
            );
        }

        // canonical <link>
        if let Some(url) = &canonical {
            doc.create_head_element(
                "link",
                &[("rel", "canonical".to_string()), ("href", url.clone())],
                None,
            );
        }

        // External scripts (async — `defer` is a no-op / harmful on dynamically
        // injected scripts: browsers add deferred scripts to a list that is only
        // flushed after the *parser* finishes, and by the time `use_hook` runs
        // the document is already parsed, so the script never executes.)
        for url in &scripts {
            doc.create_head_element(
                "script",
                &[("src", url.clone()), ("async", "".to_string())],
                None,
            );
        }

        // Inline CSS
        if let Some(css) = &inline_style {
            doc.create_head_element("style", &[], Some(css.clone()));
        }

        // Inline JavaScript
        if let Some(js) = &inline_script {
            doc.create_head_element("script", &[], Some(js.clone()));
        }

        // JSON-LD structured data
        if let Some(ld) = &json_ld {
            doc.create_head_element(
                "script",
                &[("type", "application/ld+json".to_string())],
                Some(ld.clone()),
            );
        }
    });

    VNode::empty()
}

/// Add `<link>` tags to the document head (favicons, preconnect, manifests, etc.).
#[derive(Clone, Props, PartialEq)]
pub struct DocumentLinksProps {
    #[props(default)]
    pub links: Vec<LinkSpec>,
}

/// Specification for a `<link>` tag.
#[derive(Clone, PartialEq)]
pub struct LinkSpec {
    pub rel: &'static str,
    pub href: &'static str,
    pub r#type: Option<&'static str>,
    pub media: Option<&'static str>,
    pub crossorigin: Option<&'static str>,
    pub sizes: Option<&'static str>,
    pub hreflang: Option<&'static str>,
}

#[component]
pub fn DocumentLinks(props: DocumentLinksProps) -> Element {
    let links = props.links.clone();

    use_hook(move || {
        let doc = document();
        for spec in &links {
            let mut attrs: Vec<(&str, String)> = vec![
                ("rel", spec.rel.to_string()),
                ("href", spec.href.to_string()),
            ];
            if let Some(t) = spec.r#type {
                attrs.push(("type", t.to_string()));
            }
            if let Some(m) = spec.media {
                attrs.push(("media", m.to_string()));
            }
            if let Some(c) = spec.crossorigin {
                attrs.push(("crossorigin", c.to_string()));
            }
            if let Some(s) = spec.sizes {
                attrs.push(("sizes", s.to_string()));
            }
            if let Some(h) = spec.hreflang {
                attrs.push(("hreflang", h.to_string()));
            }
            doc.create_head_element("link", &attrs, None);
        }
    });

    VNode::empty()
}

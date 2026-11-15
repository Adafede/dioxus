// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! `SimpleAnalytics` visitor badge, shown as a discreet page footer next to the analytics script.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus component macro generates a Props helper empty enum"
)]

use dioxus::prelude::*;

use crate::theme::StyleBuilder;

/// Renders the `SimpleAnalytics` badge for `domain` as a centered footer.
///
/// The badge follows the reader's color scheme: a dark variant under
/// `prefers-color-scheme: dark` and a light variant otherwise. Render it last in
/// the page so it sits at the bottom, beside the analytics script it reports to.
fn is_valid_domain(domain: &str) -> bool {
    !domain.is_empty()
        && domain.len() <= 253
        && domain.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
}

#[component]
pub fn SimpleAnalyticsBadge(domain: String) -> Element {
    let footer_style = StyleBuilder::new()
        .padding("0.5rem")
        .text_align("center")
        .build();

    if !is_valid_domain(&domain) {
        return rsx! {
            footer { style: "{footer_style}" }
        };
    }

    let href = format!("https://www.simpleanalytics.com?utm_source={domain}&utm_content=badge");
    let dark_srcset = format!(
        "https://simpleanalyticsbadges.com/{domain}?mode=dark&counter=false&radius=10&background-dark=0a0f19&text-dark=b8c5d6&logo-dark=d69999"
    );
    let light_src = format!(
        "https://simpleanalyticsbadges.com/{domain}?mode=light&counter=false&radius=10&background-light=f7fafc&text-light=4a5a6d&logo-light=990000"
    );

    rsx! {
        footer { style: "{footer_style}",
            a {
                href: "{href}",
                "referrerpolicy": "origin",
                target: "_blank",
                rel: "noopener noreferrer",
                picture {
                    source {
                        "srcset": "{dark_srcset}",
                        "media": "(prefers-color-scheme: dark)",
                    }
                    img {
                        src: "{light_src}",
                        alt: "Simple Analytics",
                        width: "118",
                        height: "28",
                        loading: "lazy",
                        "referrerpolicy": "no-referrer",
                        crossorigin: "anonymous",
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::render_debug;

    fn render_badge() -> Element {
        rsx! {
            SimpleAnalyticsBadge { domain: "tools.example.org".to_string() }
        }
    }

    #[test]
    fn simple_analytics_badge_embeds_domain_and_dark_media_query() {
        let rendered = render_debug(render_badge);
        assert!(
            rendered.contains("https://simpleanalyticsbadges.com/tools.example.org"),
            "missing badge image domain in {rendered}"
        );
        assert!(
            rendered.contains("utm_source=tools.example.org"),
            "missing utm_source query parameter in {rendered}"
        );
        let template = match SimpleAnalyticsBadge(SimpleAnalyticsBadgeProps {
            domain: "tools.example.org".to_string(),
        }) {
            Ok(node) => format!("{:?}", node.template),
            Err(err) => format!("render error: {err:?}"),
        };
        assert!(
            template.contains("prefers-color-scheme: dark"),
            "missing dark color-scheme source in {template}"
        );
    }

    #[test]
    fn rejects_invalid_and_xss_domains() {
        for bad in [
            "evil.example\" onload=\"alert(1)",
            "",
            "-bad",
            "a..b",
            "foo/bar",
        ] {
            let mut dom = VirtualDom::new_with_props(
                SimpleAnalyticsBadge,
                SimpleAnalyticsBadgeProps {
                    domain: bad.to_string(),
                },
            );
            let out = format!("{:?}", dom.rebuild_to_vec());
            assert!(
                !out.contains("onload")
                    && !out.contains("<script")
                    && !out.contains("simpleanalyticsbadges.com"),
                "unsafe domain {bad:?} leaked into badge output: {out}"
            );
        }
    }
}

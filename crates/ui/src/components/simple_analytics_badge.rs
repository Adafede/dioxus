// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! `SimpleAnalytics` visitor badge and JavaScript-free analytics pixel.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus component macro generates a Props helper empty enum"
)]

use crate::theme::StyleBuilder;
use crate::url::{is_valid_domain, percent_encode_path};
use dioxus::prelude::*;

/// Positions the JavaScript-free Simple Analytics pixel out of flow.
pub(crate) const ANALYTICS_PIXEL_CSS: &str = ".analytics-pixel { position: absolute; inset-block-start: 0; inset-inline-start: 0; inline-size: 1px; block-size: 1px; opacity: 0; pointer-events: none; }";

/// Renders the `SimpleAnalytics` badge for `domain` and the JavaScript-free
/// `noscript.gif` pixel as a centered footer.
#[component]
pub fn SimpleAnalyticsBadge(
    /// Public domain reported to Simple Analytics.
    domain: String,
    /// Optional URL path appended to the no-JS pixel URL.
    #[props(default)]
    path: Option<String>,
) -> Element {
    let footer_style = StyleBuilder::with_capacity(48)
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
    let pixel_src = path.as_deref().filter(|p| !p.is_empty()).map_or_else(
        || format!("https://queue.simpleanalyticscdn.com/noscript.gif?hostname={domain}"),
        |p| {
            format!(
                "https://queue.simpleanalyticscdn.com/noscript.gif?hostname={domain}&path={}",
                percent_encode_path(p)
            )
        },
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
        img {
            src: "{pixel_src}",
            alt: "",
            width: "1",
            height: "1",
            "referrerpolicy": "no-referrer-when-downgrade",
            "decoding": "async",
            "fetchpriority": "low",
            "aria-hidden": "true",
            class: "analytics-pixel",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::render_debug;

    fn render_badge() -> Element {
        rsx! {
            SimpleAnalyticsBadge {
                domain: "tools.example.org".to_string(),
                path: "/dioxus/".to_string(),
            }
        }
    }

    #[test]
    fn simple_analytics_badge_embeds_domain_dark_media_and_nojs_pixel() {
        let rendered = render_debug(render_badge);
        assert!(
            rendered.contains("https://simpleanalyticsbadges.com/tools.example.org"),
            "missing badge image domain in {rendered}"
        );
        assert!(
            rendered.contains(
                "https://queue.simpleanalyticscdn.com/noscript.gif?hostname=tools.example.org&path=%2Fdioxus%2F"
            ),
            "missing no-JS analytics pixel src in {rendered}"
        );
        let template = match SimpleAnalyticsBadge(SimpleAnalyticsBadgeProps {
            domain: "tools.example.org".to_string(),
            path: Some("/dioxus/".to_string()),
        }) {
            Ok(node) => format!("{:?}", node.template),
            Err(err) => format!("render error: {err:?}"),
        };
        assert!(
            template.contains("analytics-pixel") && template.contains("prefers-color-scheme: dark")
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
                    path: Some("/".to_string()),
                },
            );
            let out = format!("{:?}", dom.rebuild_to_vec());
            assert!(
                !out.contains("onload") && !out.contains("noscript.gif"),
                "unsafe domain {bad:?} leaked: {out}"
            );
        }
    }
}

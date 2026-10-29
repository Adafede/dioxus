// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! SimpleAnalytics visitor badge, shown as a discreet page footer next to the analytics script.

use dioxus::prelude::*;

use crate::theme::StyleBuilder;

/// Renders the SimpleAnalytics badge for `domain` as a centered footer.
///
/// The badge follows the reader's color scheme: a dark variant under
/// `prefers-color-scheme: dark` and a light variant otherwise. Render it last in
/// the page so it sits at the bottom, beside the analytics script it reports to.
#[component]
pub fn SimpleAnalyticsBadge(domain: String) -> Element {
    let html = format!(
        "<a href=\"https://www.simpleanalytics.com?utm_source={domain}&utm_content=badge\" referrerpolicy=\"origin\" target=\"_blank\" rel=\"noopener noreferrer\">\
<picture>\
<source srcset=\"https://simpleanalyticsbadges.com/{domain}?mode=dark&counter=false&radius=10&background-dark=0a0f19&text-dark=b8c5d6&logo-dark=d69999\" media=\"(prefers-color-scheme: dark)\" />\
<img src=\"https://simpleanalyticsbadges.com/{domain}?mode=light&counter=false&radius=10&background-light=f7fafc&text-light=4a5a6d&logo-light=990000\" alt=\"Simple Analytics\" width=\"118\" height=\"28\" loading=\"lazy\" referrerpolicy=\"no-referrer\" crossorigin=\"anonymous\" />\
</picture>\
</a>"
    );

    let footer_style = StyleBuilder::new()
        .padding("0.5rem")
        .text_align("center")
        .build();

    rsx! {
        footer { style: "{footer_style}", dangerous_inner_html: "{html}" }
    }
}

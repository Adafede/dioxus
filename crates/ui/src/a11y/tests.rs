// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use super::{SKIP_LINK_FOCUS_CSS, SKIP_LINK_STYLE, skip_link};
use dioxus::prelude::*;

#[test]
fn the_focus_rule_overrides_the_offset_the_inline_style_parks_it_with() {
    assert!(
        SKIP_LINK_STYLE.contains("top:-100%"),
        "SKIP_LINK_STYLE no longer parks the link off-screen"
    );
    assert!(
        SKIP_LINK_FOCUS_CSS.contains("top: 0 !important"),
        "SKIP_LINK_FOCUS_CSS must override inline top with !important"
    );
}

#[test]
fn the_inline_style_keeps_the_link_in_the_tab_order() {
    for hiding in ["display:none", "display: none", "visibility:hidden"] {
        assert!(
            !SKIP_LINK_STYLE.contains(hiding),
            "SKIP_LINK_STYLE contains `{hiding}`, removing the link from tab order"
        );
    }
}

#[test]
fn skip_link_renders_landmark_target_and_class() {
    let rendered = match skip_link() {
        Ok(node) => format!("{:?}", node.template),
        Err(err) => format!("render error: {err:?}"),
    };
    assert!(
        rendered.contains("#main-content") && rendered.contains("skip-link"),
        "skip_link template missing landmark href or class in {rendered}"
    );
    let mut dom = VirtualDom::new(|| rsx! { skip_link {} });
    let mutations = format!("{:?}", dom.rebuild_to_vec());
    assert!(
        mutations.contains("top:-100%"),
        "skip_link rebuild missing inline parking style in {mutations}"
    );
}

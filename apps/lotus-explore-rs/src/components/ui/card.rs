// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Card component for grouped content.

use crate::ui::classes;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    #[props(default)]
    pub children: Element,
    #[props(default)]
    pub href: Option<String>,
    #[props(default)]
    pub class: &'static str,
}

#[component]
pub fn Card(props: CardProps) -> Element {
    let base_class = format!("{} p-4 lg:p-6 {}", classes::CARD, props.class);

    if let Some(href) = &props.href {
        rsx! {
            article {
                class: "{base_class}",
                a {
                    href: href,
                    class: "block",
                    {props.children}
                }
            }
        }
    } else {
        rsx! {
            article {
                class: "{base_class}",
                {props.children}
            }
        }
    }
}

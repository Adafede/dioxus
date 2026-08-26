// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Badge components using Wikidata organism colors.

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BadgeVariant {
    Compound,
    Taxon,
    Reference,
    Entries,
    Warning,
}

impl BadgeVariant {
    pub const fn classes(&self) -> &'static str {
        match self {
            Self::Compound => {
                "border-wd-compound/35 bg-stat-compound text-wd-compound"
            }
            Self::Taxon => "border-wd-taxon/35 bg-stat-taxon text-wd-taxon",
            Self::Reference => {
                "border-wd-reference/35 bg-stat-reference text-wd-reference"
            }
            Self::Entries => "border-wd-entries/35 bg-stat-total text-wd-entries",
            Self::Warning => "border-warning/35 bg-warning/10 text-warning",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct StatusBadgeProps {
    pub label: String,
    pub variant: BadgeVariant,
    #[props(default)]
    pub count: Option<usize>,
    #[props(default)]
    pub title: Option<String>,
}

#[component]
pub fn StatusBadge(props: StatusBadgeProps) -> Element {
    let tone = props.variant.classes();

    rsx! {
        span {
            class: "inline-flex items-center gap-1.5 rounded-lotus-sm border px-2 py-0.5 text-micro font-medium shadow-xs {tone}",
            title: props.title.as_deref().unwrap_or(""),
            "{props.label}"
            if let Some(count) = props.count {
                span {
                    class: "ml-1 rounded-full bg-text/10 px-1.5 py-0.5 text-[10px]",
                    "{count}"
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct IdBadgeProps {
    pub qid: &'static str,
    #[props(default)]
    pub class: &'static str,
}

#[component]
pub fn IdBadge(props: IdBadgeProps) -> Element {
    rsx! {
        a {
            href: "https://www.wikidata.org/wiki/{props.qid}",
            target: "_blank",
            rel: "noopener noreferrer",
            class: "font-mono text-ui text-accent hover:underline {props.class}",
            "{props.qid}"
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct IdBadgeTypeProps {
    pub children: Element,
    #[props(default)]
    pub class: &'static str,
}

#[component]
pub fn IdBadgeType(props: IdBadgeTypeProps) -> Element {
    rsx! {
        span {
            class: "text-micro text-subtle {props.class}",
            {props.children}
        }
    }
}

// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! SVG validation for molecule depictions injected into the DOM.

use std::borrow::Cow;

const FORBIDDEN_MARKERS: [&str; 11] = [
    "<script",
    "<foreignobject",
    "<iframe",
    "<object",
    "<embed",
    "<animate",
    "<set",
    "<handler",
    "javascript:",
    "vbscript:",
    "data:text/html",
];

/// Returns `true` when `svg` is a plain `<svg>` document free of active scripts or inline handlers.
#[must_use]
pub(crate) fn is_safe_svg(svg: &str) -> bool {
    let lower = svg.to_ascii_lowercase();
    let trimmed = lower.trim();
    let body = trimmed
        .strip_prefix("<?xml")
        .and_then(|rest| rest.split_once("?>").map(|(_, after)| after.trim_start()))
        .unwrap_or(trimmed);
    let Some(after_svg) = body.strip_prefix("<svg") else {
        return false;
    };
    if !after_svg.starts_with(|c: char| c.is_ascii_whitespace() || c == '>')
        || !trimmed.ends_with("</svg>")
    {
        return false;
    }
    let normalized: Cow<'_, str> = if trimmed.chars().any(char::is_control) {
        Cow::Owned(trimmed.chars().filter(|c| !c.is_control()).collect())
    } else {
        Cow::Borrowed(trimmed)
    };
    if FORBIDDEN_MARKERS
        .iter()
        .any(|needle| normalized.contains(needle))
    {
        return false;
    }
    !has_inline_event_handler(body)
}

/// Filters an optional raw SVG string, dropping unsafe or malformed markup.
#[must_use]
pub(crate) fn sanitize_svg(svg: Option<String>) -> Option<String> {
    svg.filter(|markup| is_safe_svg(markup))
}

fn has_inline_event_handler(svg_lower: &str) -> bool {
    for segment in svg_lower.split('<').skip(1) {
        let Some((tag, _)) = segment.split_once('>') else {
            return true;
        };
        for (eq_idx, _) in tag.match_indices('=') {
            let Some(before_eq) = tag.get(..eq_idx) else {
                continue;
            };
            let attr = before_eq
                .trim_end()
                .rsplit(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
                .next()
                .unwrap_or("");
            if let Some(rest) = attr.strip_prefix("on")
                && !rest.is_empty()
                && rest.bytes().all(|b| b.is_ascii_alphabetic())
            {
                return true;
            }
        }
    }
    false
}

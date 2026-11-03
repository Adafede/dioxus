// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Formatting helpers for display values.
//!
//! Each function is a small, pure transformation — no DOM, no signals.

use crate::model::{contains_ignore_ascii_case, matches_any};

/// Format an NP-likeness score with a `+` or `−` sign and two decimals.
#[must_use]
pub(super) fn format_score(score: f64) -> String {
    format!("{score:+.2}")
}

/// CSS-safe verdict color class based on the verdict text.
///
/// Order mirrors [`super::csv_export::category`]: `"synthetic-leaning"` and
/// `"citation needed"` are checked before `"lotus"` so a LOTUS-scaffold hint
/// that still requires a citation renders as skeptical rather than green.
#[must_use]
pub(super) fn verdict_color(verdict: &str) -> &'static str {
    if matches_any(verdict, &["smells fishy", "highly synthetic"]) {
        "verdict-fishy"
    } else if contains_ignore_ascii_case(verdict, "synthetic-leaning") {
        "verdict-caution"
    } else if contains_ignore_ascii_case(verdict, "citation needed") {
        "verdict-skeptical"
    } else if matches_any(verdict, &["likely", "strong natural", "lotus"]) {
        "verdict-likely"
    } else if contains_ignore_ascii_case(verdict, "weak np signals")
        || (contains_ignore_ascii_case(verdict, "ertl")
            && (verdict.contains("-1") || verdict.contains("\u{2212}1")))
    {
        "verdict-caution"
    } else {
        "verdict-neutral"
    }
}

/// Emoji prefix for the scaffold family — 🌿 for NP-typical scaffolds,
/// ⚠ for polyaromatic (synthetic-typical).
#[must_use]
pub(super) fn scaffold_emoji(family: &str) -> &'static str {
    if contains_ignore_ascii_case(family, "polyaromatic") {
        "\u{26a0}"
    } else if matches_any(
        family,
        &[
            "polycyclic",
            "steroid",
            "sugar",
            "macrolide",
            "flavonoid",
            "heteroaromatic",
        ],
    ) {
        "\u{1f33f}"
    } else {
        "\u{2014}"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_score_shows_sign() {
        assert_eq!(format_score(1.5), "+1.50");
        assert_eq!(format_score(-0.5), "-0.50");
        assert_eq!(format_score(0.0), "+0.00");
    }

    #[test]
    fn verdict_color_classifies_text() {
        assert_eq!(verdict_color("Smells fishy"), "verdict-fishy");
        assert_eq!(verdict_color("highly synthetic"), "verdict-fishy");
        assert_eq!(verdict_color("Likely novel NP"), "verdict-likely");
        assert_eq!(verdict_color("Citation needed"), "verdict-skeptical");
        assert_eq!(
            verdict_color("👃 Citation needed — LOTUS scaffold hint, Ertl-only (Ertl +1.20)"),
            "verdict-skeptical"
        );
        assert_eq!(
            verdict_color("🟧 Synthetic-leaning structure (Ertl +1.50)"),
            "verdict-caution"
        );
        assert_eq!(verdict_color("Weak NP signals"), "verdict-caution");
        assert_eq!(verdict_color("Everything else"), "verdict-neutral");
    }

    #[test]
    fn scaffold_emoji_distinguishes_families() {
        assert_eq!(scaffold_emoji("polyaromatic"), "\u{26a0}");
        assert_eq!(scaffold_emoji("steroid"), "\u{1f33f}");
        assert_eq!(scaffold_emoji("unknown family"), "\u{2014}");
    }
}

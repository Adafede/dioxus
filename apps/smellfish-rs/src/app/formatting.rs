// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Formatting helpers for display values.
//!
//! Each function is a small, pure transformation — no DOM, no signals.

/// Format an NP-likeness score with a `+` or `−` sign and two decimals.
#[must_use]
pub fn format_score(score: f64) -> String {
    format!("{score:+.2}")
}

/// CSS-safe verdict color class based on the verdict text.
#[must_use]
pub fn verdict_color(verdict: &str) -> &'static str {
    let l = verdict.to_ascii_lowercase();
    if l.contains("smells fishy") || l.contains("highly synthetic") {
        "verdict-fishy"
    } else if l.contains("likely") || l.contains("strong natural") || l.contains("lotus") {
        "verdict-likely"
    } else if l.contains("citation needed") {
        "verdict-skeptical"
    } else if l.contains("weak np signals") || l.contains("ertl") && l.contains("\u{2212}1") {
        "verdict-caution"
    } else {
        "verdict-neutral"
    }
}

/// Emoji prefix for the scaffold family — 🌿 for NP-typical scaffolds,
/// ⚠ for polyaromatic (synthetic-typical).
#[must_use]
pub fn scaffold_emoji(family: &str) -> &'static str {
    let l = family.to_ascii_lowercase();
    if l.contains("polyaromatic") {
        "\u{26a0}"
    } else if l.contains("polycyclic")
        || l.contains("steroid")
        || l.contains("sugar")
        || l.contains("macrolide")
        || l.contains("flavonoid")
        || l.contains("heteroaromatic")
    {
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

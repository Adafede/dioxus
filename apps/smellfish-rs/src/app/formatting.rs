// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Score formatting helper for the web UI.

/// Format an NP-likeness score with explicit sign and two decimal places.
#[must_use]
pub(super) fn format_score(score: f64) -> String {
    format!("{score:+.2}")
}

#[cfg(test)]
mod tests {
    use super::format_score;

    #[test]
    fn format_score_shows_sign() {
        assert_eq!(format_score(1.5), "+1.50");
        assert_eq!(format_score(-0.5), "-0.50");
        assert_eq!(format_score(0.0), "+0.00");
    }
}

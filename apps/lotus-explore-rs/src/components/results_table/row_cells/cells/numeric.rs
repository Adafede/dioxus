// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Numeric and simple value cells for results-table rows: mass, formula, year.

use dioxus::prelude::*;

/// Formats a mass value to 4 decimal places for display.
///
/// Extracted as a pure function to be unit-testable independently of RSX.
#[must_use]
pub(in crate::components::results_table) fn format_mass_value(mass: f64) -> String {
    format!("{mass:.4}")
}

pub(in crate::components::results_table::row_cells) fn mass_cell(mass: Option<f64>) -> Element {
    rsx! {
        td { class: "td-num",
            if let Some(m) = mass {
                span { "{format_mass_value(m)}" }
            } else {
                span { class: "na", "-" }
            }
        }
    }
}

pub(in crate::components::results_table::row_cells) fn formula_cell(
    formula: Option<&str>,
) -> Element {
    rsx! {
        td { class: "td-formula",
            if let Some(f) = formula {
                span { class: "formula", "{f}" }
            } else {
                span { class: "na", "-" }
            }
        }
    }
}

pub(in crate::components::results_table::row_cells) fn year_cell(pub_year: Option<i16>) -> Element {
    rsx! {
        td { class: "td-year",
            if let Some(y) = pub_year {
                span { "{y}" }
            } else {
                span { class: "na", "-" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_mass_value_uses_four_decimal_places() {
        assert_eq!(format_mass_value(1.0), "1.0000");
        assert_eq!(format_mass_value(180.15588), "180.1559");
        assert_eq!(format_mass_value(0.0), "0.0000");
    }

    #[test]
    fn format_mass_value_rounds_half_up_on_fifth_decimal() {
        // 1.000045 rounded to 4dp → "1.0000" (rounds down)
        assert_eq!(format_mass_value(1.000045), "1.0000");
        // 1.000055 rounded to 4dp → "1.0001" (rounds up)
        assert_eq!(format_mass_value(1.000055), "1.0001");
    }

    #[test]
    fn format_mass_value_handles_large_masses() {
        assert_eq!(format_mass_value(1234.5678), "1234.5678");
        assert_eq!(format_mass_value(10_000.0), "10000.0000");
    }
}

// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Curation UI styles: specific to data curation workflows in lotus.
//! Shared re-export module for backward compatibility.

#[cfg(test)]
mod tests {
    use ui::styles::button_primary_sm_style;

    #[test]
    fn curation_button_primary_sm_available() {
        let style = button_primary_sm_style();
        assert!(!style.is_empty());
    }
}

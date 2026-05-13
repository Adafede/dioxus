// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Hook to access locale throughout the component tree without prop drilling.

use crate::i18n::Locale;
use dioxus::prelude::*;

/// Read the locale from context.
/// This must be called within a component tree wrapped by `LocaleProvider`.
#[must_use]
pub fn use_locale() -> Locale {
    let signal = use_context::<Signal<Locale>>();
    *signal.read()
}

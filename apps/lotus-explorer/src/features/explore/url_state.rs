// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

#[cfg(target_arch = "wasm32")]
use super::url_codec::build_query_string;
use crate::app::view::AppView;
use crate::i18n::Locale;
use std::collections::BTreeMap;

pub use super::url_codec::{
    InitialUrlState, build_shareable_url, parse_criteria_from_params,
    parse_startup_action_from_params,
};

#[cfg(test)]
pub use super::url_codec::InitialDownloadState;

pub fn initial_url_state() -> InitialUrlState {
    let params = read_url_query_params();
    InitialUrlState {
        criteria: parse_criteria_from_params(&params),
        view: AppView::from_query_value(params.get("view").map(String::as_str)),
        locale: Locale::detect(params.get("lang").map(String::as_str).unwrap_or("")),
        download: parse_startup_action_from_params(&params),
    }
}

pub fn absolute_share_url(share: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(win) = web_sys::window() {
            let loc = win.location();
            if let (Ok(origin), Ok(pathname)) = (loc.origin(), loc.pathname()) {
                return format!("{origin}{pathname}{share}");
            }
        }
    }
    share.to_string()
}

pub fn absolute_current_url_with_query(query: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(win) = web_sys::window() {
            let loc = win.location();
            if let (Ok(origin), Ok(pathname)) = (loc.origin(), loc.pathname()) {
                return format!("{origin}{pathname}?{query}");
            }
        }
    }
    format!("?{query}")
}

pub fn persist_locale_query_param(locale: Locale) {
    #[cfg(target_arch = "wasm32")]
    {
        let mut params = read_url_query_params();
        params.insert(
            "lang".to_string(),
            match locale {
                Locale::En => "en",
                Locale::Fr => "fr",
                Locale::De => "de",
                Locale::It => "it",
            }
            .to_string(),
        );
        let query = build_query_string(&params);
        let url = absolute_current_url_with_query(&query);
        if let Some(win) = web_sys::window()
            && let Ok(history) = win.history()
        {
            let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = locale;
    }
}

pub fn persist_view_query_param(view: AppView) {
    #[cfg(target_arch = "wasm32")]
    {
        let mut params = read_url_query_params();
        if let Some(view_param) = view.query_value() {
            params.insert("view".to_string(), view_param.to_string());
        } else {
            params.remove("view");
        }
        let query = build_query_string(&params);
        let url = if query.is_empty() {
            absolute_current_url_with_query("")
                .trim_end_matches('?')
                .to_string()
        } else {
            absolute_current_url_with_query(&query)
        };
        if let Some(win) = web_sys::window()
            && let Ok(history) = win.history()
        {
            let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = view.query_value();
    }
}

pub fn read_url_query_params() -> BTreeMap<String, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let mut out = BTreeMap::new();
        let Some(window) = web_sys::window() else {
            return out;
        };
        let Ok(search) = window.location().search() else {
            return out;
        };
        let query = search.trim_start_matches('?');
        for pair in query.split('&') {
            if pair.is_empty() {
                continue;
            }
            let mut parts = pair.splitn(2, '=');
            let key = parts.next().unwrap_or_default();
            let val = parts.next().unwrap_or_default();
            let key_decoded = urlencoding::decode(key)
                .map(|v| v.into_owned())
                .unwrap_or_else(|_| key.to_string());
            let val_decoded = urlencoding::decode(val)
                .map(|v| v.into_owned())
                .unwrap_or_else(|_| val.to_string());
            out.insert(key_decoded, val_decoded);
        }
        out
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        BTreeMap::new()
    }
}

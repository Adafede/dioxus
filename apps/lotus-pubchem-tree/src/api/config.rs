// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

pub fn api_base_url() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(base) = runtime_query_param("api_base") {
            if let Some(normalized) = normalize_api_base(&base) {
                return Some(normalized);
            }
        }
    }

    if let Some(base) = option_env!("LOTUS_API_BASE")
        && let Some(normalized) = normalize_api_base(base)
    {
        return Some(normalized);
    }

    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window()
            && let Ok(hostname) = window.location().hostname()
        {
            let hostname = hostname.to_ascii_lowercase();
            if hostname == "localhost" || hostname == "127.0.0.1" {
                return Some(String::new());
            }
        }
    }

    None
}

pub fn resolve_api_url(base: &str, url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return trimmed.to_string();
    }
    let base = base.trim_end_matches('/');
    if base.is_empty() {
        if trimmed.starts_with('/') {
            trimmed.to_string()
        } else {
            format!("/{trimmed}")
        }
    } else if trimmed.starts_with('/') {
        format!("{base}{trimmed}")
    } else {
        format!("{base}/{trimmed}")
    }
}

fn normalize_api_base(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return None;
    }
    Some(trimmed.to_string())
}

#[cfg(target_arch = "wasm32")]
fn runtime_query_param(name: &str) -> Option<String> {
    let window = web_sys::window()?;
    let search = window.location().search().ok()?;
    let query = search.trim_start_matches('?');
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or_default();
        let value = parts.next().unwrap_or_default();
        let decoded_key = urlencoding::decode(key).ok()?;
        if decoded_key == name {
            return urlencoding::decode(value).ok().map(|v| v.into_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_base_trims_trailing_slash() {
        assert_eq!(
            normalize_api_base("https://api.example.org/"),
            Some("https://api.example.org".to_string())
        );
    }

    #[test]
    fn resolve_api_url_handles_relative_and_absolute_urls() {
        assert_eq!(
            resolve_api_url("https://api.example.org", "/v1/pubchem-tree/fetch"),
            "https://api.example.org/v1/pubchem-tree/fetch"
        );
        assert_eq!(
            resolve_api_url("", "/v1/pubchem-tree/fetch"),
            "/v1/pubchem-tree/fetch"
        );
    }
}

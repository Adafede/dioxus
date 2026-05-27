// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::api::ApiClientError;

pub fn present_api_error(error: &ApiClientError) -> String {
    match error {
        ApiClientError::NotConfigured => error.to_string(),
        ApiClientError::Network(message)
        | ApiClientError::Parse(message)
        | ApiClientError::Http(_, message) => {
            if let Some(detail) = try_extract_error_field(message) {
                detail
            } else {
                error.to_string()
            }
        }
    }
}

fn try_extract_error_field(body: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()?
        .get("error")?
        .as_str()
        .map(ToString::to_string)
}

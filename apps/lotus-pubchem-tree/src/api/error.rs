// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiClientError {
    NotConfigured,

    Network(String),

    Http(u16, String),

    Parse(String),
}

impl std::fmt::Display for ApiClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotConfigured => write!(
                f,
                "PubChem tree API is not configured. Start lotus-api and open the app with ?api_base=http://127.0.0.1:8787 if needed."
            ),
            Self::Network(message) => write!(f, "network error: {message}"),
            Self::Http(status, body) => write!(f, "HTTP {status}: {body}"),
            Self::Parse(message) => write!(f, "parse error: {message}"),
        }
    }
}

impl std::error::Error for ApiClientError {}

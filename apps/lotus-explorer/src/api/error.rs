// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use std::fmt;

#[derive(Debug)]
pub enum ApiClientError {
    NotConfigured,
    Network(String),
    Http(u16, String),
    Parse(String),
}

impl fmt::Display for ApiClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotConfigured => write!(f, "LOTUS API not configured"),
            Self::Network(e) => write!(f, "Network error: {e}"),
            Self::Http(code, body) => write!(f, "HTTP {code}: {body}"),
            Self::Parse(e) => write!(f, "Parse error: {e}"),
        }
    }
}

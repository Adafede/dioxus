// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiClientError {
    #[error("LOTUS API not configured")]
    #[expect(dead_code)]
    NotConfigured,

    #[error("network error: {0}")]
    Network(String),

    #[error("HTTP {0}: {1}")]
    Http(u16, String),

    #[error("parse error: {0}")]
    Parse(String),
}

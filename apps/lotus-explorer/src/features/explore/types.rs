// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryPhase {
    Idle,
    ResolvingTaxon,
    Counting,
    FetchingPreview,
    Rendering,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ErrorKind {
    Validation,
    Network,
    Parse,
    #[cfg(target_arch = "wasm32")]
    Memory,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct AppError {
    pub kind: ErrorKind,
    pub message: String,
}

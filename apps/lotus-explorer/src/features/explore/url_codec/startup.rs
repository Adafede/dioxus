use super::{QueryParams, is_true_flag};
use crate::download::DownloadFormat;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InitialDownloadState {
    pub pending_format: Option<DownloadFormat>,
    pub pending_invalid_format: Option<String>,
    pub direct_execute: bool,
}

pub fn parse_startup_action_from_params(params: &QueryParams) -> InitialDownloadState {
    if !params.get("download").is_some_and(|v| is_true_flag(v)) {
        return InitialDownloadState {
            direct_execute: params.get("execute").is_some_and(|v| is_true_flag(v)),
            ..InitialDownloadState::default()
        };
    }

    let requested_format = requested_download_format(params);
    let pending_format = DownloadFormat::from_str(&requested_format);

    InitialDownloadState {
        pending_format,
        pending_invalid_format: pending_format.is_none().then_some(requested_format),
        direct_execute: false,
    }
}

fn requested_download_format(params: &QueryParams) -> String {
    params
        .get("format")
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_else(|| "csv".into())
}

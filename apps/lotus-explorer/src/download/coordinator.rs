// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::download::DownloadFormat;
use crate::queries;
use std::borrow::Cow;

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(super) struct QueryExportPlan<'a> {
    pub action: &'static str,
    pub query: Cow<'a, str>,
}

pub(super) fn query_export_plan<'a>(format: DownloadFormat, query: &'a str) -> QueryExportPlan<'a> {
    match format {
        DownloadFormat::Csv => QueryExportPlan {
            action: "csv_export",
            query: Cow::Borrowed(query),
        },
        DownloadFormat::Json => QueryExportPlan {
            action: "qlever_json_export",
            query: Cow::Borrowed(query),
        },
        DownloadFormat::Rdf => QueryExportPlan {
            action: "turtle_export",
            query: Cow::Owned(queries::query_construct_from_select(query)),
        },
    }
}

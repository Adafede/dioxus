// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::download::DownloadFormat;
use crate::queries;
use std::borrow::Cow;

pub(super) struct QueryExportPlan<'a> {
    #[cfg(target_arch = "wasm32")]
    pub action: &'static str,
    pub query: Cow<'a, str>,
}

pub(super) fn query_export_plan<'a>(format: DownloadFormat, query: &'a str) -> QueryExportPlan<'a> {
    match format {
        DownloadFormat::Csv => QueryExportPlan {
            #[cfg(target_arch = "wasm32")]
            action: "csv_export",
            query: Cow::Borrowed(query),
        },
        DownloadFormat::Json => QueryExportPlan {
            #[cfg(target_arch = "wasm32")]
            action: "qlever_json_export",
            query: Cow::Borrowed(query),
        },
        DownloadFormat::Rdf => QueryExportPlan {
            #[cfg(target_arch = "wasm32")]
            action: "turtle_export",
            query: Cow::Owned(queries::query_construct_from_select(query)),
        },
    }
}

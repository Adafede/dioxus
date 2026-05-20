// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::{
    errors::ApiError,
    types::{RowDto, SearchResponse, SearchStats},
};
use shared::lotus::{models::DatasetStats, queries, sparql};

pub(crate) async fn build_search_response(
    execution_query: &str,
    limit: usize,
    include_counts: bool,
    resolved_taxon_qid: Option<String>,
    warning: Option<String>,
) -> Result<SearchResponse, ApiError> {
    let display_query = queries::query_with_limit(execution_query, limit);
    let count_query = queries::query_counts_from_base(execution_query);

    let rows = if include_counts {
        let rows_future = async {
            let rows_bytes = sparql::execute_sparql_bytes(&display_query)
                .await
                .map_err(|e| ApiError::upstream(format!("display query failed: {e}")))?;
            sparql::parse_compounds_csv_display_bytes(&rows_bytes, limit)
                .map_err(|e| ApiError::upstream(format!("display parse failed: {e}")))
        };
        let stats_future = async {
            let count_bytes = sparql::execute_sparql_bytes(&count_query)
                .await
                .map_err(|e| ApiError::upstream(format!("count query failed: {e}")))?;
            sparql::parse_counts_csv_bytes(&count_bytes)
                .map_err(|e| ApiError::upstream(format!("count parse failed: {e}")))
        };

        let (rows_result, stats_result) = tokio::join!(rows_future, stats_future);
        let rows = rows_result?;
        let stats = match stats_result {
            Ok(stats) => stats,
            Err(err) => {
                log::warn!(
                    "event=search state=count_fallback reason={} include_counts=true",
                    err.message
                );
                DatasetStats::from_entries(&rows)
            }
        };
        return Ok(SearchResponse {
            resolved_taxon_qid,
            warning,
            query: execution_query.to_string(),
            total_matches: stats.n_entries,
            stats: SearchStats::from(stats),
            rows: rows.into_iter().map(RowDto::from).collect(),
        });
    } else {
        let rows_bytes = sparql::execute_sparql_bytes(&display_query)
            .await
            .map_err(|e| ApiError::upstream(format!("display query failed: {e}")))?;
        sparql::parse_compounds_csv_display_bytes(&rows_bytes, limit)
            .map_err(|e| ApiError::upstream(format!("display parse failed: {e}")))?
    };

    let stats = DatasetStats::from_entries(&rows);

    Ok(SearchResponse {
        resolved_taxon_qid,
        warning,
        query: execution_query.to_string(),
        rows: rows.into_iter().map(RowDto::from).collect(),
        total_matches: stats.n_entries,
        stats: stats.into(),
    })
}

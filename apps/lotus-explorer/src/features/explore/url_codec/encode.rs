use crate::models::SearchCriteria;
use std::fmt::Write;

pub fn build_shareable_url(criteria: &SearchCriteria) -> Option<String> {
    let params = criteria.shareable_query_params();
    if params.is_empty() {
        return None;
    }
    let query = build_query_string_from_pairs(params.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    Some(format!("?{query}"))
}

#[cfg(target_arch = "wasm32")]
pub fn build_query_string(params: &super::QueryParams) -> String {
    build_query_string_from_pairs(params.iter().map(|(k, v)| (k.as_str(), v.as_str())))
}

fn build_query_string_from_pairs<'a>(iter: impl Iterator<Item = (&'a str, &'a str)>) -> String {
    let mut query = String::new();
    for (index, (key, value)) in iter.enumerate() {
        if index > 0 {
            query.push('&');
        }
        let _ = write!(
            query,
            "{}={}",
            urlencoding::encode(key),
            urlencoding::encode(value)
        );
    }
    query
}

#[cfg(test)]
pub(super) fn build_query_string_for_tests(params: &super::QueryParams) -> String {
    build_query_string_from_pairs(params.iter().map(|(k, v)| (k.as_str(), v.as_str())))
}

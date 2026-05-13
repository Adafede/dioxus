// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::models::{CompoundEntry, SearchCriteria};
use sha2::{Digest, Sha256};

pub fn sanitize_taxon_input(taxon: &str) -> String {
    let replaced = taxon.replace('_', " ");
    let parts: Vec<&str> = replaced.split_whitespace().collect();
    if parts.len() > 1 {
        let first = parts[0];
        if first.is_empty() {
            return replaced;
        }
        let mut first_cap = String::with_capacity(first.len());
        let mut chars = first.chars();
        if let Some(c) = chars.next() {
            for uc in c.to_uppercase() {
                first_cap.push(uc);
            }
        }
        for c in chars {
            for lc in c.to_lowercase() {
                first_cap.push(lc);
            }
        }
        let mut out = first_cap;
        out.push(' ');
        out.push_str(&parts[1..].join(" "));
        out
    } else {
        replaced
    }
}

pub fn compute_hashes(
    qid: &str,
    criteria: &SearchCriteria,
    rows: &[CompoundEntry],
) -> (String, String) {
    let normalized_qid = if qid.trim().is_empty() { "*" } else { qid };
    let normalized_taxon = criteria.taxon.trim();
    let mut query_source = format!("{}|{}", normalized_qid, normalized_taxon);
    let params = criteria.shareable_query_params();
    if !params.is_empty() {
        query_source.push('|');
        query_source.push_str(
            &params
                .into_iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&"),
        );
    }
    let query_hash = to_hex_lower(&Sha256::digest(query_source.as_bytes()));

    let mut compounds = rows
        .iter()
        .map(|e| e.compound_qid.as_ref())
        .collect::<Vec<_>>();
    compounds.sort_unstable();
    compounds.dedup();
    let result_source = compounds.join("|");
    let result_hash = to_hex_lower(&Sha256::digest(result_source.as_bytes()));

    (query_hash, result_hash)
}

pub fn to_hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn sanitizes_taxon_prefix_case_only() {
        assert_eq!(sanitize_taxon_input("voacanga africana"), "Voacanga africana");
        assert_eq!(sanitize_taxon_input("VOACANGA africana"), "Voacanga africana");
        assert_eq!(sanitize_taxon_input("__gentiana_lutea__"), "Gentiana lutea");
    }

    #[test]
    fn hex_encoder_is_lowercase() {
        assert_eq!(to_hex_lower(&[0xAB, 0xCD, 0xEF]), "abcdef");
    }

    #[test]
    fn hashes_depend_on_query_and_rows_only() {
        let crit = SearchCriteria {
            taxon: "*".into(),
            ..SearchCriteria::default()
        };
        let row = CompoundEntry {
            compound_qid: Arc::from("Q1"),
            name: Arc::from("A"),
            inchikey: None,
            smiles: None,
            mass: None,
            formula: None,
            taxon_qid: Arc::from("Q2"),
            taxon_name: Arc::from("Taxon"),
            reference_qid: Arc::from("Q3"),
            ref_title: None,
            ref_doi: None,
            pub_year: None,
            statement: None,
        };
        let (q1, r1) = compute_hashes("QX", &crit, std::slice::from_ref(&row));
        let (q2, r2) = compute_hashes("QX", &crit, &[row]);
        assert_eq!(q1, q2);
        assert_eq!(r1, r2);
    }
}


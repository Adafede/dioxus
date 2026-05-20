use crate::features::explore::types::{DomainError, ParseFault, TaxonWarning};
use crate::models::TaxonMatch;

pub(super) struct MatchSelection<'a> {
    pub best: &'a TaxonMatch,
    pub warning: Option<TaxonWarning>,
}

pub(super) fn pick_best_match<'a>(
    sanitized: &str,
    matches: &'a [TaxonMatch],
) -> Result<MatchSelection<'a>, DomainError> {
    let lower = sanitized.to_lowercase();
    let exact: Vec<&TaxonMatch> = matches
        .iter()
        .filter(|candidate| candidate.name.to_lowercase() == lower)
        .collect();

    let best = exact
        .first()
        .copied()
        .or_else(|| matches.first())
        .ok_or_else(|| {
            DomainError::Parse(ParseFault::TaxonPick {
                details: "no candidates after parse".to_string(),
            })
        })?;

    let warning = if exact.len() > 1 || (exact.is_empty() && matches.len() > 1) {
        Some(TaxonWarning::Ambiguous {
            chosen_name: best.name.clone(),
            chosen_qid: best.qid.clone(),
            candidates: matches
                .iter()
                .take(4)
                .map(|candidate| format!("{} ({})", candidate.name, candidate.qid))
                .collect(),
        })
    } else {
        None
    };

    Ok(MatchSelection { best, warning })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(name: &str, qid: &str) -> TaxonMatch {
        TaxonMatch {
            qid: qid.to_string(),
            name: name.to_string(),
        }
    }

    #[test]
    fn exact_match_is_preferred_over_first_non_exact_candidate() {
        let matches = vec![candidate("Rosa rubiginosa", "Q2"), candidate("Rosa", "Q1")];

        let selection = pick_best_match("rosa", &matches).expect("selection should succeed");

        assert_eq!(selection.best.qid, "Q1");
        assert!(selection.warning.is_none());
    }

    #[test]
    fn multiple_candidates_without_exact_match_emit_ambiguity_warning() {
        let matches = vec![
            candidate("Rosa rubiginosa", "Q2"),
            candidate("Rosa canina", "Q3"),
        ];

        let selection = pick_best_match("rosa", &matches).expect("selection should succeed");

        assert_eq!(selection.best.qid, "Q2");
        assert!(matches!(
            selection.warning,
            Some(TaxonWarning::Ambiguous { .. })
        ));
    }

    #[test]
    fn duplicate_exact_matches_emit_ambiguity_warning() {
        let matches = vec![candidate("Rosa", "Q1"), candidate("rosa", "Q2")];

        let selection = pick_best_match("rosa", &matches).expect("selection should succeed");

        assert_eq!(selection.best.qid, "Q1");
        assert!(matches!(
            selection.warning,
            Some(TaxonWarning::Ambiguous { .. })
        ));
    }
}

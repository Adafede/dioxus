// The `embedding_size` tests, extracted from `graph.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `graph` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;
use chematic::smarts::find_matches;
use chematic::smiles::parse;

/// Every embedding of one query covers the same number of atoms.
///
/// This is the reason `best_match` below takes the first hit rather than
/// searching for a better one: `find_matches` maps each query atom onto a
/// distinct molecule atom, so a hit's length is `q.atoms.len()` whatever
/// the embedding, and there is no "best" to rank. The test is here because
/// that is an invariant of the matcher, not of this function, and the
/// arithmetic that used to rank hits was computing a constant.
#[test]
fn every_embedding_of_a_query_has_the_same_size() {
    // "CC" into "CCCC" has three embeddings, and each maps both query atoms.
    let q = molecule_to_query(&parse("CC").expect("valid SMILES"));
    let mol = parse("CCCC").expect("valid SMILES");
    let hits = find_matches(&q, &mol);
    assert!(hits.len() > 1, "the fixture must have several embeddings");
    for hit in &hits {
        assert_eq!(
            hit.len(),
            q.atoms.len(),
            "an embedding covers every query atom"
        );
    }
}

/// The same for a query with a branch, where the embeddings differ in which
/// atom they start from.
#[test]
fn a_branched_query_still_has_uniformly_sized_embeddings() {
    let q = molecule_to_query(&parse("CC(C)C").expect("valid SMILES"));
    let mol = parse("CC(C)CC(C)C").expect("valid SMILES");
    let hits = find_matches(&q, &mol);
    assert!(hits.len() > 1, "the fixture must have several embeddings");
    for hit in &hits {
        assert_eq!(hit.len(), q.atoms.len(), "one molecule atom per query atom");
    }
}

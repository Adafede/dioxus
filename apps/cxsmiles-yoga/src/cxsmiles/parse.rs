// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Input parsing and single-linkage clustering.
//!
//! `parse_list` turns raw SMILES strings into `Molecule`s; `cluster` groups
//! them by ECFP4/Tanimoto similarity so unrelated structures are never forced
//! into one nonsensical CX-SMILES.

use chematic::core::Molecule;
use chematic::fp::{BitVec2048, ecfp4};
use chematic::smiles::parse;

use super::types::CxError;

/// Parse a list of raw SMILES strings into molecules, skipping blanks.
pub(crate) fn parse_list(smiles: &[String]) -> Result<Vec<Molecule>, CxError> {
    let mut mols = Vec::new();
    for s in smiles {
        let t = s.trim();
        if t.is_empty() {
            continue;
        }
        match parse(t) {
            Ok(m) => mols.push(m),
            Err(e) => return Err(CxError(format!("could not parse '{t}': {e:?}"))),
        }
    }
    Ok(mols)
}

/// Single-linkage clustering by ECFP4/Tanimoto above `threshold`.
// Every index below is either a loop counter over `0..n` or a value `find`
// returned from that same range, into vectors that each have exactly `n` slots
// (`parent`, `fps`, `roots`, `mols`, and `order`, a permutation of `0..n`).
// `n == 0` returns above, so none of them can miss. Threading an `Option`
// through the two hottest loops in the function to report a state the code above
// has already excluded would be noise, not safety.
#[allow(clippy::indexing_slicing)]
pub(crate) fn cluster(mols: &[Molecule], threshold: f64) -> Vec<Vec<Molecule>> {
    let n = mols.len();
    if n == 0 {
        return Vec::new();
    }
    let fps: Vec<BitVec2048> = mols.iter().map(ecfp4).collect();
    let mut parent: Vec<usize> = (0..n).collect();
    let find = |p: &mut [usize], x: usize| -> usize {
        let mut r = x;
        while p[r] != r {
            r = p[r];
        }
        let mut x = x;
        while p[x] != r {
            let next = p[x];
            p[x] = r;
            x = next;
        }
        r
    };
    for i in 0..n {
        for j in (i + 1)..n {
            if fps[i].tanimoto(&fps[j]) >= threshold {
                let (ri, rj) = (find(&mut parent, i), find(&mut parent, j));
                if ri != rj {
                    parent[ri] = rj;
                }
            }
        }
    }
    let roots: Vec<usize> = (0..n).map(|i| find(&mut parent, i)).collect();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&i| roots[i]);
    let mut buckets: Vec<Vec<Molecule>> = Vec::new();
    let mut cur = roots[order[0]];
    let mut bucket: Vec<Molecule> = vec![mols[order[0]].clone()];
    for &i in &order[1..] {
        if roots[i] == cur {
            bucket.push(mols[i].clone());
        } else {
            buckets.push(bucket);
            cur = roots[i];
            bucket = vec![mols[i].clone()];
        }
    }
    buckets.push(bucket);
    buckets
}

#[cfg(test)]
// `expect` on a fixture that stopped parsing is a broken test, not a defect in
// the code under test; the out-of-range reads below are the assertions failing.
#[allow(clippy::indexing_slicing, clippy::expect_used)]
mod tests {
    use super::*;
    use chematic::smiles::parse;

    fn mols(smiles: &[&str]) -> Vec<Molecule> {
        smiles
            .iter()
            .map(|s| parse(s).expect("the fixtures in this module are valid SMILES"))
            .collect()
    }

    /// The error `parse_list` returns, or a description of what it accepted
    /// instead. `expect_err` needs `T: Debug` and `Molecule` is not `Debug`, so
    /// the failure has to be read out of a `match` — which is also a better
    /// report, because it says how many molecules came back when none should
    /// have.
    fn parse_failure(list: &[String]) -> String {
        match parse_list(list) {
            Ok(parsed) => format!("expected a parse error, got {} molecules", parsed.len()),
            Err(err) => err.to_string(),
        }
    }

    // ── parse_list ──────────────────────────────────────────────────────────

    #[test]
    fn blanks_and_whitespace_are_skipped() {
        let list = vec![
            "CCO".to_string(),
            String::new(),
            "   ".to_string(),
            "\t".to_string(),
            "\r\n".to_string(),
            "CC".to_string(),
        ];
        let parsed = parse_list(&list).expect("the non-blank lines are valid");
        assert_eq!(parsed.len(), 2, "only the two lines with content");
    }

    #[test]
    fn surrounding_whitespace_is_trimmed_before_parsing() {
        // A pasted SMILES list is full of trailing spaces, and a trim that
        // stopped happening would turn a valid line into a parse error.
        let list = vec!["  CCO  ".to_string()];
        let parsed = parse_list(&list).expect("whitespace around a SMILES is not syntax");
        assert_eq!(parsed.len(), 1, "the line parsed");
    }

    #[test]
    fn one_unparseable_line_fails_the_whole_list() {
        // Failing loudly beats returning a partial list: the caller decides
        // what to do about a molecule it never saw, and it cannot do that about
        // one that was silently dropped.
        let list = vec!["CCO".to_string(), "not a smiles".to_string()];
        let err = parse_failure(&list);
        assert!(
            err.contains("could not parse"),
            "the error says what failed: {err}"
        );
    }

    #[test]
    fn the_error_names_the_offending_line() {
        // A list of a thousand lines and one bad one: the message is the only
        // thing that points at it.
        let list = vec!["CCO".to_string(), "C(((".to_string()];
        let err = parse_failure(&list);
        assert!(err.contains("C((("), "the error quotes the input: {err}");
    }

    #[test]
    fn an_empty_list_parses_to_nothing() {
        assert!(
            parse_list(&[])
                .expect("an empty list has nothing to reject")
                .is_empty(),
            "no lines, no molecules"
        );
    }

    #[test]
    fn a_list_of_only_blanks_parses_to_nothing() {
        let list = vec![String::new(), "  ".to_string()];
        assert!(
            parse_list(&list).expect("blanks are not errors").is_empty(),
            "blanks are skipped, and skipping all of them is not a failure"
        );
    }

    // ── cluster ─────────────────────────────────────────────────────────────

    #[test]
    fn one_molecule_is_one_cluster() {
        let c = cluster(&mols(&["CCO"]), 0.5);
        assert_eq!(c.len(), 1, "a single input is a single cluster");
        assert_eq!(c[0].len(), 1, "holding that one molecule");
    }

    #[test]
    fn no_molecules_is_no_clusters() {
        // The `n == 0` early return. Without it the code would index
        // `order[0]` on an empty permutation.
        assert!(
            cluster(&[], 0.5).is_empty(),
            "nothing to cluster is not one empty cluster"
        );
    }

    #[test]
    fn an_identical_molecule_clusters_with_its_copy() {
        // Every pair is identical, so single linkage joins all of them.
        let c = cluster(&mols(&["CCO", "CCO", "CCO"]), 0.5);
        assert_eq!(c.len(), 1, "three identical molecules, one group");
        assert_eq!(c[0].len(), 3, "all of them together");
    }

    #[test]
    fn every_molecule_lands_in_exactly_one_cluster() {
        // The property that makes the partition usable: a union-find that
        // forgets to follow a parent pointer loses molecules, and one that
        // over-merges puts one in two groups.
        let input = ["CCO", "CCO", "CCC", "c1ccccc1", "C1CC1"];
        let c = cluster(&mols(&input), 0.5);
        let total: usize = c.iter().map(Vec::len).sum();
        assert_eq!(total, input.len(), "every input appears exactly once");
    }

    #[test]
    fn a_threshold_of_zero_merges_everything() {
        // Every Tanimoto is >= 0, so single linkage joins all of them however
        // unrelated they are. That is the documented meaning of the parameter,
        // and it is the one threshold where `>=` rather than `>` is visible.
        let c = cluster(&mols(&["CCO", "c1ccccc1"]), 0.0);
        assert_eq!(c.len(), 1, "a threshold of zero accepts every pair");
    }

    #[test]
    fn an_unreachable_threshold_leaves_each_molecule_alone() {
        // A Tanimoto is at most 1, so a threshold above 1 rejects every pair
        // and each molecule is its own cluster. Nothing is dropped, which is
        // the failure mode a merge bug would show up in.
        let c = cluster(&mols(&["CCO", "c1ccccc1"]), 2.0);
        assert_eq!(c.len(), 2, "no pair passes an impossible threshold");
        assert_eq!(
            c.iter().map(Vec::len).sum::<usize>(),
            2,
            "and both molecules are still accounted for"
        );
    }

    // Measured ECFP4/Tanimoto for the chain below, so the threshold in the test
    // is between two real numbers rather than a guess:
    //
    //   CCO    ~ CCCO    0.333
    //   CCCO   ~ CCCCCO  0.562
    //   CCO    ~ CCCCCO  below 0.30
    const CHAIN: f64 = 0.30;

    #[test]
    fn single_linkage_chains_transitively() {
        // The defining property of *single* linkage, and the reason for the
        // union-find: A is similar to B and B to C, but A and C are not, and all
        // three still belong to one group. A test built only from pairs could
        // not tell single linkage from complete linkage, which would have put
        // CCO and CCCCCO in different groups.
        let c = cluster(&mols(&["CCO", "CCCO", "CCCCCO"]), CHAIN);
        assert_eq!(c.len(), 1, "the chain is followed through the middle");
        assert_eq!(c[0].len(), 3, "all three end up together");
    }

    #[test]
    fn the_same_chain_breaks_when_the_threshold_excludes_a_link() {
        // 0.40 is above the 0.333 link and below the 0.562 one, so the first
        // pair no longer joins and the chain has to be cut. Pinning this is
        // what shows the previous test is about linkage rather than about the
        // molecules happening to be alike.
        let c = cluster(&mols(&["CCO", "CCCO", "CCCCCO"]), 0.40);
        assert_eq!(c.len(), 2, "the weak link is gone, so the chain is in two");
        // Sorted, because which cluster comes out first is an artefact of which
        // molecule ended up as the union-find representative — not a contract.
        let mut sizes = c.iter().map(Vec::len).collect::<Vec<_>>();
        sizes.sort_unstable();
        assert_eq!(
            sizes,
            vec![1, 2],
            "the strong pair together and the weak one alone"
        );
    }

    #[test]
    fn molecules_sharing_a_root_land_in_the_same_bucket() {
        // `order.sort_by_key(|i| roots[i])` is what puts molecules that share a
        // root next to each other. Without the sort the loop below would close
        // and reopen a bucket for every molecule, and a group of three would
        // arrive as three groups of one — the right molecules, the wrong
        // partition, and no error anywhere.
        let c = cluster(&mols(&["CCO", "CCCO", "CCCCCO", "CC(=O)Nc1ccccc1"]), CHAIN);
        let sizes = c.iter().map(Vec::len).collect::<Vec<_>>();
        assert_eq!(sizes.len(), 2, "the amide is unlike the alcohols");
        assert!(
            sizes.contains(&3),
            "the three similar alcohols are one bucket, not three of one: {sizes:?}"
        );
        assert_eq!(
            sizes.iter().sum::<usize>(),
            4,
            "and every molecule is still in exactly one bucket"
        );
    }
}

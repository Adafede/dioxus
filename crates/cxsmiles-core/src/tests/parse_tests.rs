// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

use chematic::core::Molecule;

use super::mol;
use crate::parse::{cluster, find_root, parse_list};

fn mols(smiles: &[&str]) -> Vec<Molecule> {
    smiles.iter().map(|s| mol(s)).collect()
}

fn parse_failure(list: &[String]) -> String {
    match parse_list(list) {
        Ok(parsed) => format!("expected a parse error, got {} molecules", parsed.len()),
        Err(err) => err.to_string(),
    }
}

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
    let list = vec!["  CCO  ".to_string()];
    let parsed = parse_list(&list).expect("whitespace around a SMILES is not syntax");
    assert_eq!(parsed.len(), 1, "the line parsed");
}

#[test]
fn one_unparsable_line_fails_the_whole_list() {
    let list = vec!["CCO".to_string(), "not a smiles".to_string()];
    let err = parse_failure(&list);
    assert!(
        err.contains("could not parse"),
        "the error says what failed: {err}"
    );
}

#[test]
fn the_error_names_the_offending_line() {
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

#[test]
fn one_molecule_is_one_cluster() {
    let c = cluster(&mols(&["CCO"]), 0.5);
    assert_eq!(c.len(), 1, "a single input is a single cluster");
    assert_eq!(c[0].len(), 1, "holding that one molecule");
}

#[test]
fn no_molecules_is_no_clusters() {
    assert!(
        cluster(&[], 0.5).is_empty(),
        "nothing to cluster is not one empty cluster"
    );
}

#[test]
fn an_identical_molecule_clusters_with_its_copy() {
    let c = cluster(&mols(&["CCO", "CCO", "CCO"]), 0.5);
    assert_eq!(c.len(), 1, "three identical molecules, one group");
    assert_eq!(c[0].len(), 3, "all of them together");
}

#[test]
fn every_molecule_lands_in_exactly_one_cluster() {
    let input = ["CCO", "CCO", "CCC", "c1ccccc1", "C1CC1"];
    let c = cluster(&mols(&input), 0.5);
    let total: usize = c.iter().map(Vec::len).sum();
    assert_eq!(total, input.len(), "every input appears exactly once");
}

#[test]
fn a_threshold_of_zero_merges_everything() {
    let c = cluster(&mols(&["CCO", "c1ccccc1"]), 0.0);
    assert_eq!(c.len(), 1, "a threshold of zero accepts every pair");
}

#[test]
fn an_unreachable_threshold_leaves_each_molecule_alone() {
    let c = cluster(&mols(&["CCO", "c1ccccc1"]), 2.0);
    assert_eq!(c.len(), 2, "no pair passes an impossible threshold");
    assert_eq!(
        c.iter().map(Vec::len).sum::<usize>(),
        2,
        "and both molecules are still accounted for"
    );
}

const CHAIN: f64 = 0.30;

#[test]
fn single_linkage_chains_transitively() {
    let c = cluster(&mols(&["CCO", "CCCO", "CCCCCO"]), CHAIN);
    assert_eq!(c.len(), 1, "the chain is followed through the middle");
    assert_eq!(c[0].len(), 3, "all three end up together");
}

#[test]
fn the_same_chain_breaks_when_the_threshold_excludes_a_link() {
    let c = cluster(&mols(&["CCO", "CCCO", "CCCCCO"]), 0.40);
    assert_eq!(c.len(), 2, "the weak link is gone, so the chain is in two");
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

fn chain_parents(links: &[(usize, usize)]) -> Vec<usize> {
    let mut parent: Vec<usize> = (0..4).collect();
    for &(from, to) in links {
        parent[from] = to;
    }
    parent
}

#[test]
fn find_root_walks_a_chain_to_the_end_of_it() {
    let mut parent = chain_parents(&[(3, 2), (2, 1), (1, 0)]);
    assert_eq!(find_root(&mut parent, 3), 0, "the far end of the chain");
    assert_eq!(find_root(&mut parent, 2), 0, "and every point on it");
    assert_eq!(find_root(&mut parent, 1), 0, "up to the last hop");
}

#[test]
fn find_root_leaves_an_element_that_is_already_a_root_alone() {
    let mut parent = chain_parents(&[(2, 0)]);
    assert_eq!(find_root(&mut parent, 0), 0, "a root is itself");
    assert_eq!(parent[0], 0, "and its own slot still says so");
}

#[test]
fn find_root_points_a_middle_of_the_chain_straight_at_the_root() {
    let mut parent = chain_parents(&[(3, 2), (2, 1), (1, 0)]);
    assert_eq!(find_root(&mut parent, 3), 0, "the root is unchanged");
    assert_eq!(
        parent[3], 0,
        "and the element the walk started from now points there directly"
    );

    let mut parent = chain_parents(&[(3, 2), (2, 1), (1, 0)]);
    find_root(&mut parent, 2);
    assert_eq!(parent[2], 0, "one hop is collapsed to no hops");
}

#[test]
fn find_root_on_a_flat_array_returns_each_element() {
    let mut parent: Vec<usize> = (0..5).collect();
    for i in 0..5 {
        assert_eq!(find_root(&mut parent, i), i, "element {i} is its own root");
    }
}

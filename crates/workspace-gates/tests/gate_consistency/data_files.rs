// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

// The `data_files` gates, extracted from `gate_consistency.rs`.
//
// They share the helpers in `main.rs` -- `read`, `makefiles` and the rest --
// so each module opens with `use super::*`. Split by what is being checked
// rather than by size: the groups fail for different reasons and get read
// separately.
use super::*;

/// Data that must never be committed, and the reason each one exists.
///
/// This is a list of paths, not a list of rules: it is here so that deleting a
/// `.gitignore` line has to be done in two places, and so the two places can be
/// compared. The largest of these is 33 MB of third-party LIPID MAPS data, and a
/// licence is not something a later commit can take back.
const LOCAL_ONLY_DATA: &[(&str, &str)] = &[
    (
        "/apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/LMSD.sdf.zip",
        "downloaded from lipidmaps.org by `smarts-evoliposuction download`",
    ),
    (
        "/apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/LMSD.sdf.tsv",
        "the same records as the zip, converted to TSV",
    ),
    (
        "/apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction/example_lipids.smi",
        "the scratch input used while developing a rule set",
    ),
    (
        "/smarts_results.csv",
        "the tool's default output name, so it lands in whatever directory it was run from",
    ),
    (
        "/example_lipids.smi",
        "a copy of the scratch input, left in the repository root",
    ),
    ("/smiles_sets/", "70 MB of SMILES split out of the TSV"),
    (
        "/smarts_results_tested_smarts/",
        "the tested SMARTS per class",
    ),
];

/// Every one of them has an ignore rule, and the rule is the path.
///
/// Text, not `git check-ignore`: this has to run in a checkout with no `.git` at
/// all, and a subprocess that fails is a test that fails for the wrong reason.
/// A rule that is present but does not match is a different bug, and the
/// simplest way to rule it out is to assert the rule *is* the path.
#[test]
fn the_local_data_files_are_ignored() -> Result<()> {
    let gitignore = read(".gitignore")?;
    let mut missing = Vec::new();

    for (path, why) in LOCAL_ONLY_DATA {
        let rule = format!("\n{path}\n");
        if !gitignore.contains(&rule) {
            missing.push(format!("{path} ({why})"));
        }
    }

    assert!(
        missing.is_empty(),
        "these are outputs of `smarts-evoliposuction` and its input data, not \
         source, and `git add -A` would commit them: {missing:?}. Add each to \
         `.gitignore`, or delete it here and in LOCAL_ONLY_DATA on purpose."
    );
    assert!(
        !LOCAL_ONLY_DATA.is_empty(),
        "LOCAL_ONLY_DATA is empty, so this test proves nothing"
    );
    Ok(())
}

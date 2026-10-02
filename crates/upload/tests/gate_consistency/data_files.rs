// The `data_files` gates, extracted from `gate_consistency.rs`.
//
// They share the helpers in that file -- `read`, `makefiles`,
// `task_dependencies` and the rest -- so each module opens with
// `use super::*`. Split by what is being checked rather than by
// size: the groups fail for different reasons and get read
// separately.
use super::*;

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

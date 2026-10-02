// The `apps` gates, extracted from `gate_consistency.rs`.
//
// They share the helpers in that file -- `read`, `makefiles`,
// `task_dependencies` and the rest -- so each module opens with
// `use super::*`. Split by what is being checked rather than by
// size: the groups fail for different reasons and get read
// separately.
use super::*;

/// Every app, against the two halves of its skip link.
///
/// A working skip link needs three things in three different places: an app
/// that renders the link, a `main` element whose `id` is what the link's
/// `href` says, and a `DocumentHead` -- which is what injects the focus rule
/// that un-hides the link, since `ui` parks it at `top: -100%`. None of the
/// three is visible to a linter from any of the others, and the failure mode
/// is a link that reads correctly in the source and goes nowhere in the
/// browser.
///
/// This is what found that `index` and `mgf-precursor-erro-rs` rendered no
/// un-hiding rule, that `cxsmiles-yoga`, `lipid-selecto-rs` and `smellfish-rs`
/// rendered no skip link at all, that `smellfish-rs` had no `main` element to
/// point at, and that `lipid-selecto-rs` and `mgf-precursor-erro-rs` named
/// theirs `#main` while the other four named theirs `#main-content`.
#[test]
fn every_app_pairs_its_skip_link_with_a_main_landmark() -> Result<()> {
    let root = repo_root();
    let mut apps: Vec<String> = std::fs::read_dir(root.join("apps"))?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().join("Cargo.toml").is_file())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    apps.sort();
    assert!(!apps.is_empty(), "found no apps to check");

    let mut broken: Vec<String> = Vec::new();

    for app in &apps {
        let src_dir = root.join("apps").join(app).join("src");
        let mut body = String::new();
        let mut stack = vec![src_dir];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir)? {
                let path = entry?.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    body.push_str(&std::fs::read_to_string(&path)?);
                    body.push('\n');
                }
            }
        }

        let mut fail = |reason: &str| broken.push(format!("{app}: {reason}"));

        if !body.contains("skip_link {") {
            fail("renders no skip link");
        }
        if body.matches("skip_link {").count() > 1 {
            fail("renders more than one skip link");
        }
        if !body.contains(MAIN_LANDMARK_ID) {
            fail("nothing carries id=\"main-content\" for the link to point at");
        }
        if !body.contains("DocumentHead") {
            fail("renders no DocumentHead, so nothing injects the focus rule");
        }

        // The link's href is fixed in `ui::skip_link`. An app that spells out a
        // target is not using that component, and so is not covered by the
        // focus rule or by the id checked above.
        if body
            .lines()
            .any(|l| l.contains("skip_link {") && l.contains("target:"))
        {
            fail("passes a target to skip_link, but ui::skip_link takes no props");
        }
    }

    assert!(
        broken.is_empty(),
        "skip-link wiring is wrong:\n  {}",
        broken.join("\n  ")
    );
    Ok(())
}

/// `smarts-evoliposuction` is excluded from the workspace, so nothing in CI
/// compiles it, tests it, or lints it. It sat that way while not compiling at
/// all: it imported `smiles_parser`, which is not in its manifest, and the root
/// `Cargo.toml` claimed it "builds when the line above is uncommented".
///
/// Neither claim is checkable from here -- building it needs its three git
/// dependencies and a network -- so what this asserts is the part that is: that
/// it is *excluded* rather than half-membered, that it declares itself a
/// standalone workspace (without which Cargo refuses to build it at all), and
/// that it no longer inherits anything, because inheritance requires membership
/// and would silently reintroduce the state this test exists to catch.
///
/// `cargo build` in that directory is still the real check, and still not
/// something `./mk ci` can run. Kept here so the exclusion is at least a
/// decision that is written down and checked, rather than a stale comment.
#[test]
fn the_excluded_tool_declares_itself_a_standalone_workspace() -> Result<()> {
    const TOOL: &str = "apps/lipid-selecto-rs/lipidmaps/smarts-evoliposuction";

    let root_manifest = read("Cargo.toml")?;
    assert!(
        root_manifest.contains(&format!("exclude = [\"{TOOL}\"]")),
        "{TOOL} is not in the root `exclude`. Either it was added to `members`, \
         which changes the shipped dependency graph, or it is neither a member \
         nor excluded -- in which case Cargo refuses to build it."
    );
    assert!(
        !root_manifest.contains(&format!("\"{TOOL}\",")),
        "{TOOL} is back in `members`"
    );

    let tool_manifest = read(&format!("{TOOL}/Cargo.toml"))?;

    // Comments are stripped before anything is searched. The manifest explains at
    // length why it neither inherits nor joins the workspace, and that prose
    // quotes `[workspace]` and `workspace = true` verbatim -- so a search over the
    // raw text finds this test's own subject matter in a comment and passes a
    // manifest that has the very problem being looked for. Not hypothetical:
    // deleting the `[workspace]` table left the test green until this.
    let code: String = tool_manifest
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        code.contains("[workspace]"),
        "{TOOL}/Cargo.toml has no `[workspace]` table, so Cargo walks up to the \
         root, finds it is not a member, and refuses to build it"
    );

    for inherit in [
        ".workspace = true",  // [package] and [lints] inheritance
        "{ workspace = true", // dependency inheritance
        "{ workspace = true,",
    ] {
        assert!(
            !code.contains(inherit),
            "{TOOL}/Cargo.toml contains `{inherit}` outside a comment, so it \
             still inherits from the workspace, which requires being a member. \
             Either the values are spelled out or it is back in `members`."
        );
    }

    Ok(())
}

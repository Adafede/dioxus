// The `mk_runner` gates, extracted from `gate_consistency.rs`.
//
// They share the helpers in that file -- `read`, `makefiles`,
// `task_dependencies` and the rest -- so each module opens with
// `use super::*`. Split by what is being checked rather than by
// size: the groups fail for different reasons and get read
// separately.
use super::*;

/// The checks whose value is that they *fail*, not that they run.
///
/// Each of these is a line in a task that a passing gate could have without it,
/// and a check that cannot fail is worse than no check: it reports a green run
/// for a thing it never looked at.
#[test]
fn the_failing_checks_still_fail_on_failure() -> Result<()> {
    let makefiles = makefiles()?;
    let ci = read(".github/workflows/ci.yml")?;

    // `cargo doc` without this builds the docs and prints the warning, so the
    // task would pass on a broken intra-doc link -- which is a 404 in the
    // generated documentation and nothing else.
    assert!(
        makefiles.contains("RUSTDOCFLAGS=\"-D warnings"),
        "the `doc` task must run rustdoc with `-D warnings`, or it only proves \
         the docs build"
    );

    // The three `crates/upload` modules exist only under
    // `#[cfg(target_arch = \"wasm32\")]`, so nothing else compiles their code or
    // their tests. A test calling a method that does not exist lived in one of
    // them for the whole life of this gate.
    assert!(
        makefiles.contains("cargo clippy -p upload --target wasm32-unknown-unknown --all-targets"),
        "`lint-wasm-upload` must lint `upload` on its own, tests included"
    );
    assert!(
        ci.contains("./mk ci"),
        "the CI gate job must run `./mk ci` rather than its own list of \
         checks, or the two can drift"
    );

    // The license-header check is one `find` over the tree, in a script rather
    // than in the task; a `grep -x` on the copyright prefix would make it match
    // nothing and pass forever, so assert the shape it needs.
    let headers = read("make/scripts/license-headers.sh")?;
    assert!(
        headers.contains("SPDX-License-Identifier: AGPL-3.0-only"),
        "the license-header check must name the licence it enforces"
    );
    assert!(
        !headers.contains("grep -qxF '// SPDX-FileCopyrightText"),
        "the copyright line is a prefix, not an exact string: `grep -x` on it can \
         never match and the check would pass on every file"
    );
    assert!(
        makefiles.contains("make/scripts/license-headers.sh"),
        "the license-headers task must call the script; the check is the whole \
         gate otherwise"
    );

    // `extend` must be the first key in `Makefile.toml`. cargo-make accepts the
    // file with `[config]` above it, reports no error, and loads none of the
    // task files -- so every task is "not found" and the gate appears to have no
    // tasks at all.
    let root = read("Makefile.toml")?;
    let first_key = root
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .unwrap_or_default();
    assert!(
        first_key.starts_with("extend"),
        "the first key in Makefile.toml must be `extend`, and it is `{first_key}`. \
         With anything above it, cargo-make silently loads none of the task files."
    );

    // Every CI job installs the tools it calls. The `wasm` job needs binaryen as
    // well as `dx`, and the gate needs seven cargo-installed tools; a job that
    // calls one without installing it dies at 127, with a message about the
    // change under test.
    for job in ci_jobs(&ci)? {
        let block: String = ci
            .lines()
            .skip_while(|l| *l != format!("  {job}:"))
            .skip(1)
            .take_while(|l| l.starts_with("   ") || l.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        if block.contains("./mk ") {
            assert!(
                block.contains("cargo-make@"),
                "the {job} job runs `./mk` but does not install cargo-make"
            );
        }
    }
    Ok(())
}

/// `./mk` must exist, be executable, and hold `--no-workspace`.
///
/// Without the flag cargo-make re-runs every task once per workspace member:
/// eight executions of everything that is not a Cargo task, and the whole gate
/// measured at ~2 min against ~17s. There is no config key and no environment
/// variable for it in 0.37 -- `CARGO_MAKE_CRATE_IS_WORKSPACE`,
/// `CARGO_MAKE_WORKSPACE_EMULATION` and `CARGO_MAKE_WORKSPACE_SKIP_MEMBERS` were
/// all tried and all left the fan-out in place -- so the flag in this one file is
/// the whole mechanism, and a task run without it is a task run eight times.
#[test]
fn mk_holds_the_no_workspace_flag() -> Result<()> {
    let path = repo_root().join("mk");
    assert!(
        path.exists(),
        "`./mk` is missing: nothing has a way to run a task once"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path)?.permissions().mode();
        assert!(
            mode & 0o111 != 0,
            "`./mk` is not executable (mode {mode:o}); the hooks and the workflow \
             call it directly, so they would fail with 126"
        );
    }

    let mk = read("mk")?;
    assert!(
        mk.contains("--no-workspace"),
        "`./mk` does not pass `--no-workspace`. Without it every task runs once \
         per workspace member; see the header comment in the file."
    );
    assert!(
        mk.contains("exec cargo make"),
        "`./mk` should `exec` the runner so a signal reaches it and the exit code \
         is the runner's"
    );
    Ok(())
}

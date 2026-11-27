// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

// The `workflows` gates, extracted from `gate_consistency.rs`.
//
// They share the helpers in `main.rs` -- `read`, `makefiles` and the rest --
// so each module opens with `use super::*`. Split by what is being checked
// rather than by size: the groups fail for different reasons and get read
// separately.
use super::*;

/// Every `dtolnay/rust-toolchain` step must pass `toolchain:`.
///
/// The action's `toolchain` input is *required* and it does not fall back to
/// `rust-toolchain.toml`: a step without it exits 1 with "'toolchain' is a
/// required input", before a single check runs. Six steps across three workflows
/// were written that way, on the reasoning that rustup reads the pinned
/// file -- which is true, and is why those six steps do not exist any more, but
/// the action cannot be left in place with its required input unset.
///
/// The MSRV job is the one that must keep the action, because its whole point
/// is a toolchain other than the pinned one. That is why this asserts the input
/// rather than the absence of the action.
#[test]
fn every_rust_toolchain_action_passes_its_required_input() -> Result<()> {
    const REQUIRES_TOOLCHAIN: &str = "dtolnay/rust-toolchain";

    // Every workflow, listed from the directory, so a new one is covered.
    for name in workflow_files()? {
        let file = format!(".github/workflows/{name}");
        let yaml = read(&file)?;
        let mut saw = false;
        for (n, line) in yaml.lines().enumerate() {
            if !line.contains(REQUIRES_TOOLCHAIN) {
                continue;
            }
            saw = true;
            // The `with:` block is the lines indented further than the `- uses:`.
            let rest = yaml.lines().skip(n + 1);
            let inputs: Vec<&str> = rest
                .take_while(|l| l.trim().is_empty() || l.starts_with("        "))
                .collect();
            assert!(
                inputs.iter().any(|l| l.contains("toolchain:")),
                "{file}:{} uses {REQUIRES_TOOLCHAIN} with no `toolchain:` input. \
                 The input is required and the step exits 1 before anything runs. \
                 Either give it the toolchain, or drop the action and let rustup \
                 read `rust-toolchain.toml`.",
                n + 1
            );
        }
        let _ = saw;
    }
    Ok(())
}

/// Nothing in the hooks or the workflows may invoke `cargo make` directly.
///
/// Every one of them delegates to `./mk`, and a bare `cargo make` in a hook is a
/// check that runs eight times — which reads as a slow hook and is not noticed,
/// because a check that runs eight times still passes.
#[test]
fn no_hook_or_workflow_bypasses_mk() -> Result<()> {
    // Every workflow, listed from the directory, so a new one is covered.
    for name in workflow_files()? {
        let file = format!(".github/workflows/{name}");
        let yaml = read(&file)?;
        for (n, line) in yaml.lines().enumerate() {
            let code = line.split('#').next().unwrap_or("");
            if code.contains("cargo make") {
                return Err(format!(
                    "{file}:{} invokes `cargo make` directly. Use `./mk`, which holds \
                     `--no-workspace`: {line}",
                    n + 1
                )
                .into());
            }
        }
    }

    let hooks = read("prek.toml")?;
    for (n, line) in hooks.lines().enumerate() {
        if line.contains("entry = \"cargo make") {
            return Err(format!(
                "prek.toml:{} is a hook that runs `cargo make` directly, so it \
                 pays the eight-member fan-out on every commit: {line}",
                n + 1
            )
            .into());
        }
    }
    Ok(())
}

/// Nothing inside a `${{ ... }}` may be Rust.
///
/// The gap this closes is real and was hit here: a deploy fix used
/// `format!('/{0}/{1}', repo, pkg)` in an `env:` value. That is a Rust macro, and
/// GitHub Actions rejected the workflow with "Unrecognized named-value: 'format'".
/// Every gate in `./mk ci` passed first -- the YAML is well-formed, `actionlint`
/// is not in the toolchain, and nothing here evaluates an expression. A file that
/// GitHub refuses to parse is a broken deploy, and it reached review.
///
/// GitHub's expression language has functions (`format(...)`, `join(...)`) and
/// no macros. So the shape to reject is a call with a `!` on its name, plus the
/// Rust spellings that have no meaning there at all.
#[test]
fn no_workflow_expression_contains_rust() -> Result<()> {
    let mut offenders: Vec<String> = Vec::new();

    for name in workflow_files()? {
        let text = read(&format!(".github/workflows/{name}"))?;
        for (line_no, line) in text.lines().enumerate() {
            for expr in expression_bodies(line) {
                for call in re_macros(expr) {
                    offenders.push(format!(
                        "{name}:{}: `{call}!` is a Rust macro; GitHub's function is \
                         `{call}(...)` with no bang",
                        line_no + 1
                    ));
                }
                for token in ["::", "let ", ".unwrap", ".expect", "Some(", "None", "&mut "] {
                    if expr.contains(token) {
                        offenders.push(format!(
                            "{name}:{}: `{token}` has no meaning in a GitHub expression",
                            line_no + 1
                        ));
                    }
                }
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "GitHub Actions expressions are not Rust:\n  {}",
        offenders.join("\n  ")
    );
    Ok(())
}

/// GitHub Pages upload and deploy must live in separate jobs.
///
/// `actions/deploy-pages` calls `listArtifacts()` once without retries or
/// `latest: true`, so uploading and deploying in the same job can race artifact
/// finalization and then fail every "Re-run failed jobs" attempt with
/// `Multiple artifacts named "github-pages" were unexpectedly found`.
#[test]
fn pages_upload_and_deploy_use_separate_jobs() -> Result<()> {
    let mut offenders = Vec::new();
    let mut saw_upload = false;
    let mut saw_deploy = false;

    for name in workflow_files()? {
        let yaml = read(&format!(".github/workflows/{name}"))?;
        let (found_upload, found_deploy) = scan_pages_jobs(&name, &yaml, &mut offenders);
        saw_upload |= found_upload;
        saw_deploy |= found_deploy;
    }

    assert!(
        saw_upload && saw_deploy,
        "expected both `actions/upload-pages-artifact` and `actions/deploy-pages` \
         in `.github/workflows/`; scraper broken?"
    );
    assert!(
        offenders.is_empty(),
        "Pages upload and deploy must be in separate jobs:\n  {}",
        offenders.join("\n  ")
    );
    Ok(())
}

/// Scans one workflow's `jobs:` map and records any job that invokes both
/// `actions/upload-pages-artifact` and `actions/deploy-pages`.
fn scan_pages_jobs(name: &str, yaml: &str, offenders: &mut Vec<String>) -> (bool, bool) {
    let mut in_jobs = false;
    let mut current_job: Option<&str> = None;
    let mut job_has_upload = false;
    let mut job_has_deploy = false;
    let mut job_reported = false;
    let mut saw_upload = false;
    let mut saw_deploy = false;

    for line in yaml.lines() {
        let code = line.split('#').next().unwrap_or("");
        let trimmed = code.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !code.starts_with([' ', '\t']) {
            in_jobs = trimmed == "jobs:";
            current_job = None;
            continue;
        }
        if !in_jobs {
            continue;
        }
        if let Some(rest) = code.strip_prefix("  ")
            && !rest.starts_with([' ', '\t'])
            && let Some(job_id) = rest.trim().strip_suffix(':')
        {
            current_job = Some(job_id);
            job_has_upload = false;
            job_has_deploy = false;
            job_reported = false;
            continue;
        }
        let Some(job) = current_job else {
            continue;
        };
        if code.contains("actions/upload-pages-artifact") {
            saw_upload = true;
            job_has_upload = true;
        }
        if code.contains("actions/deploy-pages") {
            saw_deploy = true;
            job_has_deploy = true;
        }
        if job_has_upload && job_has_deploy && !job_reported {
            offenders.push(format!("{name}: job `{job}`"));
            job_reported = true;
        }
    }

    (saw_upload, saw_deploy)
}

/// Workflow file names, read from the directory so a new workflow is covered
/// without editing this list.
fn workflow_files() -> Result<Vec<String>> {
    let dir = repo_root().join(".github/workflows");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            // Extension compared case-insensitively, because `.YML` is a workflow
            // to GitHub and would otherwise slip past this gate.
            e.path()
                .extension()
                .and_then(std::ffi::OsStr::to_str)
                .is_some_and(|ext| {
                    ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml")
                })
        })
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    Ok(names)
}

/// The bodies of every `${{ ... }}` on one line.
fn expression_bodies(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.find("${{") {
        let Some(after) = rest.get(open + 3..) else {
            break;
        };
        match after.find("}}") {
            Some(close) => {
                if let Some(body) = after.get(..close) {
                    out.push(body);
                }
                rest = after.get(close + 2..).unwrap_or("");
            }
            None => break,
        }
    }
    out
}

/// Names called with a `!` inside an expression body, ignoring anything in
/// single quotes.
fn re_macros(expr: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut in_quotes = false;
    for (i, c) in expr.char_indices() {
        if c == '\'' {
            in_quotes = !in_quotes;
        }
        if c != '!' || in_quotes {
            continue;
        }
        let prefix = expr.get(..i).unwrap_or("");
        let start = prefix
            .rfind(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
            .map_or(0, |pos| pos + 1);
        if let Some(name) = prefix.get(start..)
            && !name.is_empty()
        {
            found.push(name);
        }
    }
    found
}

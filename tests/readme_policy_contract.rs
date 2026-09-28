use std::path::{Path, PathBuf};

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn read_file(relative: &str) -> String {
    let path = repo_path(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("Expected {} to be readable: {}", path.display(), error))
        .replace("\r\n", "\n")
}

#[test]
fn readme_policy_uses_locked_upstream_tooling() {
    let package = read_file("ci/readme-policy/package.json");
    let remark = read_file("ci/readme-policy/.remarkrc.json");

    for required in [
        r#""remark-cli": "12.0.1""#,
        r#""remark-preset-lint-consistent": "6.0.1""#,
        r#""remark-preset-lint-recommended": "7.0.1""#,
        r#""standard-readme-preset": "1.0.13""#,
    ] {
        assert!(
            package.contains(required),
            "README policy must pin upstream dependency: {required}"
        );
    }

    for required in [
        r#""standard-readme-preset""#,
        r#""standard-readme-preset/rules/require-sections.js""#,
        r#""installable": true"#,
        r#""remark-preset-lint-recommended""#,
        r#""remark-preset-lint-consistent""#,
    ] {
        assert!(
            remark.contains(required),
            "README policy must configure upstream contract: {required}"
        );
    }
}

#[test]
fn verify_workflow_enforces_readme_policy_fail_closed() {
    let workflow = read_file(".github/workflows/verify.yml");

    for required in [
        "name: Verify",
        "name: Required CI",
        "Enforce Standard Readme and remark-lint",
        "docker.io/library/node:24.14.1-bookworm-slim@sha256:b506e7321f176aae77317f99d67a24b272c1f09f1d10f1761f2773447d8da26c",
        r#"--volume "$PWD:/remove-chrome-ai""#,
        "--workdir /remove-chrome-ai",
        "NPM_CONFIG_IGNORE_SCRIPTS=true npm ci",
        "ci/readme-policy/node_modules/.bin/remark README.md --frail --rc-path ci/readme-policy/.remarkrc.json",
        "Check README links",
        "lycheeverse/lychee:0.24.2@sha256:e2d19e57cf6ab037026f20b8e449a1f30d9d7f81eef4194763aab2eab20bd28d",
        r#"--volume "$PWD:/remove-chrome-ai:ro""#,
        "--no-progress --root-dir . README.md",
    ] {
        assert!(
            workflow.contains(required),
            "Verify workflow must enforce README contract: {required}"
        );
    }

    assert!(!workflow.contains("|| true"));
    assert!(!workflow.contains("continue-on-error"));
}

#[test]
fn readme_has_standard_structure_and_preserves_safety_guidance() {
    let readme = read_file("README.md");

    assert!(readme.starts_with("# remove-chrome-ai\n"));

    let required = [
        "## Table of Contents",
        "## Install",
        "## Usage",
        "## Features",
        "## Configuration",
        "## Architecture",
        "## Development",
        "## Releases",
        "## Contributing",
        "## License",
    ];
    let positions: Vec<_> = required
        .iter()
        .map(|heading| {
            readme
                .find(heading)
                .unwrap_or_else(|| panic!("README must contain {heading}"))
        })
        .collect();

    assert!(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "README sections must remain in canonical order"
    );

    for safety_contract in [
        "scan --dry-run",
        "--chrome-root PATH",
        "--policy-dir PATH",
        "sudo",
        "pkexec",
        "XDG_CONFIG_HOME",
        "/etc/opt/chrome/policies/managed",
    ] {
        assert!(
            readme.contains(safety_contract),
            "README must preserve operational guidance: {safety_contract}"
        );
    }

    assert!(!readme.contains("\n## Quick Start\n"));
    assert!(!readme.contains("\n## Installation\n"));
    assert!(!readme.contains("\n## Project Structure\n"));
    assert!(!readme.contains("\n## Testing\n"));
}

#[test]
fn required_ci_remains_the_single_stable_merge_check() {
    let workflow = read_file(".github/workflows/verify.yml");

    assert!(workflow.contains("required-ci:"));
    assert!(workflow.contains("needs:\n      - commit-messages\n      - quality"));
    assert!(workflow.contains(r#"test "$COMMIT_RESULT" = "success""#));
    assert!(workflow.contains(r#"test "$QUALITY_RESULT" = "success""#));
    assert!(!workflow.contains("\n  documentation:\n"));
}

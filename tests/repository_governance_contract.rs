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
fn commitlint_is_pinned_and_requires_meaningful_commit_bodies() {
    let package = read_file("package.json");
    let config = read_file("commitlint.config.cjs");
    let title = read_file("commitlint.title.config.cjs");
    let hook = read_file(".husky/commit-msg");

    for required in [
        r#""@commitlint/cli": "21.2.2""#,
        r#""@commitlint/config-conventional": "21.2.2""#,
        r#""husky": "9.1.7""#,
    ] {
        assert!(
            package.contains(required),
            "development tooling must pin {required}"
        );
    }

    for required in [
        "'body-empty': [2, 'never']",
        "'body-min-length': [2, 'always', 20]",
        "'body-leading-blank': [2, 'always']",
        "'header-max-length': [2, 'always', 72]",
        "'body-max-line-length': [2, 'always', 100]",
    ] {
        assert!(
            config.contains(required),
            "commit profile must enforce {required}"
        );
    }

    assert!(title.contains("'body-empty': [0]"));
    assert!(title.contains("'body-min-length': [0]"));
    assert!(hook.contains("commitlint --edit"));
}

#[test]
fn pull_request_contract_uses_exact_standard_sections() {
    let template = read_file(".github/pull_request_template.md");
    let validator = read_file("scripts/validate_pr_description.py");

    let required = [
        "## Summary",
        "## Motivation",
        "## Changes",
        "## Verification",
        "## Risk and rollback",
        "## Release impact",
    ];
    let positions: Vec<_> = required
        .iter()
        .map(|heading| {
            template
                .find(heading)
                .unwrap_or_else(|| panic!("PR template must contain {heading}"))
        })
        .collect();

    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(template.contains("Release-Type: <none|alpha|beta|stable>"));
    assert!(template.contains("Release-Reason:"));

    for required in [
        r#""Summary""#,
        r#""Motivation""#,
        r#""Changes""#,
        r#""Verification""#,
        r#""Risk and rollback""#,
        r#""Release impact""#,
        r#"RELEASE_TYPES = {"none", "alpha", "beta", "stable"}"#,
    ] {
        assert!(
            validator.contains(required),
            "PR validator must encode {required}"
        );
    }
}

#[test]
fn trusted_pr_governance_validates_title_and_body() {
    let workflow = read_file(".github/workflows/pr-governance.yml");

    for required in [
        "name: Pull Request Governance",
        "name: PR Governance",
        "pull_request_target:",
        "Checkout trusted base revision",
        r#"ref: ${{ github.event.pull_request.base.sha }}"#,
        "commitlint.title.config.cjs",
        "scripts/validate_pr_description.py",
        "--body-file",
    ] {
        assert!(
            workflow.contains(required),
            "PR governance must enforce {required}"
        );
    }
}

#[test]
fn required_ci_aggregates_commit_governance_and_quality() {
    let workflow = read_file(".github/workflows/verify.yml");

    for required in [
        "commit-messages:",
        "name: Commit Message Governance",
        "fetch-depth: 0",
        "npm run test:commitlint",
        "commitlint.title.config.cjs",
        "scripts/validate_pr_description.py",
        "npm exec --no -- commitlint",
        "--from \"$BASE_SHA\"",
        "--to \"$HEAD_SHA\"",
        "name: Required CI",
        "needs:",
        "- commit-messages",
        "- quality",
    ] {
        assert!(
            workflow.contains(required),
            "Verify workflow must enforce {required}"
        );
    }

    assert!(workflow.contains(r#"test "$COMMIT_RESULT" = "success""#));
    assert!(workflow.contains(r#"test "$QUALITY_RESULT" = "success""#));
}

#[test]
fn release_tags_are_guarded_by_version_and_protected_master_identity() {
    let workflow = read_file(".github/workflows/release-guard.yml");

    for required in [
        "name: Release Tag Guard",
        "tags:",
        "- 'v*'",
        "cargo metadata --locked --no-deps --format-version 1",
        "refs/remotes/origin/master",
        "TAG_COMMIT",
        "MASTER_COMMIT",
        "v$PACKAGE_VERSION",
    ] {
        assert!(
            workflow.contains(required),
            "release guard must enforce {required}"
        );
    }
}

#[test]
fn contributing_documents_the_remote_authority_and_merge_method() {
    let contributing = read_file("CONTRIBUTING.md");

    for required in [
        "Conventional Commits",
        "required body",
        "Summary",
        "Motivation",
        "Changes",
        "Verification",
        "Risk and rollback",
        "Release impact",
        "Required CI",
        "PR Governance",
        "rebase",
        "Release Tag Guard",
    ] {
        assert!(
            contributing.contains(required),
            "contributing policy must document {required}"
        );
    }
}

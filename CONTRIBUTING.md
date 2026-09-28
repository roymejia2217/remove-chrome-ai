# Contributing to remove-chrome-ai

## Repository governance

Repository metadata is treated as executable acceptance input. Local hooks provide fast
feedback, while GitHub required checks remain the authoritative merge boundary.

The repository uses:

- [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/)
  through pinned Commitlint.
- An exact pull request body schema supplied by the checked-in GitHub template.
- Semantic Versioning from the version declared in `Cargo.toml`.
- Protected `master` with reviewed pull requests, required checks, and rebase-only merges.

## Commit messages

Commit messages use the Conventional Commits structure:

```text
type(scope): subject

A meaningful required body explaining the change.

Optional-Trailer: value
```

The body is required, must be separated from the header by a blank line, and must contain
at least 20 characters. The header is limited to 72 characters and body lines are limited
to 100 characters.

The configured type vocabulary is `build`, `chore`, `ci`, `docs`, `feat`, `fix`,
`perf`, `refactor`, `revert`, `style`, and `test`.

Supported scopes are `core`, `cli`, `config`, `scanner`, `cleaner`, `policy`,
`reporting`, `ci`, `deps`, `docs`, `governance`, and `release`. The scope remains
optional.

Install the pinned local governance tooling with:

```sh
npm ci
```

Husky installs the local `commit-msg` hook and Commitlint validates the message before the
commit is accepted locally. GitHub repeats the validation over the complete pull request
commit range.

## Pull requests

Pull request titles use the same Conventional Commit header grammar. The pull request body
must contain exactly these level-two sections, in this order:

```text
## Summary
## Motivation
## Changes
## Verification
## Risk and rollback
## Release impact
```

Unknown, duplicated, missing, or reordered level-two sections are rejected.

The `Release impact` section contains exactly:

```text
Release-Type: none|alpha|beta|stable
Release-Reason: a meaningful explanation of at least 20 characters
```

`PR Governance` validates the title and body from the trusted base revision. `Required CI`
aggregates commit-message governance with repository quality and README checks.

## Verification

Before opening a pull request, run:

```sh
npm run test:commitlint
npm run test:pr-governance
cargo fmt --all -- --check
cargo check --locked --all-targets
cargo test --locked --all-targets
```

Required remote checks remain authoritative.

## Merge policy

Protected `master` accepts changes only through pull requests. The repository preserves
linear history and allows rebase merge only. Force-pushes and deletion of the protected
branch are rejected. Review conversations must be resolved before merge.

## Release tags

Version tags use the exact `v<version>` value from `Cargo.toml`. The `Release Tag Guard`
requires the tagged commit to be identical to the current protected `master` commit.

Version tags are immutable once created: moving or rewriting an existing release tag is not
an approved release path. Tags and releases are created only after protected post-merge CI
has succeeded and the release version has been reviewed through the normal pull request
flow.

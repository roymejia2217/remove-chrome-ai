# remove-chrome-ai

[![Rust 2024](https://img.shields.io/badge/Rust-2024-orange?style=flat&logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Safely scans and removes Google Chrome on-device AI model directories and applies managed policies.

The tool is designed for Linux environments. It resolves the Chrome
configuration root, detects known on-device model directories, reports
potentially unsafe conditions, and can apply managed Chrome policies that
control generative AI features.

## Table of Contents

- [Install](#install)
- [Usage](#usage)
- [Features](#features)
- [Configuration](#configuration)
- [Safety](#safety)
- [Architecture](#architecture)
- [Development](#development)
- [Releases](#releases)
- [Contributing](#contributing)
- [License](#license)

## Install

### Prerequisites

- A stable Rust toolchain capable of building Rust 2024 edition projects.
- Google Chrome when scanning or managing an actual Chrome installation.

Clone the repository and build the locked dependency graph:

```sh
git clone https://github.com/roymejia2217/remove-chrome-ai.git
cd remove-chrome-ai
cargo build --release --locked
```

The compiled executable is:

```text
target/release/remove-chrome-ai
```

Before making changes to Chrome data or policy files, run a non-destructive
scan:

```sh
./target/release/remove-chrome-ai scan --dry-run
```

## Usage

```sh
./target/release/remove-chrome-ai [scan|clean|protect|repair|status|doctor] \
  [--dry-run] [--strict] [--json] \
  [--chrome-root PATH] [--policy-dir PATH]
```

Commands:

1. `scan` inspects known model directories without changing files.
2. `clean` removes detected model directories and applies the core policy.
3. `protect` applies the managed policy without removing model directories.
4. `repair` performs cleanup and policy application using the same workflow as
   `clean`.
5. `status` and `doctor` inspect the environment without cleanup or policy
   writes.

Options:

- `--dry-run` previews cleanup and policy changes.
- `--strict` adds the default and DevTools generative AI policy settings.
- `--json` emits machine-readable reports.
- `--chrome-root PATH` overrides the detected Chrome configuration root.
- `--policy-dir PATH` overrides the managed policy directory.

If no command is supplied, the application runs `clean` with the core policy
in apply mode.

Policy installation may require elevated privileges. If the initial write is
denied, the application looks for `sudo` and then `pkexec` to retry the
policy write.

## Features

| Feature | Description |
| --- | --- |
| **Model scanning** | Identifies known Chrome on-device AI model directories and reports their sizes. |
| **Safe cleanup** | Removes only exact, high-confidence model directories directly under the resolved Chrome configuration root. |
| **Dry-run mode** | Reports cleanup and policy changes without deleting files or writing policy data. |
| **Managed policies** | Writes core or strict Chrome policy settings and detects conflicting existing JSON policies. |
| **Privilege elevation** | Retries policy installation through `sudo` or `pkexec` when elevated permissions are required. |
| **Incident reporting** | Reports running Chrome processes, symlink candidates, malformed policies, conflicts, and blocking conditions. |
| **JSON output** | Emits machine-readable scan, cleanup, policy, and incident reports with `--json`. |

## Configuration

Command-line path options take precedence over environment-derived paths.

| Variable | Required | Description |
| --- | --- | --- |
| `HOME` | No | Supplies the home directory for the default Chrome root when `XDG_CONFIG_HOME` and `--chrome-root` are not set. |
| `XDG_CONFIG_HOME` | No | Sets the base directory for the default Chrome root, which becomes `$XDG_CONFIG_HOME/google-chrome`. |

By default, the application resolves Chrome configuration from
`XDG_CONFIG_HOME/google-chrome` or `HOME/.config/google-chrome`.

The default managed policy directory is:

```text
/etc/opt/chrome/policies/managed
```

## Safety

This tool can delete Chrome model directories and modify managed policy files.
Use `scan --dry-run` before applying changes, review the reported paths and
incidents, and use `--chrome-root PATH` or `--policy-dir PATH` only when the
target locations are known and intentional.

Cleanup is intentionally limited to exact, high-confidence model directory
names under the resolved Chrome configuration root. Symlink candidates and
policy conflicts are reported instead of being treated as ordinary removable
content.

## Architecture

The application is split into focused Rust modules:

- `src/chrome_detector.rs` resolves the Chrome configuration root.
- `src/model_scanner.rs` discovers and sizes known model directories.
- `src/cleaner.rs` implements guarded cleanup behavior.
- `src/policy_manager.rs` reads, validates, and writes managed policies.
- `src/incident_detector.rs` identifies blocking or suspicious conditions.
- `src/reporter.rs` renders human-readable and JSON output.
- `src/config.rs` resolves command-line and environment configuration.
- `src/cli.rs` coordinates commands, dry-run behavior, policy application,
  and privilege elevation.
- `src/lib.rs` exposes the reusable application modules.
- `src/main.rs` is the executable entry point.

Integration tests under `tests/` cover CLI behavior, configuration
resolution, policy handling, and scanner/cleaner safety.

## Development

Use the locked dependency graph for repository verification:

```sh
cargo fmt --all -- --check
cargo check --locked --all-targets
cargo test --locked --all-targets
```

The repository CI also enforces this README with Standard Readme,
remark-lint, and Lychee link validation.

## Releases

Version metadata is defined in `Cargo.toml`. Published versions, when
available, are listed under
[GitHub Releases](https://github.com/roymejia2217/remove-chrome-ai/releases).

## Contributing

Keep changes focused and preserve the cleanup and policy safety boundaries.
Before opening a pull request, run:

```sh
cargo fmt --all -- --check
cargo check --locked --all-targets
cargo test --locked --all-targets
```

Pull requests must pass the repository's `Required CI` check before merge.

## License

MIT © 2026 Roy Mejia. See [LICENSE](LICENSE).

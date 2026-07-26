<h1 align="center">remove-chrome-ai</h1>

<p align="center">
  <a href="https://www.rust-lang.org/">
    <img src="https://img.shields.io/badge/Rust-2024-orange?style=flat&logo=rust" alt="Rust 2024" />
  </a>
  <img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT" />
</p>

<p align="center">
  Safely scans and removes Google Chrome on-device AI model directories and applies managed policies.
</p>

---

## Quick Start

```bash
git clone https://github.com/roymejia2217/remove-chrome-ai.git
cd remove-chrome-ai
cargo build --release
```

Run a non-destructive scan before making changes:

```bash
./target/release/remove-chrome-ai scan --dry-run
```

---

## Features

| Feature | Description |
|---------|-------------|
| **Model scanning** | Identifies known Chrome on-device AI model directories and reports their sizes. |
| **Safe cleanup** | Removes only exact, high-confidence model directories directly under the resolved Chrome configuration root. |
| **Dry-run mode** | Reports cleanup and policy changes without deleting files or writing policy data. |
| **Managed policies** | Writes core or strict Chrome policy settings and detects conflicting existing JSON policies. |
| **Privilege elevation** | Retries policy installation through `sudo` or `pkexec` when the managed policy directory requires elevated permissions. |
| **Incident reporting** | Reports running Chrome processes, symlink candidates, malformed policies, conflicts, and other blocking conditions. |
| **JSON output** | Emits machine-readable scan, cleanup, policy, and incident reports with `--json`. |

---

## Prerequisites

| Dependency | Purpose | Installation |
|------------|---------|--------------|
| **Rust stable toolchain** | Builds and runs the command-line application. | [Install Rust](https://www.rust-lang.org/tools/install) |
| **Google Chrome** (optional) | Provides the Chrome configuration and policy targets inspected by the tool. | [Install Chrome](https://www.google.com/chrome/) |

**Note:** The application is designed for Linux environments. It resolves Chrome configuration from `XDG_CONFIG_HOME/google-chrome` or `HOME/.config/google-chrome` and manages policies under `/etc/opt/chrome/policies/managed` by default.

---

## Installation

```bash
cargo build --release
```

The compiled executable is `target/release/remove-chrome-ai`.

---

## Usage

```bash
./target/release/remove-chrome-ai [scan|clean|protect|repair|status|doctor] [--dry-run] [--strict] [--json] [--chrome-root PATH] [--policy-dir PATH]
```

1. Run `scan` to inspect known model directories without changing files.
2. Run `clean` to remove detected model directories and apply the core policy.
3. Run `protect` to apply the policy without removing model directories.
4. Run `repair` to perform cleanup and policy application using the same workflow as `clean`.
5. Run `status` or `doctor` to inspect the environment without cleanup or policy writes.
6. Add `--dry-run` to preview cleanup and policy changes.
7. Add `--strict` to include the default and DevTools generative AI policy settings.
8. Add `--json` for machine-readable output.

**Path overrides:**
- `--chrome-root PATH` overrides the detected Chrome configuration root.
- `--policy-dir PATH` overrides the managed policy directory.

**Default command:** If no command is supplied, the application runs `clean` with the core policy in apply mode.

**Policy permissions:** Policy installation may require root privileges. If the initial write is denied, the application looks for `sudo` and then `pkexec` to retry the policy write.

---

## Configuration

The command-line path options take precedence over environment-derived paths:

| Variable | Required | Description |
|----------|----------|-------------|
| `HOME` | No | Supplies the home directory used for the default Chrome root when `XDG_CONFIG_HOME` and `--chrome-root` are not set. |
| `XDG_CONFIG_HOME` | No | Sets the base directory for the default Chrome root, which becomes `$XDG_CONFIG_HOME/google-chrome`. |

---

## Project Structure

```
├── Cargo.toml
├── Cargo.lock
├── LICENSE
├── README.md
├── src
│   ├── chrome_detector.rs
│   ├── cleaner.rs
│   ├── cli.rs
│   ├── config.rs
│   ├── error.rs
│   ├── incident_detector.rs
│   ├── lib.rs
│   ├── main.rs
│   ├── model_scanner.rs
│   ├── policy_manager.rs
│   └── reporter.rs
└── tests
    ├── cli_tests.rs
    ├── config_tests.rs
    ├── policy_tests.rs
    └── scanner_cleaner_tests.rs
```

---

## Testing

```bash
cargo test
```

Test coverage includes:
- CLI policy writing and strict policy generation.
- Configuration resolution through XDG and home-directory fallbacks.
- Detection and sizing of known model directories.
- Symlink safety and idempotent cleanup behavior.
- Policy conflict detection and atomic policy writes.

---

## Credits

| Project | Description | License |
|---------|-------------|---------|
| [serde_json](https://crates.io/crates/serde_json) | Serializes policy and command reports as JSON. | MIT or Apache-2.0 |

---

## License

MIT License. See [LICENSE](LICENSE) for details.

**Disclaimer:** This tool modifies Google Chrome configuration files and managed policy files. Review `scan` or `--dry-run` output before applying changes, and use it at your own risk.

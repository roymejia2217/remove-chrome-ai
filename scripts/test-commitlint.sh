#!/usr/bin/env bash
set -euo pipefail

commitlint() {
  npm exec --no -- commitlint --verbose
}

assert_rejected() {
  local name="$1"
  local message="$2"

  if printf '%s
' "$message" | commitlint; then
    echo "Expected Commitlint to reject: $name" >&2
    exit 1
  fi
}

valid_message=$'feat(scanner): report model candidates\n\nDescribe detected Chrome AI model candidates without mutating files.'
valid_footer=$'fix(policy): preserve managed policy data\n\nKeep unrelated managed policy keys intact during updates.\n\nRefs: #1'

printf '%s
' "$valid_message" | commitlint
printf '%s
' "$valid_footer" | commitlint
printf '%s
' 'docs(governance): standardize README policy' |
  npm exec --no -- commitlint --config commitlint.title.config.cjs --verbose

assert_rejected 'missing body' 'feat(scanner): report model candidates'
assert_rejected 'missing type' $'Update scanner behavior\n\nDescribe the scanner behavior with enough detail.'
assert_rejected 'unknown type' $'unknown(scanner): report models\n\nDescribe the scanner behavior with enough detail.'
assert_rejected 'unknown scope' $'fix(browser): report models\n\nDescribe the scanner behavior with enough detail.'
assert_rejected 'uppercase subject' $'feat(scanner): Report models\n\nDescribe the scanner behavior with enough detail.'
assert_rejected 'subject with period' $'feat(scanner): report models.\n\nDescribe the scanner behavior with enough detail.'
assert_rejected 'body without leading blank' $'fix(policy): preserve data\nKeep unrelated managed policy keys intact during updates.'

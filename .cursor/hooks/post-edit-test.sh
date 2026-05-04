#!/bin/bash
# Post-edit test runner hook
# Triggered when the agent completes work (stop event)
# Runs relevant tests based on which files were modified

set -euo pipefail

input=$(cat)

# Get git-tracked modified files
changed_files=$(git diff --name-only HEAD 2>/dev/null || git diff --name-only 2>/dev/null || echo "")

if [ -z "$changed_files" ]; then
  echo '{"decision": "approve"}'
  exit 0
fi

test_failures=""
ran_tests=false

# Check if frontend files were modified
frontend_changed=$(echo "$changed_files" | grep -E '^src/.*\.(ts|tsx)$' || true)
if [ -n "$frontend_changed" ]; then
  ran_tests=true

  # Run TypeScript type check
  if ! npx tsc --noEmit 2>&1; then
    tsc_output=$(npx tsc --noEmit 2>&1 || true)
    test_failures="${test_failures}\n## TypeScript Errors:\n${tsc_output}"
  fi

  # Run vitest
  if ! npm run test 2>&1; then
    vitest_output=$(npm run test 2>&1 || true)
    test_failures="${test_failures}\n## Vitest Failures:\n${vitest_output}"
  fi
fi

# Check if Rust files were modified
rust_changed=$(echo "$changed_files" | grep -E '^src-tauri/.*\.rs$' || true)
if [ -n "$rust_changed" ]; then
  ran_tests=true

  # Run cargo check (faster than full build)
  if ! cargo check --manifest-path src-tauri/Cargo.toml 2>&1; then
    cargo_output=$(cargo check --manifest-path src-tauri/Cargo.toml 2>&1 || true)
    test_failures="${test_failures}\n## Cargo Check Errors:\n${cargo_output}"
  fi

  # Run cargo test
  if ! cargo test --manifest-path src-tauri/Cargo.toml 2>&1; then
    cargo_test_output=$(cargo test --manifest-path src-tauri/Cargo.toml 2>&1 || true)
    test_failures="${test_failures}\n## Cargo Test Failures:\n${cargo_test_output}"
  fi
fi

if [ "$ran_tests" = false ]; then
  echo '{"decision": "approve"}'
  exit 0
fi

if [ -n "$test_failures" ]; then
  # Truncate output to avoid overly long messages (keep first 2000 chars)
  truncated=$(echo -e "$test_failures" | head -c 2000)
  escaped=$(echo "$truncated" | sed 's/"/\\"/g' | sed ':a;N;$!ba;s/\n/\\n/g')
  cat <<EOF
{
  "decision": "continue",
  "followup_message": "Tests failed after your changes. Please fix the following issues:\\n${escaped}"
}
EOF
else
  echo '{"decision": "approve"}'
fi

exit 0

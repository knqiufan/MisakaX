#!/bin/bash
# Post-edit test runner hook for Cursor
# Triggered when the agent completes work (stop event)
# Runs relevant tests based on which files were modified

set -euo pipefail

input=$(cat)

# Prevent infinite loops
stop_hook_active=$(echo "$input" | jq -r '.stop_hook_active // false' 2>/dev/null || echo "false")
if [ "$stop_hook_active" = "true" ]; then
  echo '{"decision": "approve"}'
  exit 0
fi

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

  tsc_output=$(npx tsc --noEmit 2>&1 || true)
  tsc_exit=$?
  if [ $tsc_exit -ne 0 ] || echo "$tsc_output" | grep -q "error TS"; then
    test_failures="${test_failures}\n## TypeScript Errors:\n${tsc_output}"
  fi

  vitest_output=$(npm run test 2>&1 || true)
  vitest_exit=$?
  if [ $vitest_exit -ne 0 ]; then
    test_failures="${test_failures}\n## Vitest Failures:\n${vitest_output}"
  fi
fi

# Check if Rust files were modified
rust_changed=$(echo "$changed_files" | grep -E '^src-tauri/.*\.rs$' || true)
if [ -n "$rust_changed" ]; then
  ran_tests=true

  cargo_output=$(cargo check --manifest-path src-tauri/Cargo.toml 2>&1 || true)
  cargo_exit=$?
  if [ $cargo_exit -ne 0 ]; then
    test_failures="${test_failures}\n## Cargo Check Errors:\n${cargo_output}"
  fi

  cargo_test_output=$(cargo test --manifest-path src-tauri/Cargo.toml 2>&1 || true)
  cargo_test_exit=$?
  if [ $cargo_test_exit -ne 0 ]; then
    test_failures="${test_failures}\n## Cargo Test Failures:\n${cargo_test_output}"
  fi
fi

if [ "$ran_tests" = false ]; then
  echo '{"decision": "approve"}'
  exit 0
fi

if [ -n "$test_failures" ]; then
  # Truncate to avoid overly long messages
  truncated=$(echo -e "$test_failures" | head -c 2000)
  escaped=$(echo "$truncated" | sed 's/"/\\"/g' | sed ':a;N;$!ba;s/\n/\\n/g')
  cat <<EOF
{
  "decision": "continue",
  "followup_message": "Tests failed after your changes:\\n${escaped}\\nPlease fix these issues."
}
EOF
else
  echo '{"decision": "approve"}'
fi

exit 0

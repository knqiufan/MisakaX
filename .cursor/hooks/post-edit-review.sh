#!/bin/bash
# Post-edit code review hook
# Triggered when the agent completes work (stop event)
# Phase 1: Static checks (TypeScript/Rust)
# Phase 2: LLM-based deep code review via configurable third-party API

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

input=$(cat)

# Check if this is already a stop-hook continuation to prevent infinite loops
stop_hook_active=$(echo "$input" | jq -r '.stop_hook_active // false' 2>/dev/null || echo "false")
if [ "$stop_hook_active" = "true" ]; then
  echo '{"decision": "approve"}'
  exit 0
fi

# Get git-tracked modified files (staged + unstaged)
changed_files=$(git diff --name-only HEAD 2>/dev/null || git diff --name-only 2>/dev/null || echo "")

if [ -z "$changed_files" ]; then
  echo '{"decision": "approve"}'
  exit 0
fi

issues=""

# ============================================================
# Phase 1: Static Analysis
# ============================================================

# Check TypeScript/React files
ts_files=$(echo "$changed_files" | grep -E '\.(ts|tsx)$' || true)
if [ -n "$ts_files" ]; then
  for file in $ts_files; do
    [ -f "$file" ] || continue

    if grep -n 'console\.log' "$file" 2>/dev/null | grep -v '// eslint-disable' | grep -v '// keep' > /dev/null; then
      issues="${issues}\n- $file: contains console.log statements (consider removing or using proper logging)"
    fi

    if grep -n ': any\b' "$file" 2>/dev/null > /dev/null; then
      issues="${issues}\n- $file: uses 'any' type (consider using proper typing)"
    fi

    line_count=$(wc -l < "$file" 2>/dev/null || echo "0")
    if [ "$line_count" -gt 300 ]; then
      issues="${issues}\n- $file: file has ${line_count} lines (consider splitting into smaller modules)"
    fi
  done
fi

# Check Rust files
rs_files=$(echo "$changed_files" | grep -E '\.rs$' || true)
if [ -n "$rs_files" ]; then
  for file in $rs_files; do
    [ -f "$file" ] || continue

    if grep -n '\.unwrap()' "$file" 2>/dev/null | grep -v '#\[test\]' | grep -v '#\[cfg(test)\]' > /dev/null; then
      issues="${issues}\n- $file: contains .unwrap() calls (consider using proper error handling with ? or expect())"
    fi

    if grep -n 'todo!\|unimplemented!' "$file" 2>/dev/null > /dev/null; then
      issues="${issues}\n- $file: contains todo!/unimplemented! macros"
    fi

    line_count=$(wc -l < "$file" 2>/dev/null || echo "0")
    if [ "$line_count" -gt 300 ]; then
      issues="${issues}\n- $file: file has ${line_count} lines (consider splitting into smaller modules)"
    fi
  done
fi

# ============================================================
# Phase 2: LLM Deep Review
# ============================================================

llm_review_result=""
config_file="$SCRIPT_DIR/review-config.json"

if [ -f "$config_file" ]; then
  llm_enabled=$(jq -r '.llm_review.enabled // false' "$config_file" 2>/dev/null || echo "false")

  if [ "$llm_enabled" = "true" ]; then
    api_base_url=$(jq -r '.llm_review.api_base_url // ""' "$config_file")
    api_key=$(jq -r '.llm_review.api_key // ""' "$config_file")
    model=$(jq -r '.llm_review.model // "deepseek-chat"' "$config_file")
    max_tokens=$(jq -r '.llm_review.max_tokens // 4096' "$config_file")
    temperature=$(jq -r '.llm_review.temperature // 0.3' "$config_file")
    timeout_seconds=$(jq -r '.llm_review.timeout_seconds // 45' "$config_file")
    language=$(jq -r '.llm_review.language // "zh-CN"' "$config_file")

    # Allow env var override for API key (security best practice)
    if [ -z "$api_key" ] && [ -n "${REVIEW_LLM_API_KEY:-}" ]; then
      api_key="$REVIEW_LLM_API_KEY"
    fi

    if [ -n "$api_base_url" ] && [ -n "$api_key" ]; then
      # Collect diff content for LLM review (limit to 8000 chars to stay within token limits)
      diff_content=$(git diff HEAD 2>/dev/null || git diff 2>/dev/null || echo "")
      diff_content=$(echo "$diff_content" | head -c 8000)

      if [ -n "$diff_content" ]; then
        # Build the review prompt
        if [ "$language" = "zh-CN" ]; then
          system_prompt="你是一位资深代码审查员。请对以下代码变更进行审查，重点关注：\n1. 代码逻辑是否合理、是否闭环（没有遗漏的边界条件或错误处理）\n2. 代码思路是否清晰、是否易于维护\n3. 需求功能是否完整实现（是否有未完成的部分）\n4. 是否存在潜在的 Bug 或安全隐患\n5. 是否遵循了良好的设计原则\n\n请用中文简洁地列出发现的问题（如果有的话）。如果代码质量良好没有问题，回复 LGTM。"
        else
          system_prompt="You are a senior code reviewer. Review the following code changes, focusing on:\n1. Whether the logic is sound and complete (no missing edge cases or error handling)\n2. Whether the code is clear and maintainable\n3. Whether the feature/requirement is fully implemented\n4. Potential bugs or security issues\n5. Whether good design principles are followed\n\nList any issues found concisely. If the code quality is good, reply with LGTM."
        fi

        # Escape content for JSON
        escaped_diff=$(echo "$diff_content" | jq -Rs '.')
        escaped_system=$(printf '%s' "$system_prompt" | jq -Rs '.')

        request_body=$(cat <<JSONEOF
{
  "model": "$model",
  "messages": [
    {"role": "system", "content": $escaped_system},
    {"role": "user", "content": "以下是本次代码变更的 diff:\n\n" }
  ],
  "max_tokens": $max_tokens,
  "temperature": $temperature
}
JSONEOF
)
        # Inject diff into user message properly
        request_body=$(echo "$request_body" | jq --argjson diff "$escaped_diff" '
          .messages[1].content = "以下是本次代码变更的 diff:\n\n" + $diff
        ')

        # Call LLM API
        llm_response=$(curl -s --max-time "$timeout_seconds" \
          -H "Content-Type: application/json" \
          -H "Authorization: Bearer $api_key" \
          -d "$request_body" \
          "${api_base_url}/chat/completions" 2>/dev/null || echo "")

        if [ -n "$llm_response" ]; then
          llm_review_result=$(echo "$llm_response" | jq -r '.choices[0].message.content // ""' 2>/dev/null || echo "")
        fi
      fi
    fi
  fi
fi

# ============================================================
# Compose Final Output
# ============================================================

has_issues=false
final_message=""

if [ -n "$issues" ]; then
  has_issues=true
  final_message="## Static Analysis Issues\n${issues}\n"
fi

if [ -n "$llm_review_result" ] && [ "$llm_review_result" != "LGTM" ] && [ "$llm_review_result" != "null" ]; then
  has_issues=true
  final_message="${final_message}\n## LLM Code Review\n${llm_review_result}\n"
fi

if [ "$has_issues" = true ]; then
  escaped_message=$(echo -e "$final_message" | sed 's/"/\\"/g' | sed ':a;N;$!ba;s/\n/\\n/g')
  cat <<EOF
{
  "decision": "block",
  "reason": "Code review found issues that need attention:\\n${escaped_message}\\n\\nPlease review and fix these issues."
}
EOF
else
  echo '{"decision": "approve"}'
fi

exit 0

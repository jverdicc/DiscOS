#!/usr/bin/env bash
set -euo pipefail

duplicates="$({
  git ls-files \
    | tr '[:upper:]' '[:lower:]' \
    | sort \
    | uniq -d
} || true)"

if [[ -n "$duplicates" ]]; then
  echo "case-insensitive filename collision(s) detected:" >&2
  while IFS= read -r duplicate; do
    [[ -z "$duplicate" ]] && continue
    echo "- ${duplicate}" >&2
    git ls-files | awk -v lower_name="$duplicate" 'tolower($0) == lower_name { print "  - " $0 }' >&2
  done <<< "$duplicates"
  exit 1
fi

echo "no case-insensitive filename collisions detected"

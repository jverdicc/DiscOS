#!/usr/bin/env bash
set -euo pipefail

EXPECTED_THRESHOLD=95
TEST_EVIDENCE_SCRIPT="scripts/test_evidence.sh"
MAKEFILE="Makefile"
DOC="docs/TEST_EVIDENCE.md"

script_threshold="$(sed -n 's/^COVERAGE_THRESHOLD_LINES=\([0-9]\+\)$/\1/p' "$TEST_EVIDENCE_SCRIPT")"

if [[ -z "$script_threshold" ]]; then
  echo "failed to read COVERAGE_THRESHOLD_LINES from $TEST_EVIDENCE_SCRIPT" >&2
  exit 1
fi

if [[ "$script_threshold" != "$EXPECTED_THRESHOLD" ]]; then
  echo "coverage threshold in $TEST_EVIDENCE_SCRIPT is $script_threshold, expected $EXPECTED_THRESHOLD" >&2
  exit 1
fi

makefile_thresholds="$(sed -n 's/.*--fail-under-lines \([0-9]\+\).*/\1/p' "$MAKEFILE")"
if [[ -z "$makefile_thresholds" ]]; then
  echo "failed to read Makefile coverage thresholds" >&2
  exit 1
fi

while IFS= read -r threshold; do
  [[ -z "$threshold" ]] && continue
  if [[ "$threshold" != "$EXPECTED_THRESHOLD" ]]; then
    echo "Makefile coverage threshold mismatch: found $threshold, expected $EXPECTED_THRESHOLD" >&2
    exit 1
  fi
done <<< "$makefile_thresholds"

doc_thresholds="$(sed -n 's/.*line coverage \*\*>= \([0-9]\+\)%\*\*.*/\1/p' "$DOC")"
if [[ -z "$doc_thresholds" ]]; then
  echo "failed to read docs coverage thresholds in $DOC" >&2
  exit 1
fi

while IFS= read -r threshold; do
  [[ -z "$threshold" ]] && continue
  if [[ "$threshold" != "$EXPECTED_THRESHOLD" ]]; then
    echo "docs coverage threshold mismatch: found $threshold, expected $EXPECTED_THRESHOLD" >&2
    exit 1
  fi
done <<< "$doc_thresholds"

echo "coverage thresholds are in sync at ${EXPECTED_THRESHOLD}%"

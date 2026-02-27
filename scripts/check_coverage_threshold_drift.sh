#!/usr/bin/env bash
set -euo pipefail

EXPECTED_THRESHOLD=95
TEST_EVIDENCE_SCRIPT="scripts/test_evidence.sh"
MAKEFILE="Makefile"
DOC="docs/TEST_EVIDENCE.md"

script_threshold="$(grep -Eo 'COVERAGE_THRESHOLD_LINES=[0-9]+' "${TEST_EVIDENCE_SCRIPT}" | head -n1 | cut -d= -f2)"
if [[ -z "${script_threshold}" ]]; then
  echo "failed to read COVERAGE_THRESHOLD_LINES from ${TEST_EVIDENCE_SCRIPT}" >&2
  exit 1
fi
if [[ "${script_threshold}" != "${EXPECTED_THRESHOLD}" ]]; then
  echo "coverage threshold in ${TEST_EVIDENCE_SCRIPT} is ${script_threshold}, expected ${EXPECTED_THRESHOLD}" >&2
  exit 1
fi

makefile_thresholds="$(grep -Eo -- '--fail-under-lines[[:space:]]+[0-9]+' "${MAKEFILE}" | awk '{print $2}' | sort -u)"
if [[ -z "${makefile_thresholds}" ]]; then
  echo "failed to read Makefile coverage thresholds" >&2
  exit 1
fi
while IFS= read -r threshold; do
  [[ -z "${threshold}" ]] && continue
  if [[ "${threshold}" != "${EXPECTED_THRESHOLD}" ]]; then
    echo "Makefile coverage threshold mismatch: found ${threshold}, expected ${EXPECTED_THRESHOLD}" >&2
    exit 1
  fi
done <<< "${makefile_thresholds}"

doc_thresholds="$(grep -Eo 'line coverage \*\*>= [0-9]+%\*\*' "${DOC}" | grep -Eo '[0-9]+' | sort -u)"
if [[ -z "${doc_thresholds}" ]]; then
  echo "failed to read docs coverage thresholds in ${DOC}" >&2
  exit 1
fi
while IFS= read -r threshold; do
  [[ -z "${threshold}" ]] && continue
  if [[ "${threshold}" != "${EXPECTED_THRESHOLD}" ]]; then
    echo "docs coverage threshold mismatch: found ${threshold}, expected ${EXPECTED_THRESHOLD}" >&2
    exit 1
  fi
done <<< "${doc_thresholds}"

echo "coverage thresholds are in sync at ${EXPECTED_THRESHOLD}%"

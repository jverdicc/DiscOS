#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
LOCK_FILE="${SCRIPT_DIR}/evidenceos_vendor.lock"

if [[ ! -f "${LOCK_FILE}" ]]; then
  echo "[FAIL] Missing lock file: ${LOCK_FILE}" >&2
  exit 1
fi

# shellcheck disable=SC1090
source "${LOCK_FILE}"

upstream_repo="${EVIDENCEOS_REPO_OVERRIDE:-${EVIDENCEOS_REPO}}"
upstream_rev="${EVIDENCEOS_REV_OVERRIDE:-${EVIDENCEOS_REV}}"

if [[ -z "${upstream_repo}" || -z "${upstream_rev}" ]]; then
  echo "[FAIL] EVIDENCEOS_REPO and EVIDENCEOS_REV must be set." >&2
  exit 1
fi

# Commit pin
if [[ "${upstream_rev}" =~ ^[0-9a-fA-F]{7,40}$ ]]; then
  tmp="$(mktemp -d)"
  trap 'rm -rf "${tmp}"' EXIT

  git init -q "${tmp}"
  git -C "${tmp}" remote add origin "${upstream_repo}"

  if ! git -C "${tmp}" fetch --depth 1 origin "${upstream_rev}" >/dev/null 2>&1; then
    echo "[FAIL] Pin commit ${upstream_rev} not found in ${upstream_repo}." >&2
    exit 1
  fi

  echo "[OK] Found pin commit ${upstream_rev} in ${upstream_repo}."
  exit 0
fi

# Tag pin
if git ls-remote --exit-code --tags "${upstream_repo}" "refs/tags/${upstream_rev}" >/dev/null 2>&1; then
  echo "[OK] Found pin tag refs/tags/${upstream_rev} in ${upstream_repo}."
  exit 0
fi

echo "[FAIL] Pin tag ${upstream_rev} not found in ${upstream_repo}." >&2
exit 1

#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
LOCK_FILE="${SCRIPT_DIR}/evidenceos_vendor.lock"

if [[ ! -f "${LOCK_FILE}" ]]; then
  echo "[FAIL] Missing lock file: ${LOCK_FILE}" >&2
  exit 1
fi

external_repo="${EVIDENCEOS_REPO:-}"
external_rev="${EVIDENCEOS_REV:-}"
# shellcheck disable=SC1090
source "${LOCK_FILE}"

upstream_repo="${EVIDENCEOS_REPO}"
upstream_rev="${EVIDENCEOS_REV}"
if [[ -n "${external_repo}" ]]; then
  upstream_repo="${external_repo}"
fi
if [[ -n "${external_rev}" ]]; then
  upstream_rev="${external_rev}"
fi

if [[ -z "${upstream_repo}" || -z "${upstream_rev}" ]]; then
  echo "[FAIL] EVIDENCEOS_REPO and EVIDENCEOS_REV must be set." >&2
  exit 1
fi

set +e
ls_remote_output="$(git ls-remote --tags "${upstream_repo}" "refs/tags/${upstream_rev}" 2>&1)"
ls_remote_status=$?
set -e

if [[ ${ls_remote_status} -ne 0 ]]; then
  echo "[FAIL] Unable to query ${upstream_repo}: ${ls_remote_output}" >&2
  exit 1
fi

if [[ -z "${ls_remote_output}" ]]; then
  echo "[FAIL] Pin tag not found; create/push tag in EvidenceOS first." >&2
  exit 1
fi

echo "[OK] Found pin tag refs/tags/${upstream_rev} in ${upstream_repo}."

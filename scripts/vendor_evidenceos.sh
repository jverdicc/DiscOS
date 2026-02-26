#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
LOCK_FILE="${SCRIPT_DIR}/evidenceos_vendor.lock"

if [[ ! -f "${LOCK_FILE}" ]]; then
  echo "[FAIL] Missing lock file: ${LOCK_FILE}" >&2
  exit 1
fi

# shellcheck disable=SC1090
source "${LOCK_FILE}"

upstream_repo="${EVIDENCEOS_REPO}"
upstream_rev="${EVIDENCEOS_REV}"
if [[ -n "${EVIDENCEOS_REPO_OVERRIDE:-}" ]]; then
  upstream_repo="${EVIDENCEOS_REPO_OVERRIDE}"
fi
if [[ -n "${EVIDENCEOS_REV_OVERRIDE:-}" ]]; then
  upstream_rev="${EVIDENCEOS_REV_OVERRIDE}"
fi

if [[ -z "${upstream_repo}" || -z "${upstream_rev}" ]]; then
  echo "[FAIL] evidenceos_vendor.lock must define EVIDENCEOS_REPO and EVIDENCEOS_REV." >&2
  exit 1
fi

if [[ ${#EVIDENCEOS_VENDORED_CRATES[@]} -eq 0 ]]; then
  echo "[FAIL] EVIDENCEOS_VENDORED_CRATES is empty in ${LOCK_FILE}." >&2
  exit 1
fi

temp_dir=""
source_dir=""
cleanup() {
  if [[ -n "${temp_dir}" && -d "${temp_dir}" ]]; then
    rm -rf "${temp_dir}"
  fi
}
trap cleanup EXIT

if [[ -n "${EVIDENCEOS_SOURCE_DIR:-}" ]]; then
  source_dir="${EVIDENCEOS_SOURCE_DIR}"
  if [[ ! -d "${source_dir}/.git" ]]; then
    echo "[FAIL] EVIDENCEOS_SOURCE_DIR is not a git repository: ${source_dir}" >&2
    exit 1
  fi
  git -C "${source_dir}" fetch --tags --force origin >/dev/null 2>&1 || true
else
  temp_dir="$(mktemp -d)"
  source_dir="${temp_dir}/EvidenceOS"
  git clone --depth 1 --branch "${upstream_rev}" "${upstream_repo}" "${source_dir}"
fi

if ! git -C "${source_dir}" rev-parse --verify "refs/tags/${upstream_rev}" >/dev/null 2>&1; then
  if ! git -C "${source_dir}" fetch --depth 1 origin "refs/tags/${upstream_rev}:refs/tags/${upstream_rev}" >/dev/null 2>&1; then
    echo "[FAIL] Tag ${upstream_rev} not found in ${upstream_repo}." >&2
    exit 1
  fi
fi

git -C "${source_dir}" checkout -q "${upstream_rev}"

for crate in "${EVIDENCEOS_VENDORED_CRATES[@]}"; do
  src="${source_dir}/${crate}"
  dst="${REPO_ROOT}/${crate}"

  if [[ ! -d "${src}" ]]; then
    echo "[FAIL] Missing upstream crate path: ${crate}" >&2
    exit 1
  fi

  rm -rf "${dst}"
  mkdir -p "$(dirname -- "${dst}")"
  cp -a "${src}" "${dst}"
  find "${dst}" -type d -name .git -prune -exec rm -rf {} +
  echo "[OK] Vendored ${crate} from ${upstream_rev}."
done

echo "Vendored EvidenceOS crates refreshed from ${upstream_repo} @ ${upstream_rev}."

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

if ! command -v rsync >/dev/null 2>&1; then
  echo "[FAIL] rsync is required but not installed." >&2
  exit 1
fi

temp_dir="$(mktemp -d)"
cleanup() {
  if [[ -d "${temp_dir}" ]]; then
    rm -rf "${temp_dir}"
  fi
}
trap cleanup EXIT

source_dir="${temp_dir}/EvidenceOS"
git clone --depth 1 --branch "${upstream_rev}" "${upstream_repo}" "${source_dir}" >/dev/null

for crate in "${EVIDENCEOS_VENDORED_CRATES[@]}"; do
  src="${source_dir}/${crate}/"
  dst="${REPO_ROOT}/${crate}/"

  if [[ ! -d "${src}" ]]; then
    echo "[FAIL] Missing upstream crate path: ${crate}" >&2
    exit 1
  fi

  mkdir -p "${dst}"
  rsync -a --delete --exclude '.git/' --exclude 'target/' "${src}" "${dst}"
  echo "[OK] Vendored ${crate} from ${upstream_rev}."
done

"${SCRIPT_DIR}/check_vendor_drift.sh"

echo "Vendored EvidenceOS crates updated from ${upstream_repo} @ ${upstream_rev}."

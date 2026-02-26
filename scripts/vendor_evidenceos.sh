#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
LOCK_FILE="${SCRIPT_DIR}/evidenceos_vendor.lock"

if [[ ! -f "${LOCK_FILE}" ]]; then
  echo "[FAIL] Missing lock file: ${LOCK_FILE}" >&2
  exit 1
fi

update_lock_rev=""
if [[ "${1:-}" == "--update-lock" ]]; then
  if [[ -z "${2:-}" ]]; then
    echo "[FAIL] --update-lock requires a commit SHA argument." >&2
    exit 1
  fi
  update_lock_rev="${2}"
elif [[ $# -gt 0 ]]; then
  echo "Usage: $0 [--update-lock <newrev>]" >&2
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

if [[ -n "${update_lock_rev}" ]]; then
  upstream_rev="${update_lock_rev}"
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

temp_dir="$(mktemp -d)"
source_dir="${temp_dir}/EvidenceOS"
git clone "${upstream_repo}" "${source_dir}" >/dev/null 2>&1

git -C "${source_dir}" checkout -q "${upstream_rev}"

if ! command -v rsync >/dev/null 2>&1; then
  echo "[FAIL] rsync is required but not installed." >&2
  exit 1
fi

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

if [[ -n "${update_lock_rev}" ]]; then
  escaped_rev="$(printf '%s\n' "${update_lock_rev}" | sed -e 's/[\/&]/\\&/g')"
  sed -i "s/^EVIDENCEOS_REV=\".*\"/EVIDENCEOS_REV=\"${escaped_rev}\"/" "${LOCK_FILE}"
  echo "[OK] Updated scripts/evidenceos_vendor.lock to EVIDENCEOS_REV=${update_lock_rev}."
fi

echo "Changed files summary:"
git -C "${REPO_ROOT}" status --short -- "${EVIDENCEOS_VENDORED_CRATES[@]}" "${LOCK_FILE}" || true

echo "Vendored EvidenceOS crates refreshed from ${upstream_repo} @ ${upstream_rev}."

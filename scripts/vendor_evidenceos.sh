#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
LOCK_FILE="${SCRIPT_DIR}/evidenceos_vendor.lock"

# shellcheck disable=SC1090
source "${LOCK_FILE}"

upstream_repo="${EVIDENCEOS_REPO_OVERRIDE:-${EVIDENCEOS_REPO}}"
upstream_rev="${EVIDENCEOS_REV_OVERRIDE:-${EVIDENCEOS_REV}}"

if [[ -z "${upstream_repo}" || -z "${upstream_rev}" ]]; then
  echo "[FAIL] EVIDENCEOS_REPO and EVIDENCEOS_REV must be set." >&2
  exit 1
fi

"${SCRIPT_DIR}/validate_evidenceos_pin.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "${tmp}"' EXIT

echo "[INFO] Cloning ${upstream_repo} @ ${upstream_rev}"
git clone --quiet "${upstream_repo}" "${tmp}/EvidenceOS"
git -C "${tmp}/EvidenceOS" checkout --quiet "${upstream_rev}"

copy_dir() {
  local src="$1"
  local dst="$2"
  rm -rf "${dst}"
  mkdir -p "$(dirname "${dst}")"
  cp -a "${src}" "${dst}"
}

for crate in "${EVIDENCEOS_VENDORED_CRATES[@]}"; do
  if [[ ! -d "${tmp}/EvidenceOS/${crate}" ]]; then
    echo "[FAIL] Upstream ${upstream_rev} missing: ${crate}" >&2
    exit 1
  fi
  echo "[INFO] Vendoring ${crate}"
  copy_dir "${tmp}/EvidenceOS/${crate}" "${REPO_ROOT}/${crate}"
done

echo "[OK] Vendored EvidenceOS crates updated from ${upstream_repo}@${upstream_rev}"
echo "[OK] Run ./scripts/check_vendor_drift.sh to verify."

#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
LOCK_FILE="${SCRIPT_DIR}/evidenceos_vendor.lock"

if [[ ! -f "${LOCK_FILE}" ]]; then
  echo "[FAIL] Missing lock file: ${LOCK_FILE}" >&2
  exit 1
fi

# shellcheck disable=SC2153
external_repo="${EVIDENCEOS_REPO:-}"
external_rev="${EVIDENCEOS_REV:-}"
# shellcheck disable=SC1090
source "${LOCK_FILE}"

upstream_repo="${EVIDENCEOS_REPO}"
upstream_rev="${EVIDENCEOS_REV}"
if [[ -n "${EVIDENCEOS_REPO_OVERRIDE:-}" ]]; then
  upstream_repo="${EVIDENCEOS_REPO_OVERRIDE}"
elif [[ -n "${external_repo}" ]]; then
  upstream_repo="${external_repo}"
fi
if [[ -n "${EVIDENCEOS_REV_OVERRIDE:-}" ]]; then
  upstream_rev="${EVIDENCEOS_REV_OVERRIDE}"
elif [[ -n "${external_rev}" ]]; then
  upstream_rev="${external_rev}"
fi

if [[ -z "${upstream_repo}" || -z "${upstream_rev}" ]]; then
  echo "[FAIL] evidenceos_vendor.lock must define EVIDENCEOS_REPO and EVIDENCEOS_REV." >&2
  exit 1
fi

EVIDENCEOS_REPO="${upstream_repo}" EVIDENCEOS_REV="${upstream_rev}" "${SCRIPT_DIR}/validate_evidenceos_pin.sh"

upstream_git_dir=""
temp_dir=""
cleanup() {
  if [[ -n "${temp_dir}" && -d "${temp_dir}" ]]; then
    rm -rf "${temp_dir}"
  fi
}
trap cleanup EXIT

if [[ -n "${EVIDENCEOS_SOURCE_DIR:-}" ]]; then
  if [[ ! -d "${EVIDENCEOS_SOURCE_DIR}/.git" ]]; then
    echo "[FAIL] EVIDENCEOS_SOURCE_DIR is set but is not a git repository: ${EVIDENCEOS_SOURCE_DIR}" >&2
    exit 1
  fi
  upstream_git_dir="${EVIDENCEOS_SOURCE_DIR}"
else
  temp_dir="$(mktemp -d)"
  upstream_git_dir="${temp_dir}/EvidenceOS"
  git init -q "${upstream_git_dir}"
  git -C "${upstream_git_dir}" remote add origin "${upstream_repo}"
  git -C "${upstream_git_dir}" fetch --depth 1 origin "refs/tags/${upstream_rev}:refs/tags/${upstream_rev}" >/dev/null 2>&1 || {
    echo "[FAIL] Unable to fetch tag ${upstream_rev} from ${upstream_repo}." >&2
    exit 1
  }
fi

if ! git -C "${upstream_git_dir}" cat-file -e "${upstream_rev}^{commit}" 2>/dev/null; then
  echo "[FAIL] Tag/commit ${upstream_rev} not found in ${upstream_git_dir}." >&2
  exit 1
fi

if [[ ${#EVIDENCEOS_VENDORED_CRATES[@]} -eq 0 ]]; then
  echo "[FAIL] EVIDENCEOS_VENDORED_CRATES is empty in ${LOCK_FILE}." >&2
  exit 1
fi

status=0
for crate in "${EVIDENCEOS_VENDORED_CRATES[@]}"; do
  crate_ok=1
  if [[ ! -d "${REPO_ROOT}/${crate}" ]]; then
    echo "[FAIL] Missing vendored crate directory in DiscOS: ${crate}" >&2
    status=1
    crate_ok=0
    continue
  fi

  if ! git -C "${upstream_git_dir}" cat-file -e "${upstream_rev}:${crate}" 2>/dev/null; then
    echo "[FAIL] Upstream commit ${upstream_rev} does not contain ${crate}." >&2
    status=1
    crate_ok=0
    continue
  fi

  mapfile -t local_files < <(git -C "${REPO_ROOT}" ls-files -- "${crate}")
  mapfile -t upstream_files < <(git -C "${upstream_git_dir}" ls-tree -r --name-only "${upstream_rev}" -- "${crate}")

  if ! diff -u <(printf '%s\n' "${upstream_files[@]}") <(printf '%s\n' "${local_files[@]}") >/dev/null; then
    echo "[FAIL] File list drift detected for ${crate}." >&2
    diff -u <(printf '%s\n' "${upstream_files[@]}") <(printf '%s\n' "${local_files[@]}") >&2 || true
    status=1
    crate_ok=0
    continue
  fi

  for file_path in "${local_files[@]}"; do
    upstream_hash="$(git -C "${upstream_git_dir}" show "${upstream_rev}:${file_path}" | sha256sum | awk '{print $1}')"
    local_hash="$(sha256sum "${REPO_ROOT}/${file_path}" | awk '{print $1}')"
    if [[ "${upstream_hash}" != "${local_hash}" ]]; then
      echo "[FAIL] Content drift in ${file_path}." >&2
      status=1
      crate_ok=0
      break
    fi
  done

  if [[ ${crate_ok} -eq 1 ]]; then
    echo "[OK] ${crate} matches ${upstream_rev}."
  fi
done

if [[ ${status} -ne 0 ]]; then
  cat >&2 <<'MSG'

Vendored EvidenceOS crates have drifted from the pinned upstream tag/commit.
If this change is intentional, update scripts/evidenceos_vendor.lock and synchronize files mechanically.
MSG
  exit 1
fi

echo "All vendored EvidenceOS crates match pinned upstream ${upstream_rev}."

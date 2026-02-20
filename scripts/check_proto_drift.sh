#!/usr/bin/env bash
set -euo pipefail

matches="$(rg --glob '*.proto' --line-number --no-heading '^\s*package\s+evidenceos\.' . || true)"

if [[ -n "$matches" ]]; then
  echo "[FAIL] Local .proto files defining package evidenceos.* are not allowed in DiscOS." >&2
  echo "$matches" >&2
  cat >&2 <<'MSG'

Use the shared `evidenceos-protocol` crate from the EvidenceOS repository as the only source of truth.
If you need protocol changes, update EvidenceOS first and then bump the pinned dependency in DiscOS.
MSG
  exit 1
fi

echo "No local evidenceos.* package .proto definitions found."

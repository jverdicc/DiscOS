#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST_PATH="${1:-$ROOT_DIR/artifacts/forc10/FULL_ARTIFACT_MANIFEST.json}"
OUT_DIR="${2:-$ROOT_DIR/artifacts/forc10/external/full_bundle}"

if [[ ! -f "$MANIFEST_PATH" ]]; then
  echo "[forc10-fetch] manifest missing: $MANIFEST_PATH" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"

readarray -t manifest_values < <(python3 - "$MANIFEST_PATH" <<'PY'
import json, sys
from pathlib import Path
manifest = json.loads(Path(sys.argv[1]).read_text())
artifact = manifest["artifact"]
print(artifact["bundle_url"])
print(artifact["bundle_filename"])
print(artifact["sha256"])
PY
)

BUNDLE_URL="${manifest_values[0]}"
BUNDLE_FILE="$OUT_DIR/${manifest_values[1]}"
EXPECTED_SHA="${manifest_values[2]}"

if [[ -f "$BUNDLE_FILE" ]]; then
  echo "[forc10-fetch] using existing bundle: $BUNDLE_FILE"
else
  echo "[forc10-fetch] downloading $BUNDLE_URL"
  curl --fail --location --retry 3 --output "$BUNDLE_FILE" "$BUNDLE_URL"
fi

ACTUAL_SHA="$(sha256sum "$BUNDLE_FILE" | awk '{print $1}')"
if [[ "$ACTUAL_SHA" != "$EXPECTED_SHA" ]]; then
  echo "[forc10-fetch] checksum mismatch for $BUNDLE_FILE" >&2
  echo "[forc10-fetch] expected: $EXPECTED_SHA" >&2
  echo "[forc10-fetch] actual  : $ACTUAL_SHA" >&2
  exit 2
fi

echo "[forc10-fetch] sha256 verified: $ACTUAL_SHA"

echo "$BUNDLE_FILE"

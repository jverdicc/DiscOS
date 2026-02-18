#!/usr/bin/env bash
set -euo pipefail

PINNED_REV="${EVIDENCEOS_REV:-3f8b95a6615874d80526e447cb33ad0396b079f4}"
UPSTREAM_REPO="https://github.com/jverdicc/EvidenceOS.git"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

cargo metadata --format-version 1 > "$TMP_DIR/metadata.json"

PROTO_FROM_DEP="$(python - <<'PY' "$TMP_DIR/metadata.json"
import json
import pathlib
import sys

meta_path = pathlib.Path(sys.argv[1])
meta = json.loads(meta_path.read_text())
for pkg in meta.get('packages', []):
    if pkg.get('name') == 'evidenceos-protocol':
        manifest = pathlib.Path(pkg['manifest_path']).parent
        proto = manifest / 'proto' / 'evidenceos.proto'
        print(proto)
        break
else:
    raise SystemExit('evidenceos-protocol package not found in cargo metadata')
PY
)"

if [[ ! -f "$PROTO_FROM_DEP" ]]; then
  echo "expected dependency proto file not found: $PROTO_FROM_DEP" >&2
  exit 1
fi

git clone --quiet "$UPSTREAM_REPO" "$TMP_DIR/EvidenceOS"
git -C "$TMP_DIR/EvidenceOS" checkout --quiet "$PINNED_REV"
UPSTREAM_PROTO="$TMP_DIR/EvidenceOS/crates/evidenceos-protocol/proto/evidenceos.proto"

if [[ ! -f "$UPSTREAM_PROTO" ]]; then
  echo "upstream proto file not found at expected path: $UPSTREAM_PROTO" >&2
  exit 1
fi

diff -u "$UPSTREAM_PROTO" "$PROTO_FROM_DEP"

echo "evidenceos.proto is synchronized with EvidenceOS@$PINNED_REV"

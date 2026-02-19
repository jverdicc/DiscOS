#!/usr/bin/env bash
set -euo pipefail

PINNED_REV="${EVIDENCEOS_REV:-3f8b95a6615874d80526e447cb33ad0396b079f4}"
UPSTREAM_REPO="${EVIDENCEOS_REPO:-https://github.com/EvidenceOS/evidenceos.git}"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

LOCAL_DIR="crates/evidenceos-protocol"
if [[ ! -d "$LOCAL_DIR" ]]; then
  echo "missing local protocol directory: $LOCAL_DIR" >&2
  exit 1
fi

git clone --quiet "$UPSTREAM_REPO" "$TMP_DIR/EvidenceOS"
git -C "$TMP_DIR/EvidenceOS" checkout --quiet "$PINNED_REV"
UPSTREAM_DIR="$TMP_DIR/EvidenceOS/crates/evidenceos-protocol"

if [[ ! -d "$UPSTREAM_DIR" ]]; then
  echo "upstream protocol directory not found: $UPSTREAM_DIR" >&2
  exit 1
fi

DIFF_FILE="$TMP_DIR/protocol.diff"
if ! diff -ru --exclude target "$UPSTREAM_DIR" "$LOCAL_DIR" > "$DIFF_FILE"; then
  echo "[FAIL] crates/evidenceos-protocol is out of sync with EvidenceOS@$PINNED_REV" >&2
  echo "--- protocol diff begin ---" >&2
  cat "$DIFF_FILE" >&2
  echo "--- protocol diff end ---" >&2
  cat >&2 <<MSG

Remediation:
  1) git clone "$UPSTREAM_REPO" /tmp/EvidenceOS && git -C /tmp/EvidenceOS checkout "$PINNED_REV"
  2) rsync -a --delete /tmp/EvidenceOS/crates/evidenceos-protocol/ crates/evidenceos-protocol/
  3) cargo build --workspace
  4) re-run ./scripts/check_evidenceos_proto_sync.sh
MSG
  exit 1
fi

echo "crates/evidenceos-protocol is synchronized with EvidenceOS@$PINNED_REV"

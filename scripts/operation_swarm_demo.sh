#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

mkdir -p artifacts/system-test

cargo test -p discos-client --test operation_swarm_system -- --nocapture

echo ""
echo "Operation swarm artifact:"
cat artifacts/system-test/operation_swarm_results.json

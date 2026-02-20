#!/usr/bin/env bash
set -euo pipefail

python - <<'PY'
from pathlib import Path
import re
import sys

ROOT = Path('.')
SCAN_DIRS = [ROOT / 'crates', ROOT / 'tests', ROOT / 'scripts']

violations = []

# 1) verify_signed_oracle_record must use real ed25519 verify when present.
for path in SCAN_DIRS:
    if not path.exists():
        continue
    for file in path.rglob('*.rs'):
        text = file.read_text(encoding='utf-8')
        lines = text.splitlines()
        for idx, line in enumerate(lines):
            if 'verify_signed_oracle_record' in line:
                window = '\n'.join(lines[idx: idx + 40])
                has_verify_call = '.verify(' in window
                has_ed25519 = 'ed25519' in window.lower()
                if not (has_verify_call and has_ed25519):
                    violations.append(
                        f"{file}:{idx+1}: verify_signed_oracle_record must use ed25519 verify()"
                    )

# 2) Forbid '* 0.0' shortcuts in accounting/runtime code.
mul_zero_pattern = re.compile(r'\*\s*0\.0\b')
for path in SCAN_DIRS:
    if not path.exists():
        continue
    for file in path.rglob('*.rs'):
        rel = file.as_posix()
        if '/tests/' in rel or rel.startswith('tests/'):
            continue
        for line_no, line in enumerate(file.read_text(encoding='utf-8').splitlines(), start=1):
            if mul_zero_pattern.search(line):
                violations.append(f"{file}:{line_no}: forbidden DP/accounting shortcut '* 0.0'")

# 3) Forbid derive_holdout_labels unless explicitly insecure-flag guarded.
for path in SCAN_DIRS:
    if not path.exists():
        continue
    for file in path.rglob('*.rs'):
        text = file.read_text(encoding='utf-8')
        lines = text.splitlines()
        for idx, line in enumerate(lines):
            if 'derive_holdout_labels' in line:
                window = '\n'.join(lines[max(0, idx - 20): idx + 20])
                if '--insecure-synthetic-holdout' not in window:
                    violations.append(
                        f"{file}:{idx+1}: derive_holdout_labels must be guarded by --insecure-synthetic-holdout"
                    )

if violations:
    print('implementation honesty gate failed:', file=sys.stderr)
    for v in violations:
        print(f' - {v}', file=sys.stderr)
    sys.exit(1)

print('implementation honesty gate passed')
PY

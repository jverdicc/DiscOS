.PHONY: fmt lint test coverage-core coverage-client check-coverage-threshold-drift test-evidence demo-exfil-baseline demo-exfil-evidenceos-mock

fmt:
	cargo fmt --all --check

lint:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

coverage-core:
	cargo llvm-cov --package discos-core --all-features --summary-only --fail-under-lines 95

coverage-client:
	cargo llvm-cov --package discos-client --summary-only --fail-under-lines 95

check-coverage-threshold-drift:
	./scripts/check_coverage_threshold_drift.sh

test-evidence: check-coverage-threshold-drift
	./scripts/test_evidence.sh

demo-exfil-baseline:
	python3 examples/exfiltration_demo/attack_bitflip.py --mode baseline --n 64 --seed 7

demo-exfil-evidenceos-mock:
	python3 examples/exfiltration_demo/attack_bitflip.py --mode evidenceos-mock --n 64 --seed 7 --quant-step 0.05 --hysteresis 0.03 --budget 48

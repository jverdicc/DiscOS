.PHONY: fmt lint test coverage-core coverage-client check-coverage-threshold-drift test-evidence

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

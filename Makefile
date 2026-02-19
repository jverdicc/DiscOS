.PHONY: fmt lint test coverage-core coverage-client test-evidence

fmt:
	cargo fmt --all --check

lint:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

coverage-core:
	cargo llvm-cov --package discos-core --all-features --summary-only --fail-under-lines 90

coverage-client:
	cargo llvm-cov --package discos-client --summary-only --fail-under-lines 80

test-evidence:
	./scripts/test_evidence.sh

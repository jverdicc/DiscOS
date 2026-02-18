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
	mkdir -p artifacts
	bash -o pipefail -c "cargo test --workspace 2>&1 | tee artifacts/test.log"
	cargo llvm-cov --workspace --lcov --output-path artifacts/coverage.lcov
	cargo llvm-cov report --lcov --summary-only > artifacts/coverage-summary.txt
	$(MAKE) coverage-core
	$(MAKE) coverage-client

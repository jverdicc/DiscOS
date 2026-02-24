.PHONY: setup reproduce figures verify verify-quick verify-full full-fetch clean

OUT_DIR ?= generated
GOLDEN_DIR ?= golden
MODE ?= quick
MANIFEST ?= FULL_ARTIFACT_MANIFEST.json
FULL_BUNDLE_DIR ?= external/full_bundle
EVIDENCEOS_REPO ?= ../EvidenceOS

setup:
	python3 -m pip install --requirement ../../paper_artifacts/requirements.txt
	cargo build --workspace --locked

REPRO_MODE ?= quick

reproduce:
	python3 scripts/reproduce.py --mode $(REPRO_MODE) --out $(OUT_DIR)

figures:
	python3 scripts/figures.py --out $(OUT_DIR)

verify-quick: REPRO_MODE=quick
verify-quick: reproduce figures
	@echo "[FORC10] mode=QUICK (CI-friendly, no network)"
	@echo "[FORC10] reproduced=legacy deterministic DiscOS harness outputs (results + figures subset)"
	python3 scripts/verify.py --actual $(OUT_DIR) --golden $(GOLDEN_DIR)

full-fetch:
	@echo "[FORC10] mode=FULL (paper-faithful)"
	@echo "[FORC10] fetching authoritative bundle + checksum verification"
	../../scripts/fetch_forc10_artifacts.sh $(MANIFEST) $(FULL_BUNDLE_DIR)

verify-full: REPRO_MODE=full
verify-full: full-fetch
	@echo "[FORC10] reproduced=authoritative Python paper artifact pipeline from EvidenceOS"
	python3 ../../paper_artifacts/reproduce_paper.py --evidenceos-repo $(EVIDENCEOS_REPO) -- --verify

verify:
ifeq ($(MODE),full)
	@$(MAKE) verify-full
else
	@$(MAKE) verify-quick
endif

clean:
	rm -rf $(OUT_DIR)

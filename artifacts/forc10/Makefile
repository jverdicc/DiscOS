.PHONY: setup reproduce figures verify clean

OUT_DIR ?= generated
GOLDEN_DIR ?= golden

setup:
	python3 -m pip install --requirement ../../paper_artifacts/requirements.txt
	cargo build --workspace --locked

reproduce:
	python3 scripts/reproduce.py --out $(OUT_DIR)

figures:
	python3 scripts/figures.py --out $(OUT_DIR)

verify: reproduce figures
	python3 scripts/verify.py --actual $(OUT_DIR) --golden $(GOLDEN_DIR)

clean:
	rm -rf $(OUT_DIR)

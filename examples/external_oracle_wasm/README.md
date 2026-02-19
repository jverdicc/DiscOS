# External Oracle WASM Example

This example shows how to submit a DiscOS claim against an EvidenceOS deployment that has a pluggable external oracle bundle loaded.

## 1) Package and load oracle bundle in EvidenceOS

Use EvidenceOS tooling to build a restricted oracle wasm and accompanying manifest bundle, then register it with the daemon under an oracle identifier (example: `acme.safety.v1`).

> Exact daemon-side commands depend on your EvidenceOS deployment and release. The key requirement is that the daemon advertises and accepts the same `oracle_id` string you pass from DiscOS.

## 2) Create and execute claim from DiscOS with `--oracle-id`

```bash
# Create a claim selecting external oracle id
cargo run -p discos-cli -- \
  --endpoint http://127.0.0.1:50051 \
  claim create \
  --claim-name ext-oracle-demo \
  --alpha-micros 10 \
  --lane lane-a \
  --epoch-config-ref epoch.v1 \
  --holdout-ref holdout.v1 \
  --epoch-size 64 \
  --oracle-num-symbols 8 \
  --access-credit 1000 \
  --oracle-id acme.safety.v1
```

Then continue with the normal `claim commit`, `claim freeze`, `claim seal`, and `claim execute` flow.

## 3) Fetch capsule and inspect oracle metadata

```bash
cargo run -p discos-cli -- \
  --endpoint http://127.0.0.1:50051 \
  claim fetch-capsule \
  --claim-id <claim_id_hex> \
  --print-capsule-json
```

DiscOS prints policy/oracle receipt metadata and preserves compatibility with older capsules that omit new fields.

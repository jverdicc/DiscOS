# TopicID v1 Canonical Specification

This document defines the canonical TopicID v1 algorithm intended to be shared identically between DiscOS and EvidenceOS.

## Domain separation

- Domain string (UTF-8 bytes): `evidenceos/topicid/v1`
- Hash function: `sha256_domain(domain_bytes, payload_bytes)` where:
  - `sha256_domain(a, b) = SHA256(a || 0x00 || b)`
  - `||` means byte concatenation.

## Field set and order

TopicID payload fields must be encoded in the exact order below:

1. `lane` (string)
2. `alpha_micros` (`uint32`)
3. `epoch_config_ref` (string)
4. `output_schema_id` (string)
5. `epoch_size` (`uint32`)
6. `semantic_hash` (optional 32-byte hash)
7. `phys_hir_signature_hash` (required 32-byte hash)
8. `dependency_merkle_root` (optional 32-byte hash)

These fields must align with claim creation inputs.

## Encoding rules

### Strings

Strings are encoded as:

- 4-byte unsigned length prefix in big-endian (`u32::to_be_bytes(len)`)
- Raw UTF-8 bytes of the string

### Numeric fields

- `alpha_micros`: 4-byte unsigned big-endian
- `epoch_size`: 4-byte unsigned big-endian

### Optional 32-byte hashes

Optional hash fields are encoded as:

- `0x00` when absent
- `0x01` followed by exactly 32 bytes when present

### Required 32-byte hash

- `phys_hir_signature_hash` is encoded as exactly 32 bytes with no prefix.

## Hash rule

Compute `topic_id` as:

- `payload = encode(fields_in_order)`
- `topic_id = SHA256(domain_bytes || 0x00 || payload)`

The output is a 32-byte digest (commonly represented as lowercase hex).

## Compatibility note

Legacy implementations may have used domain `evidenceos/topic-id/v1` and omitted `epoch_size`; those values are not TopicID v1.

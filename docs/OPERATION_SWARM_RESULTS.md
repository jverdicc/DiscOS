# Operation Swarm System Test Results Artifact

This document describes the generated artifact for the SAFE multi-client system test in `tests/operation_swarm_system.rs`.

## Artifact path

- `artifacts/system-test/operation_swarm_results.json`

## Schema

The artifact records deterministic run summary fields:

- `n_clients`: number of concurrent identities in the swarm.
- `n_total_attempts`: total execute attempts observed by the daemon.
- `first_escalation_at_attempt`: first attempt index where lane escalates above base assurance (or `null` if no escalation occurred before terminal reject).
- `final_state`: terminal operation state (`REJECT` or `FROZEN`).
- `final_k_remaining`: final remaining shared topic budget bits.

## Notes

- The scenario is defensive and does **not** implement or rely on any authentication bypass behavior.
- Identity labels are modeled through request metadata only to verify budget sharing behavior at the topic/operation level.
- The test uses a fixed RNG seed for deterministic and CI-stable outcomes.

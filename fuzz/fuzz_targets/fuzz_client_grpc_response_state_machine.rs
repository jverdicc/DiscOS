#![no_main]

use discos_client::{pb, verify_capsule_response, SignedTreeHead};
use libfuzzer_sys::fuzz_target;
use prost::Message;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClientPhase {
    Init,
    ArtifactsCommitted,
    GatesFrozen,
    SealedVault,
    Executed,
}

#[derive(Debug)]
struct ClientStateMachine {
    phase: ClientPhase,
    sealed_handshake_completed: bool,
}

impl ClientStateMachine {
    fn new() -> Self {
        Self {
            phase: ClientPhase::Init,
            sealed_handshake_completed: false,
        }
    }

    fn apply_commit_artifacts(&mut self, accepted: bool) -> bool {
        if self.phase != ClientPhase::Init || !accepted {
            return false;
        }
        self.phase = ClientPhase::ArtifactsCommitted;
        true
    }

    fn apply_freeze_gates(&mut self, frozen: bool) -> bool {
        if self.phase != ClientPhase::ArtifactsCommitted || !frozen {
            return false;
        }
        self.phase = ClientPhase::GatesFrozen;
        true
    }

    fn apply_seal_claim(&mut self, sealed: bool) -> bool {
        if self.phase != ClientPhase::GatesFrozen || !sealed {
            return false;
        }
        self.phase = ClientPhase::SealedVault;
        self.sealed_handshake_completed = true;
        true
    }

    fn apply_execute_claim(&mut self, certified: bool) -> bool {
        if self.phase != ClientPhase::SealedVault || !certified {
            return false;
        }
        self.phase = ClientPhase::Executed;
        true
    }
}

fn decode_grpc_frame<M: Message + Default>(frame: &[u8]) -> Option<M> {
    if frame.len() < 5 {
        return None;
    }

    let compressed_flag = frame[0];
    if compressed_flag != 0 {
        return None;
    }

    let declared_len = u32::from_be_bytes([frame[1], frame[2], frame[3], frame[4]]) as usize;
    if frame.len() != 5 + declared_len {
        return None;
    }

    M::decode(&frame[5..]).ok()
}

fn split_input(data: &[u8], sections: usize) -> Vec<&[u8]> {
    if sections == 0 {
        return vec![];
    }
    let mut out = Vec::with_capacity(sections);
    let mut cursor = 0usize;
    for i in 0..sections {
        let remaining_sections = sections - i;
        let remaining_bytes = data.len().saturating_sub(cursor);
        let take = remaining_bytes / remaining_sections;
        let end = cursor + take;
        out.push(&data[cursor..end]);
        cursor = end;
    }
    out
}

fuzz_target!(|data: &[u8]| {
    let chunks = split_input(data, 6);
    let mut sm = ClientStateMachine::new();

    if let Some(resp) = decode_grpc_frame::<pb::CommitArtifactsResponse>(chunks[0]) {
        let prev = sm.phase;
        let transitioned = sm.apply_commit_artifacts(resp.accepted);
        if !transitioned {
            assert_eq!(sm.phase, prev, "invalid commit transition mutated state");
        }
    }

    if let Some(resp) = decode_grpc_frame::<pb::FreezeGatesResponse>(chunks[1]) {
        let prev = sm.phase;
        let transitioned = sm.apply_freeze_gates(resp.frozen);
        if !transitioned {
            assert_eq!(sm.phase, prev, "invalid freeze transition mutated state");
        }
    }

    if let Some(resp) = decode_grpc_frame::<pb::SealClaimResponse>(chunks[2]) {
        let prev = sm.phase;
        let transitioned = sm.apply_seal_claim(resp.sealed);
        if !transitioned {
            assert_eq!(sm.phase, prev, "invalid seal transition mutated state");
        }
    }

    // Fuzz malformed capsule responses through client-side verifier.
    if let Some(resp) = decode_grpc_frame::<pb::FetchCapsuleResponse>(chunks[3]) {
        let mut expected_claim = [0u8; 32];
        let mut expected_topic = [0u8; 32];
        let mut kernel_pubkey = [0u8; 32];

        let seed = chunks[4];
        for (i, b) in seed.iter().copied().enumerate() {
            expected_claim[i % 32] ^= b;
            expected_topic[(i * 7) % 32] ^= b.rotate_left((i % 8) as u32);
            kernel_pubkey[(i * 13) % 32] ^= b.rotate_right((i % 8) as u32);
        }

        let previous_sth = SignedTreeHead {
            tree_size: 1,
            root_hash: [0u8; 32],
            signature: [0u8; 64],
        };

        let _ = verify_capsule_response(
            &resp,
            chunks[5],
            &expected_claim,
            &expected_topic,
            &kernel_pubkey,
            Some(&previous_sth),
        );
    }

    if let Some(resp) = decode_grpc_frame::<pb::ExecuteClaimResponse>(chunks[4]) {
        let prev = sm.phase;
        let transitioned = sm.apply_execute_claim(resp.certified);
        if !transitioned {
            assert_eq!(sm.phase, prev, "invalid execute transition mutated state");
        }
    }

    // Sealed Vault handshake invariant: execution can only happen after a valid seal.
    if sm.phase == ClientPhase::Executed {
        assert!(sm.sealed_handshake_completed, "execute bypassed Sealed Vault handshake");
    }
});

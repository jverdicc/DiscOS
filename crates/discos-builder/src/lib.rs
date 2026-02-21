// Copyright 2026 Joseph Verdicchio
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use evidenceos_core::{guest_abi, manifest};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wasm_encoder::{
    CodeSection, DataSection, ExportKind, ExportSection, Function, FunctionSection, ImportSection,
    Instruction, MemorySection, MemoryType, Module, TypeSection, ValType,
};

const DOMAIN_WASM_HASH: &[u8] = b"evidenceos/wasm-code-hash/v1";
const DOMAIN_MANIFEST_HASH: &[u8] = b"evidenceos/manifest-hash/v1";
const PAYLOAD: &[u8] = &[0x01, 0x02, 0x03];

pub const VAULT_IMPORT_MODULE: &str = guest_abi::VAULT_IMPORT_MODULE;
pub const VAULT_IMPORT_ORACLE_QUERY: &str = guest_abi::VAULT_IMPORT_ORACLE_QUERY;
pub const VAULT_IMPORT_EMIT_STRUCTURED_CLAIM: &str = guest_abi::VAULT_IMPORT_EMIT_STRUCTURED_CLAIM;
pub const VAULT_IMPORT_GET_LOGICAL_EPOCH: &str = guest_abi::VAULT_IMPORT_GET_LOGICAL_EPOCH;
pub const GUEST_EXPORT_RUN: &str = guest_abi::GUEST_EXPORT_RUN;
pub const GUEST_EXPORT_MEMORY: &str = guest_abi::GUEST_EXPORT_MEMORY;

pub use manifest::canonical_json_string as canonical_json;

pub fn sha256(input: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().into()
}

fn hash32(domain: &[u8], value: &[u8]) -> [u8; 32] {
    let mut material = Vec::with_capacity(domain.len() + 1 + value.len());
    material.extend_from_slice(domain);
    material.push(0);
    material.extend_from_slice(value);
    sha256(&material)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WasmBuildOutput {
    pub wasm_bytes: Vec<u8>,
    pub code_hash: [u8; 32],
}

pub fn build_restricted_wasm() -> WasmBuildOutput {
    build_restricted_wasm_with_payload(PAYLOAD)
}

pub fn build_restricted_wasm_with_payload(payload: &[u8]) -> WasmBuildOutput {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    types
        .ty()
        .function(vec![ValType::I32, ValType::I32], vec![ValType::I32]);
    types
        .ty()
        .function(vec![ValType::I32, ValType::I32], vec![]);
    types.ty().function(vec![], vec![ValType::I64]);
    types.ty().function(vec![], vec![]);
    module.section(&types);

    let mut imports = ImportSection::new();
    imports.import(
        VAULT_IMPORT_MODULE,
        VAULT_IMPORT_ORACLE_QUERY,
        wasm_encoder::EntityType::Function(0),
    );
    imports.import(
        VAULT_IMPORT_MODULE,
        VAULT_IMPORT_EMIT_STRUCTURED_CLAIM,
        wasm_encoder::EntityType::Function(1),
    );
    imports.import(
        VAULT_IMPORT_MODULE,
        VAULT_IMPORT_GET_LOGICAL_EPOCH,
        wasm_encoder::EntityType::Function(2),
    );
    module.section(&imports);

    let mut funcs = FunctionSection::new();
    funcs.function(3u32);
    module.section(&funcs);

    let mut memories = MemorySection::new();
    memories.memory(MemoryType {
        minimum: 1,
        maximum: None,
        memory64: false,
        shared: false,
        page_size_log2: None,
    });
    module.section(&memories);

    let mut exports = ExportSection::new();
    exports.export(GUEST_EXPORT_RUN, ExportKind::Func, 3);
    exports.export(GUEST_EXPORT_MEMORY, ExportKind::Memory, 0);
    module.section(&exports);

    let mut data = DataSection::new();
    data.active(
        0,
        &wasm_encoder::ConstExpr::i32_const(0),
        payload.iter().copied(),
    );
    module.section(&data);

    let mut code = CodeSection::new();
    let mut run = Function::new(vec![]);
    run.instruction(&Instruction::I32Const(0));
    run.instruction(&Instruction::I32Const(payload.len() as i32));
    run.instruction(&Instruction::Call(0));
    run.instruction(&Instruction::Drop);
    run.instruction(&Instruction::I32Const(0));
    run.instruction(&Instruction::I32Const(payload.len() as i32));
    run.instruction(&Instruction::Call(1));
    run.instruction(&Instruction::Call(2));
    run.instruction(&Instruction::Drop);
    run.instruction(&Instruction::End);
    code.function(&run);
    module.section(&code);

    let wasm_bytes = module.finish();
    let code_hash = hash32(DOMAIN_WASM_HASH, &wasm_bytes);
    WasmBuildOutput {
        wasm_bytes,
        code_hash,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlphaHIRManifest {
    pub plan_id: String,
    pub code_hash_hex: String,
    pub oracle_kinds: Vec<String>,
    pub output_schema_id: String,
    pub nullspec_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PhysHIRManifest {
    pub physical_signature_hash: String,
    pub envelope_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CausalDSLManifest {
    pub dag_hash: String,
    pub adjustment_sets: Vec<Vec<String>>,
}

pub fn manifest_hash<T: Serialize>(value: &T) -> Result<[u8; 32], serde_json::Error> {
    let canonical = manifest::canonical_json_bytes(value)?;
    Ok(hash32(DOMAIN_MANIFEST_HASH, &canonical))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use wasmparser::{ExternalKind, Parser, Payload, TypeRef};

    #[test]
    fn wasm_build_is_deterministic() {
        let a = build_restricted_wasm();
        let b = build_restricted_wasm();
        assert_eq!(a.wasm_bytes, b.wasm_bytes);
        assert_eq!(a.code_hash, b.code_hash);
    }

    #[test]
    fn manifests_have_stable_hashes() {
        let m = AlphaHIRManifest {
            plan_id: "p1".into(),
            code_hash_hex: "ab".into(),
            oracle_kinds: vec!["accuracy".into()],
            output_schema_id: "cbrn-sc.v1".into(),
            nullspec_id: "n0".into(),
        };
        assert_eq!(manifest_hash(&m).ok(), manifest_hash(&m).ok());
    }

    #[test]
    fn restricted_wasm_has_expected_abi_and_exports() {
        let wasm = build_restricted_wasm();
        let mut imports = Vec::new();
        let mut exports = Vec::new();
        let mut memories = 0usize;

        for payload in Parser::new(0).parse_all(&wasm.wasm_bytes) {
            match payload.expect("parse payload") {
                Payload::ImportSection(reader) => {
                    for i in reader {
                        let i = i.expect("import entry");
                        imports.push((
                            i.module.to_string(),
                            i.name.to_string(),
                            matches!(i.ty, TypeRef::Func(_)),
                        ));
                    }
                }
                Payload::MemorySection(reader) => {
                    memories += reader.count() as usize;
                }
                Payload::ExportSection(reader) => {
                    for e in reader {
                        let e = e.expect("export entry");
                        exports.push((e.name.to_string(), e.kind));
                    }
                }
                _ => {}
            }
        }

        assert_eq!(memories, 1);
        assert!(exports
            .iter()
            .any(|(n, k)| n == GUEST_EXPORT_RUN && *k == ExternalKind::Func));
        assert!(exports
            .iter()
            .any(|(n, k)| n == GUEST_EXPORT_MEMORY && *k == ExternalKind::Memory));

        assert!(imports
            .iter()
            .any(|(m, n, is_func)| m == VAULT_IMPORT_MODULE
                && n == VAULT_IMPORT_ORACLE_QUERY
                && *is_func));
        assert!(imports
            .iter()
            .any(|(m, n, is_func)| m == VAULT_IMPORT_MODULE
                && n == VAULT_IMPORT_EMIT_STRUCTURED_CLAIM
                && *is_func));
        assert!(imports
            .iter()
            .any(|(m, n, is_func)| m == VAULT_IMPORT_MODULE
                && n == VAULT_IMPORT_GET_LOGICAL_EPOCH
                && *is_func));

        for (module, name, _) in imports {
            assert_eq!(module, VAULT_IMPORT_MODULE);
            assert!([
                VAULT_IMPORT_ORACLE_QUERY,
                VAULT_IMPORT_EMIT_STRUCTURED_CLAIM,
                VAULT_IMPORT_GET_LOGICAL_EPOCH,
            ]
            .contains(&name.as_str()));
        }
        for (name, kind) in exports {
            match (name.as_str(), kind) {
                (GUEST_EXPORT_RUN, ExternalKind::Func)
                | (GUEST_EXPORT_MEMORY, ExternalKind::Memory) => {}
                _ => panic!("unexpected export"),
            }
        }
    }

    #[test]
    fn manifest_hash_is_stable_for_known_manifest() {
        let manifest = AlphaHIRManifest {
            plan_id: "example-plan".into(),
            code_hash_hex: "00".repeat(32),
            oracle_kinds: vec![VAULT_IMPORT_ORACLE_QUERY.into()],
            output_schema_id: "cbrn-sc.v1".into(),
            nullspec_id: "nullspec.v1".into(),
        };

        assert_eq!(
            hex::encode(manifest_hash(&manifest).expect("manifest hash")),
            "0a4f0f3ec1f7f43d347ec4f6f6209f26455d0b2f30d16cd11f33ed9f0e91ef99"
        );
    }

    proptest! {
        #[test]
        fn random_payloads_have_deterministic_wasm_and_manifest_hash(payload in prop::collection::vec(any::<u8>(), 0..256)) {
            let first = build_restricted_wasm_with_payload(&payload);
            let second = build_restricted_wasm_with_payload(&payload);
            prop_assert_eq!(first.wasm_bytes, second.wasm_bytes);
            prop_assert_eq!(first.code_hash, second.code_hash);

            let manifest_a = AlphaHIRManifest {
                plan_id: "prop-plan".to_string(),
                code_hash_hex: hex::encode(first.code_hash),
                oracle_kinds: vec![VAULT_IMPORT_ORACLE_QUERY.to_string()],
                output_schema_id: "cbrn-sc.v1".to_string(),
                nullspec_id: "nullspec.v1".to_string(),
            };
            let manifest_b = manifest_a.clone();

            let hash_a = manifest_hash(&manifest_a).map_err(|e| TestCaseError::fail(e.to_string()))?;
            let hash_b = manifest_hash(&manifest_b).map_err(|e| TestCaseError::fail(e.to_string()))?;
            prop_assert_eq!(hash_a, hash_b);
        }
    }
}

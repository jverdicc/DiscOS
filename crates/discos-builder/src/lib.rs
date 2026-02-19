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

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wasm_encoder::{
    CodeSection, DataSection, ExportKind, ExportSection, Function, FunctionSection, ImportSection,
    Instruction, MemorySection, MemoryType, Module, TypeSection, ValType,
};

const DOMAIN_WASM_HASH: &[u8] = b"evidenceos/wasm-code-hash/v1";
const DOMAIN_MANIFEST_HASH: &[u8] = b"evidenceos/manifest-hash/v1";
const PAYLOAD: &[u8] = &[0x01, 0x02, 0x03];

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
    imports.import("env", "oracle_query", wasm_encoder::EntityType::Function(0));
    imports.import(
        "env",
        "emit_structured_claim",
        wasm_encoder::EntityType::Function(1),
    );
    imports.import(
        "env",
        "get_logical_epoch",
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
    exports.export("run", ExportKind::Func, 3);
    exports.export("memory", ExportKind::Memory, 0);
    module.section(&exports);

    let mut data = DataSection::new();
    data.active(
        0,
        &wasm_encoder::ConstExpr::i32_const(0),
        PAYLOAD.iter().copied(),
    );
    module.section(&data);

    let mut code = CodeSection::new();
    let mut run = Function::new(vec![]);
    run.instruction(&Instruction::I32Const(0));
    run.instruction(&Instruction::I32Const(PAYLOAD.len() as i32));
    run.instruction(&Instruction::Call(0));
    run.instruction(&Instruction::Drop);
    run.instruction(&Instruction::I32Const(0));
    run.instruction(&Instruction::I32Const(PAYLOAD.len() as i32));
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

pub fn canonical_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

pub fn manifest_hash<T: Serialize>(value: &T) -> Result<[u8; 32], serde_json::Error> {
    let canonical = canonical_json(value)?;
    Ok(hash32(DOMAIN_MANIFEST_HASH, canonical.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
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
            .any(|(n, k)| n == "run" && *k == ExternalKind::Func));
        assert!(exports
            .iter()
            .any(|(n, k)| n == "memory" && *k == ExternalKind::Memory));

        assert!(imports
            .iter()
            .any(|(m, n, is_func)| m == "env" && n == "oracle_query" && *is_func));
        assert!(imports
            .iter()
            .any(|(m, n, is_func)| m == "env" && n == "emit_structured_claim" && *is_func));
        assert!(imports
            .iter()
            .any(|(m, n, is_func)| m == "env" && n == "get_logical_epoch" && *is_func));

        for (module, name, _) in imports {
            assert_eq!(module, "env");
            assert!(
                ["oracle_query", "emit_structured_claim", "get_logical_epoch"]
                    .contains(&name.as_str())
            );
        }
        for (name, kind) in exports {
            match (name.as_str(), kind) {
                ("run", ExternalKind::Func) | ("memory", ExternalKind::Memory) => {}
                _ => panic!("unexpected export"),
            }
        }
    }
}

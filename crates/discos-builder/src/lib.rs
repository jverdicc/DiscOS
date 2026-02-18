use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, ImportSection, Instruction,
    Module, TypeSection, ValType,
};

fn hash32<T: Hash>(value: &T) -> [u8; 32] {
    let mut out = [0u8; 32];
    for i in 0..4 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        i.hash(&mut h);
        value.hash(&mut h);
        out[i * 8..(i + 1) * 8].copy_from_slice(&h.finish().to_be_bytes());
    }
    out
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WasmBuildOutput {
    pub wasm_bytes: Vec<u8>,
    pub code_hash: [u8; 32],
}

pub fn build_restricted_wasm() -> WasmBuildOutput {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    let abi_sig = types
        .ty()
        .function(vec![ValType::I32, ValType::I32], vec![ValType::I32]);
    let epoch_sig = types.ty().function(vec![], vec![ValType::I64]);
    let run_sig = types.ty().function(vec![], vec![]);
    module.section(&types);

    let mut imports = ImportSection::new();
    imports.import("kernel", "oracle_query", abi_sig);
    imports.import("kernel", "emit_structured_claim", abi_sig);
    imports.import("kernel", "get_logical_epoch", epoch_sig);
    module.section(&imports);

    let mut funcs = FunctionSection::new();
    funcs.function(run_sig);
    module.section(&funcs);

    let mut exports = ExportSection::new();
    exports.export("run", ExportKind::Func, 3);
    module.section(&exports);

    let mut code = CodeSection::new();
    let mut run = Function::new(vec![]);
    run.instruction(&Instruction::I32Const(0));
    run.instruction(&Instruction::I32Const(0));
    run.instruction(&Instruction::Call(0));
    run.instruction(&Instruction::Drop);
    run.instruction(&Instruction::I32Const(0));
    run.instruction(&Instruction::I32Const(0));
    run.instruction(&Instruction::Call(1));
    run.instruction(&Instruction::Drop);
    run.instruction(&Instruction::Call(2));
    run.instruction(&Instruction::Drop);
    run.instruction(&Instruction::End);
    code.function(&run);
    module.section(&code);

    let wasm_bytes = module.finish();
    let code_hash = hash32(&wasm_bytes);
    WasmBuildOutput {
        wasm_bytes,
        code_hash,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AlphaHIRManifest {
    pub plan_id: String,
    pub code_hash_hex: String,
    pub oracle_kinds: Vec<String>,
    pub output_schema_id: String,
    pub nullspec_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PhysHIRManifest {
    pub physical_signature_hash: String,
    pub envelope_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CausalDSLManifest {
    pub dag_hash: String,
    pub adjustment_sets: Vec<Vec<String>>,
}

pub fn canonical_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("serialize manifest")
}

pub fn manifest_hash<T: Serialize + Hash>(value: &T) -> [u8; 32] {
    hash32(value)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(manifest_hash(&m), manifest_hash(&m));
    }

    #[test]
    fn wasm_fixture_is_valid_binary() {
        let out = build_restricted_wasm();
        assert!(out.wasm_bytes.starts_with(&[0x00, 0x61, 0x73, 0x6D]));
    }
}

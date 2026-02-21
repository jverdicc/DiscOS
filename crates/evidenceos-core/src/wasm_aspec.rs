use crate::guest_abi;
use thiserror::Error;
use wasmparser::{ExternalKind, Parser, Payload, TypeRef};

#[derive(Debug, Error)]
pub enum AspecError {
    #[error("wasm parse error: {0}")]
    Parse(#[from] wasmparser::BinaryReaderError),
    #[error("unexpected import module: {0}")]
    UnexpectedImportModule(String),
    #[error("missing required import: {0}")]
    MissingImport(&'static str),
    #[error("unexpected import: {0}")]
    UnexpectedImport(String),
    #[error("missing required export: {0}")]
    MissingExport(&'static str),
    #[error("unexpected export: {0}")]
    UnexpectedExport(String),
    #[error("restricted wasm must expose exactly one memory")]
    InvalidMemoryCount,
}

pub fn verify_restricted_wasm(wasm: &[u8]) -> Result<(), AspecError> {
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    let mut memories = 0usize;

    for payload in Parser::new(0).parse_all(wasm) {
        match payload? {
            Payload::ImportSection(reader) => {
                for import in reader {
                    let import = import?;
                    if !matches!(import.ty, TypeRef::Func(_)) {
                        continue;
                    }
                    if import.module != guest_abi::VAULT_IMPORT_MODULE {
                        return Err(AspecError::UnexpectedImportModule(
                            import.module.to_string(),
                        ));
                    }
                    imports.push(import.name.to_string());
                }
            }
            Payload::MemorySection(reader) => memories += reader.count() as usize,
            Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export?;
                    exports.push((export.name.to_string(), export.kind));
                }
            }
            _ => {}
        }
    }

    if memories != 1 {
        return Err(AspecError::InvalidMemoryCount);
    }

    let required_imports = [
        guest_abi::VAULT_IMPORT_ORACLE_QUERY,
        guest_abi::VAULT_IMPORT_EMIT_STRUCTURED_CLAIM,
        guest_abi::VAULT_IMPORT_GET_LOGICAL_EPOCH,
    ];
    for required in required_imports {
        if !imports.iter().any(|name| name == required) {
            return Err(AspecError::MissingImport(required));
        }
    }
    for import in imports {
        if !required_imports.contains(&import.as_str()) {
            return Err(AspecError::UnexpectedImport(import));
        }
    }

    let required_exports = [
        (guest_abi::GUEST_EXPORT_RUN, ExternalKind::Func),
        (guest_abi::GUEST_EXPORT_MEMORY, ExternalKind::Memory),
    ];
    for (required_name, required_kind) in required_exports {
        if !exports
            .iter()
            .any(|(name, kind)| name == required_name && *kind == required_kind)
        {
            return Err(AspecError::MissingExport(required_name));
        }
    }
    for (name, kind) in exports {
        let allowed = required_exports
            .iter()
            .any(|(required_name, required_kind)| name == *required_name && kind == *required_kind);
        if !allowed {
            return Err(AspecError::UnexpectedExport(name));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_wasm() {
        let err = verify_restricted_wasm(b"not wasm").expect_err("must reject");
        assert!(matches!(err, AspecError::Parse(_)));
    }
}

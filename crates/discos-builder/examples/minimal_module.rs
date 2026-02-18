use discos_builder::build_restricted_wasm;

fn main() {
    let out = build_restricted_wasm();
    println!(
        "wasm_size={} code_hash_len={}",
        out.wasm_bytes.len(),
        out.code_hash.len()
    );
}

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

use discos_builder::build_restricted_wasm;

fn main() {
    let out = build_restricted_wasm();
    println!(
        "wasm_size={} code_hash_len={}",
        out.wasm_bytes.len(),
        out.code_hash.len()
    );
}

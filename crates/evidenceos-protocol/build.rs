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

fn main() {
    let proto_file = "proto/evidenceos.proto";
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR set");
    let descriptor_path = std::path::Path::new(&out_dir).join("evidenceos_descriptor.bin");

    tonic_build::configure()
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(&[proto_file], &["proto"])
        .expect("compile evidenceos proto");

    println!("cargo:rerun-if-changed={proto_file}");
}

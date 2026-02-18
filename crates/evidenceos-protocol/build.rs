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

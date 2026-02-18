fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
    std::env::set_var("PROTOC", protoc);

    tonic_build::configure()
        .file_descriptor_set_path(
            std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"))
                .join("evidenceos_descriptor.bin"),
        )
        .compile_protos(&["proto/evidenceos.proto"], &["proto"])
        .expect("compile protos");
}

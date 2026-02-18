fn main() {
    let proto = "../../proto/evidenceos.proto";
    println!("cargo:rerun-if-changed={proto}");

    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
    // SAFETY: build scripts may mutate env vars in-process for downstream tools.
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    tonic_build::configure()
        .compile_protos(&[proto], &["../../proto"])
        .expect("compile proto");
}

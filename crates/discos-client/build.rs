fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto = "../../proto/evidenceos.proto";
    println!("cargo:rerun-if-changed={proto}");

    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    // SAFETY: build scripts may mutate env vars in-process for downstream tools.
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    tonic_build::configure().compile_protos(&[proto], &["../../proto"])?;
    Ok(())
}

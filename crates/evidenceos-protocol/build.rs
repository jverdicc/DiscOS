fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    std::env::set_var("PROTOC", protoc);

    let out_dir = std::env::var("OUT_DIR")?;
    tonic_build::configure()
        .file_descriptor_set_path(std::path::PathBuf::from(out_dir).join("evidenceos_descriptor.bin"))
        .compile_protos(&["proto/evidenceos.proto"], &["proto"])?;

    Ok(())
}

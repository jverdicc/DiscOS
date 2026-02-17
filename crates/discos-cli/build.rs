fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use a vendored protoc so contributors/CI don't need a system installation.
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    std::env::set_var("PROTOC", protoc);

    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(&["../../proto/evidenceos.proto"], &["../../proto"])?;

    println!("cargo:rerun-if-changed=../../proto/evidenceos.proto");
    Ok(())
}

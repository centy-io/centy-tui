fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile the proto file - use local copy for CI, or shared copy for development
    let proto_file = if std::path::Path::new("proto/centy.proto").exists() {
        "proto/centy.proto"
    } else {
        "../centy-daemon/proto/centy.proto"
    };

    let include_dir = if std::path::Path::new("proto").exists() {
        "proto"
    } else {
        "../centy-daemon/proto"
    };

    // Tell cargo to recompile if the proto file changes
    println!("cargo:rerun-if-changed={}", proto_file);

    tonic_build::configure()
        .build_server(false) // We only need the client
        .compile_protos(&[proto_file], &[include_dir])?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile the proto file from the shared centy-daemon proto directory
    let proto_file = "../centy-daemon/proto/centy.proto";

    // Tell cargo to recompile if the proto file changes
    println!("cargo:rerun-if-changed={}", proto_file);

    tonic_build::configure()
        .build_server(false) // We only need the client
        .compile_protos(&[proto_file], &["../centy-daemon/proto"])?;

    Ok(())
}

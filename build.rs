// - Configures protobuf compilation
// - Generates Rust code from proto definitions

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tell cargo to recompile if proto file changes
    println!("cargo:rerun-if-changed=proto/msg.proto");
    
    // Configure and compile protos
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(&["proto/msg.proto"], &["proto"])?;

    Ok(())
}
